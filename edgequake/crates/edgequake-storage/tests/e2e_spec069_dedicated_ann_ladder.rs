//! SPEC-069 — Dedicated per-workspace table ANN mid-scale ladder.
//!
//! Shape: namespace `*_ws_*` → dedicated table (= hot-set); global HNSW; no partial.
//! Env:
//! - `EQ_DEDICATED_ROWS_LIST` default `100000,125000,150000,200000`
//! - `EQ_DEDICATED_EF_LIST` default `80,240` (product default clamp vs concurrent tip)
//! - `EQ_DEDICATED_CONTENTION=1` (default) — clients∈{4,8,16} × scan_mem on first fail
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_stress.rs"]
mod perf_stress;
#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType, VectorStorageMode};
use perf_ann_corpus::{
    emb, measure_single, measure_stress, seed_single_ws, workspace_filter,
};
use perf_stress::{ceiling_hang_cliff_ms, stress_mult, stress_pool_max, with_stress_pool};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TOP_K: usize = 20;
const Q1D_SLO_MS: f64 = 500.0;
const RECALL_GATE: f64 = 0.99;
const DIM: usize = 1536;
const WS: &str = "ws-dedicated";
const TENANT: &str = "t-ded069";
const REF_EF: u32 = 400;

fn parse_u32_list(env: &str, default: &[u32]) -> Vec<u32> {
    match std::env::var(env) {
        Ok(s) if !s.trim().is_empty() => s
            .split(',')
            .filter_map(|p| p.trim().parse().ok())
            .collect(),
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

async fn explain_hnsw(
    config: &edgequake_storage::PostgresConfig,
    table: &str,
    emb_type: &str,
    index_name: &str,
) -> String {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    let _ = sqlx::query(&format!("ANALYZE {table}")).execute(&pool).await;
    let emb: String = {
        let vals: Vec<String> = (0..DIM)
            .map(|i| format!("{:.8}", ((i as f32 + 10.0) * 0.019).sin()))
            .collect();
        format!("[{}]", vals.join(","))
    };
    let mut tx = pool.begin().await.expect("explain tx");
    for stmt in [
        "SET LOCAL hnsw.ef_search = 80",
        "SET LOCAL hnsw.iterative_scan = relaxed_order",
        "SET LOCAL enable_seqscan = off",
        "SET LOCAL random_page_cost = 1.1",
    ] {
        let _ = sqlx::query(stmt).execute(&mut *tx).await;
    }
    let sql = format!(
        r#"EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
           SELECT id, 1 - (embedding <=> $1::{emb_type}) AS score
           FROM {table}
           WHERE workspace_id = $2 AND tenant_id = $3 AND metadata->>'type' = 'chunk'
           ORDER BY embedding <=> $1::{emb_type}
           LIMIT 20"#
    );
    let plan_rows: Vec<(String,)> = sqlx::query_as(&sql)
        .bind(&emb)
        .bind(WS)
        .bind(TENANT)
        .fetch_all(&mut *tx)
        .await
        .unwrap_or_default();
    let _ = tx.commit().await;
    let plan = plan_rows
        .into_iter()
        .map(|r| r.0)
        .collect::<Vec<_>>()
        .join("\n");
    let uses_hnsw = plan.to_lowercase().contains("index scan")
        && plan.to_lowercase().contains(&index_name.to_lowercase());
    let uses_sort = plan.to_lowercase().contains("sort");
    format!("uses_hnsw={uses_hnsw} uses_sort={uses_sort}\n{plan}")
}

async fn measure_recall(
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
            .expect("ref");
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", query_ef.to_string());
        let lo = storage
            .query_filtered(&q, TOP_K, None, Some(mf))
            .await
            .expect("cand");
        let hi_ids: Vec<_> = hi.iter().map(|h| h.id.clone()).collect();
        let lo_ids: Vec<_> = lo.iter().map(|h| h.id.clone()).collect();
        recalls.push(recall_at_k(&hi_ids, &lo_ids));
    }
    recalls.iter().sum::<f64>() / recalls.len() as f64
}

struct CellResult {
    full_green: bool,
    recall_ok: bool,
    abs_ok: bool,
    single_p95: f64,
    stress_p95: f64,
}

async fn run_cell(
    storage: &Arc<PgVectorStorage>,
    rows: u32,
    query_ef: u32,
    clients: usize,
    pool: u32,
    cliff: f64,
    mult: f64,
    arm: &str,
) -> CellResult {
    let mf = workspace_filter(WS, TENANT);
    std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", query_ef.to_string());

    let recall = measure_recall(storage, &mf, query_ef).await;
    let recall_ok = recall >= RECALL_GATE;
    emit(
        "dedicated_recall",
        recall * 1000.0,
        recall_ok,
        "recall_vs_ef400",
        format!(
            "arm={arm} rows={rows} ef={query_ef} clients={clients} \
             recall@20_ann_mean={recall:.4} gate={RECALL_GATE}"
        ),
        &[],
    );

    let (single_p95, single) = measure_single(storage, DIM, &mf, TOP_K).await;
    let slo_pass = single_p95 < Q1D_SLO_MS;
    assert!(
        single_p95 < cliff,
        "hang cliff: single {single_p95:.2} >= {cliff}"
    );
    emit(
        "dedicated_single",
        single_p95,
        slo_pass && recall_ok,
        "hnsw_dedicated",
        format!(
            "arm={arm} rows={rows} ef={query_ef} clients={clients} pool={pool} \
             slo_pass={slo_pass} recall_ok={recall_ok}"
        ),
        &single,
    );

    let (stress_p95, all, wall) =
        measure_stress(Arc::clone(storage), DIM, mf, clients, 20, TOP_K).await;
    let abs_ok = stress_p95 < Q1D_SLO_MS;
    let rel_ok = stress_p95 < (single_p95 * mult).max(50.0);
    let full_green = slo_pass && recall_ok && abs_ok;
    emit(
        "dedicated_stress",
        stress_p95,
        full_green,
        "hnsw_dedicated",
        format!(
            "arm={arm} rows={rows} ef={query_ef} clients={clients} single_p95={single_p95:.2} \
             abs_ok={abs_ok} rel_ok={rel_ok} wall={wall:?} full_green={full_green}"
        ),
        &all,
    );
    if full_green {
        eprintln!("GREEN SPEC-069: arm={arm} rows={rows} ef={query_ef} clients={clients}");
    } else {
        eprintln!(
            "NOTE SPEC-069: not green arm={arm} rows={rows} ef={query_ef} clients={clients} \
             recall_ok={recall_ok} slo_pass={slo_pass} abs_ok={abs_ok}"
        );
    }
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");
    CellResult {
        full_green,
        recall_ok,
        abs_ok,
        single_p95,
        stress_p95,
    }
}

async fn seed_dedicated(rows: usize) -> (Arc<PgVectorStorage>, edgequake_storage::PostgresConfig) {
    let clients = 16usize;
    let base = postgres_test_config::require_or_skip_postgres("ded069")
        .expect("DATABASE_URL required");
    let mut config = with_stress_pool(base, clients).with_vector_index(VectorIndexType::None);
    config.namespace = format!("eq_ded069_ws_{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let storage = Arc::new(
        PgVectorStorage::with_dimension(config.clone(), DIM)
            .with_storage_mode(VectorStorageMode::Half),
    );
    storage.initialize().await.expect("init");
    assert!(
        storage.is_dedicated_workspace_table(),
        "must be dedicated *_ws_* table"
    );

    let seed_ms = seed_single_ws(&storage, rows, DIM, 1000, "ded069", TENANT, WS).await;
    let index_wall = Instant::now();
    storage.ensure_ann_index().await.expect("ensure_ann_index");
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    emit(
        "dedicated_index",
        index_ms,
        true,
        "hnsw_create",
        format!(
            "rows={rows} dedicated=true seed_ms={seed_ms:.0} table={}",
            storage.vectors_table_name()
        ),
        &[Duration::from_secs_f64(index_ms / 1000.0)],
    );

    let explain = explain_hnsw(
        &config,
        storage.vectors_table_name(),
        storage.embedding_sql_type(),
        &storage.ann_index_name(),
    )
    .await;
    emit(
        "dedicated_explain",
        0.0,
        explain.contains("uses_hnsw=true"),
        "explain",
        explain.chars().take(4000).collect::<String>(),
        &[],
    );
    (storage, config)
}

#[tokio::test]
async fn e2e_spec069_dedicated_midscale_ladder() {
    let rows_list = parse_u32_list(
        "EQ_DEDICATED_ROWS_LIST",
        &[100_000, 125_000, 150_000, 200_000],
    );
    let ef_list = parse_u32_list("EQ_DEDICATED_EF_LIST", &[80, 240]);
    let do_contention = std::env::var("EQ_DEDICATED_CONTENTION")
        .map(|v| v != "0")
        .unwrap_or(true);

    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    // Dedicated path skips partial; leave flag off to match prod dedicated shape.
    std::env::remove_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE");
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");
    std::env::remove_var("EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER");

    if postgres_test_config::require_or_skip_postgres("ded069").is_none() {
        return;
    }

    let gate_clients = 16usize;
    let pool = stress_pool_max(gate_clients);
    let mult = stress_mult();

    let mut first_fail: Option<(u32, u32)> = None;
    let mut green_150k = false;

    for &rows in &rows_list {
        let cliff = ceiling_hang_cliff_ms(rows as usize);
        let (storage, _cfg) = seed_dedicated(rows as usize).await;
        for &ef in &ef_list {
            let cell = run_cell(
                &storage,
                rows,
                ef,
                gate_clients,
                pool,
                cliff,
                mult,
                "ladder",
            )
            .await;
            if rows == 150_000 && cell.full_green {
                green_150k = true;
            }
            if first_fail.is_none() && !cell.full_green && cell.recall_ok && !cell.abs_ok {
                first_fail = Some((rows, ef));
            }
            if first_fail.is_none() && !cell.full_green {
                first_fail = Some((rows, ef));
            }
        }
    }

    if do_contention {
        let (fail_rows, fail_ef) = first_fail.unwrap_or((150_000, 240));
        eprintln!(
            "NOTE SPEC-069: contention matrix at rows={fail_rows} ef={fail_ef} \
             (diagnostic; promote only clients=16)"
        );
        let cliff = ceiling_hang_cliff_ms(fail_rows as usize);
        let (storage, _) = seed_dedicated(fail_rows as usize).await;
        for &clients in &[4usize, 8, 16] {
            for scan_mem in &[None, Some(2u32)] {
                match scan_mem {
                    Some(m) => std::env::set_var("EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER", m.to_string()),
                    None => std::env::remove_var("EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER"),
                }
                let arm = match scan_mem {
                    Some(_) => "contention_scanmem2",
                    None => "contention",
                };
                let pool_c = stress_pool_max(clients);
                let cell = run_cell(
                    &storage,
                    fail_rows,
                    fail_ef,
                    clients,
                    pool_c,
                    cliff,
                    mult,
                    arm,
                )
                .await;
                emit(
                    "dedicated_contention_summary",
                    cell.stress_p95,
                    cell.full_green && clients == 16,
                    "contention",
                    format!(
                        "rows={fail_rows} ef={fail_ef} clients={clients} scan_mem={:?} \
                         single_p95={:.2} stress_p95={:.2} recall_ok={} abs_ok={} \
                         promote_eligible={}",
                        scan_mem,
                        cell.single_p95,
                        cell.stress_p95,
                        cell.recall_ok,
                        cell.abs_ok,
                        cell.full_green && clients == 16
                    ),
                    &[],
                );
            }
        }
        std::env::remove_var("EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER");
    }

    emit(
        "dedicated_decision",
        if green_150k { 1.0 } else { 0.0 },
        green_150k,
        "promote",
        format!(
            "green_150k={green_150k} first_fail={first_fail:?} \
             open_spec070={}",
            !green_150k
        ),
        &[],
    );
}
