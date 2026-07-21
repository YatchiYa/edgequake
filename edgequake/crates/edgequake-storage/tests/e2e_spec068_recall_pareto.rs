//! SPEC-068 — Recall × latency Pareto (Wave-2 + SPEC-067 planner bias).
//!
//! Env:
//! - `EQ_PARETO_ROWS_LIST` — comma list (default `100000,150000,200000,250000`)
//! - `EQ_PARETO_EF_LIST` — comma list (default `80,160,240,400`)
//! - `EQ_PARETO_REBUILD=1` — optional rebuild arm (`ef_construction=128`, `m=32`) at max N
//!
//! Soft-fails product gates (archives cliffs). Hang cliff hard-fails.
#![cfg(feature = "postgres")]

#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_stress.rs"]
mod perf_stress;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType, VectorStorageMode};
use perf_ann_corpus::{emb, measure_single, measure_stress, seed_ws_split, workspace_filter};
use perf_stress::{
    ceiling_hang_cliff_ms, stress_clients, stress_mult, stress_pool_max, with_stress_pool,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TOP_K: usize = 20;
const Q1D_SLO_MS: f64 = 500.0;
const RECALL_GATE: f64 = 0.99;
const DIM: usize = 1536;
const WS: &str = "ws-a";
const TENANT: &str = "t-pareto068";
const REF_EF: u32 = 400;

fn parse_u32_list(env: &str, default: &[u32]) -> Vec<u32> {
    match std::env::var(env) {
        Ok(s) if !s.trim().is_empty() => {
            s.split(',').filter_map(|p| p.trim().parse().ok()).collect()
        }
        _ => default.to_vec(),
    }
}

fn recall_at_k(reference: &[String], candidate: &[String]) -> f64 {
    if reference.is_empty() {
        return 1.0;
    }
    let set: std::collections::HashSet<&String> = candidate.iter().collect();
    let hit = reference.iter().filter(|id| set.contains(id)).count();
    hit as f64 / reference.len() as f64
}

fn emit(
    op: &str,
    p95_ms: f64,
    pass: bool,
    plan_class: &str,
    detail: impl Into<String>,
    samples: &[Duration],
) {
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": op,
            "p95_ms": p95_ms,
            "samples_ms": samples.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
            "plan_class": plan_class,
            "pass": pass,
            "detail": detail.into(),
        })
    );
}

async fn measure_recall_vs_ref(
    storage: &PgVectorStorage,
    mf: &edgequake_storage::traits::MetadataFilter,
    query_ef: u32,
) -> f64 {
    let mut recalls = Vec::new();
    for s in 0..5 {
        let seed = (s + 42) as f32;
        let q = emb(DIM, seed);
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", REF_EF.to_string());
        let hi = storage
            .query_filtered(&q, TOP_K, None, Some(mf))
            .await
            .expect("ann ref");
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", query_ef.to_string());
        let lo = storage
            .query_filtered(&q, TOP_K, None, Some(mf))
            .await
            .expect("ann cand");
        let hi_ids: Vec<_> = hi.iter().map(|h| h.id.clone()).collect();
        let lo_ids: Vec<_> = lo.iter().map(|h| h.id.clone()).collect();
        recalls.push(recall_at_k(&hi_ids, &lo_ids));
    }
    recalls.iter().sum::<f64>() / recalls.len() as f64
}

#[allow(clippy::too_many_arguments)] // pareto cell knobs stay explicit for emit labels
async fn run_cell(
    storage: &Arc<PgVectorStorage>,
    rows: u32,
    query_ef: u32,
    arm: &str,
    clients: usize,
    pool: u32,
    cliff: f64,
    mult: f64,
) {
    let mf = workspace_filter(WS, TENANT);
    std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", query_ef.to_string());

    let recall = measure_recall_vs_ref(storage, &mf, query_ef).await;
    let recall_ok = recall >= RECALL_GATE;
    emit(
        "pareto_recall",
        recall * 1000.0,
        recall_ok,
        "recall_vs_ef400",
        format!(
            "arm={arm} rows={rows} ef={query_ef} recall@20_ann_mean={recall:.4} gate={RECALL_GATE}"
        ),
        &[],
    );

    let (single_p95, single) = measure_single(storage, DIM, &mf, TOP_K).await;
    let slo_pass = single_p95 < Q1D_SLO_MS;
    let under_cliff = single_p95 < cliff;
    assert!(
        under_cliff,
        "single p95 {single_p95:.2}ms exceeds hang cliff {cliff}ms"
    );
    let rung_latency = slo_pass && recall_ok;
    emit(
        "pareto_single",
        single_p95,
        rung_latency,
        "hnsw_partial_ws",
        format!(
            "arm={arm} rows={rows} ef={query_ef} pool={pool} slo_pass={slo_pass} \
             recall_ok={recall_ok} q1d_slo_ms={Q1D_SLO_MS}"
        ),
        &single,
    );

    let qpc = 20usize;
    let (stress_p95, all, stress_wall) =
        measure_stress(Arc::clone(storage), DIM, mf.clone(), clients, qpc, TOP_K).await;
    let abs_ok = stress_p95 < Q1D_SLO_MS;
    let rel_budget = (single_p95 * mult).max(50.0);
    let rel_ok = stress_p95 < rel_budget;
    let full_green = rung_latency && abs_ok;
    emit(
        "pareto_stress",
        stress_p95,
        full_green,
        "hnsw_partial_ws",
        format!(
            "arm={arm} rows={rows} ef={query_ef} clients={clients} single_p95={single_p95:.2} \
             abs_ok={abs_ok} rel_ok={rel_ok} rel_budget={rel_budget:.2} wall={stress_wall:?} \
             full_green={full_green}"
        ),
        &all,
    );
    if full_green {
        eprintln!("GREEN SPEC-068: arm={arm} rows={rows} ef={query_ef} (full gate)");
    } else {
        eprintln!(
            "NOTE SPEC-068: not green arm={arm} rows={rows} ef={query_ef} \
             recall_ok={recall_ok} slo_pass={slo_pass} abs_ok={abs_ok}"
        );
    }
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");
}

async fn seed_indexed(rows: usize, m: u32, ef_c: u32, arm: &str) -> Arc<PgVectorStorage> {
    let clients = stress_clients();
    let base = postgres_test_config::require_or_skip_postgres("pareto068")
        .expect("DATABASE_URL required under EDGEQUAKE_REQUIRE_POSTGRES_TESTS");
    let mut config = with_stress_pool(base, clients).with_vector_index(VectorIndexType::None);
    config.hnsw_m = m;
    config.hnsw_ef_construction = ef_c;

    let storage = Arc::new(
        PgVectorStorage::with_dimension(config.clone(), DIM)
            .with_storage_mode(VectorStorageMode::Half),
    );
    storage.initialize().await.expect("init");
    let seed_ms = seed_ws_split(&storage, rows, DIM, 1000, "pareto068", TENANT, WS, "ws-b").await;
    let index_wall = Instant::now();
    let created = storage
        .ensure_hot_workspace_ann(WS)
        .await
        .expect("ensure_hot_workspace_ann");
    assert!(
        storage
            .partial_ann_index_exists(WS)
            .await
            .expect("partial probe"),
        "Wave-2 partial must exist"
    );
    storage.drop_global_ann_index().await.expect("drop global");
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    emit(
        "pareto_index",
        index_ms,
        true,
        "hnsw_create",
        format!(
            "arm={arm} rows={rows} m={m} ef_construction={ef_c} seed_ms={seed_ms:.0} \
             partial_created={created}"
        ),
        &[Duration::from_secs_f64(index_ms / 1000.0)],
    );
    storage
}

#[tokio::test]
async fn e2e_spec068_recall_latency_pareto() {
    let rows_list = parse_u32_list("EQ_PARETO_ROWS_LIST", &[100_000, 150_000, 200_000, 250_000]);
    let ef_list = parse_u32_list("EQ_PARETO_EF_LIST", &[80, 160, 240, 400]);
    let do_rebuild = std::env::var("EQ_PARETO_REBUILD").ok().as_deref() == Some("1");

    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE", "1");
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS", "1000");
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");

    if postgres_test_config::require_or_skip_postgres("pareto068").is_none() {
        return;
    }

    let clients = stress_clients();
    let pool = stress_pool_max(clients);
    let mult = stress_mult();

    for &rows in &rows_list {
        let cliff = ceiling_hang_cliff_ms(rows as usize);
        let storage = seed_indexed(rows as usize, 16, 64, "query_ef").await;
        for &ef in &ef_list {
            // ef=400 vs ref ef=400 is tautological ~1.0 — still useful as latency cell
            run_cell(&storage, rows, ef, "query_ef", clients, pool, cliff, mult).await;
        }
    }

    if do_rebuild {
        let rows = *rows_list.iter().max().unwrap_or(&250_000);
        let cliff = ceiling_hang_cliff_ms(rows as usize);
        let storage = seed_indexed(rows as usize, 32, 128, "rebuild_m32_efc128").await;
        for &ef in &[80u32, 160, 240] {
            run_cell(
                &storage,
                rows,
                ef,
                "rebuild_m32_efc128",
                clients,
                pool,
                cliff,
                mult,
            )
            .await;
        }
    }
}
