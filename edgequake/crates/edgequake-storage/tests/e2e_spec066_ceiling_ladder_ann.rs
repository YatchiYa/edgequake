//! SPEC-066 — Wave-2 ceiling ladder (halfvec + partial HNSW + column filter).
//!
//! Env:
//! - `EDGEQUAKE_VECTOR_STORAGE=halfvec`
//! - `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1`
//! - `EDGEQUAKE_CEILING_ROWS` or `EQ_CEILING_STEP=L2|L3|SEEK`
//!
//! Completes under hang cliff even when Q1-d / recall SLO misses (measured cliff).
//! Hard-fail only on hang cliff (FORBIDDEN / host undersized).
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
use perf_ann_corpus::{emb, measure_single, measure_stress, seed_ws_split, workspace_filter};
use perf_harness::percentile_p95_ms;
use perf_stress::{
    ceiling_corpus_rows, ceiling_hang_cliff_ms, stress_clients, stress_mult, stress_pool_max,
    with_stress_pool,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TOP_K: usize = 20;
const Q1D_SLO_MS: f64 = 500.0;
const RECALL_GATE: f64 = 0.99;
const DIM: usize = 1536;
const WS: &str = "ws-a";
const TENANT: &str = "t-ceil066";

fn recall_at_k(reference: &[String], candidate: &[String]) -> f64 {
    if reference.is_empty() {
        return 1.0;
    }
    let set: std::collections::HashSet<&String> = candidate.iter().collect();
    let hit = reference.iter().filter(|id| set.contains(id)).count();
    hit as f64 / reference.len() as f64
}

fn emb_literal(dim: usize, seed: f32) -> String {
    let vals: Vec<String> = (0..dim)
        .map(|i| format!("{:.8}", ((i as f32 + seed) * 0.019).sin()))
        .collect();
    format!("[{}]", vals.join(","))
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

/// Exact filtered top-k (seq scan) — ground truth for recall@20 (SPEC-064 style honesty).
async fn exact_topk_ids(
    config: &edgequake_storage::PostgresConfig,
    table: &str,
    emb_type: &str,
    seed: f32,
) -> Vec<String> {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    let emb = emb_literal(DIM, seed);
    let mut tx = pool.begin().await.expect("begin exact");
    sqlx::query("SET LOCAL enable_indexscan = off")
        .execute(&mut *tx)
        .await
        .ok();
    sqlx::query("SET LOCAL enable_bitmapscan = off")
        .execute(&mut *tx)
        .await
        .ok();
    sqlx::query("SET LOCAL enable_indexonlyscan = off")
        .execute(&mut *tx)
        .await
        .ok();
    let q = format!(
        r#"
        SELECT id
        FROM {table}
        WHERE workspace_id = $1 AND tenant_id = $2 AND metadata->>'type' = 'chunk'
        ORDER BY embedding <=> $3::{emb_type}
        LIMIT {TOP_K}
        "#
    );
    let rows: Vec<(String,)> = sqlx::query_as(&q)
        .bind(WS)
        .bind(TENANT)
        .bind(&emb)
        .fetch_all(&mut *tx)
        .await
        .expect("exact topk");
    tx.commit().await.ok();
    rows.into_iter().map(|r| r.0).collect()
}

async fn explain_partial_plan(
    config: &edgequake_storage::PostgresConfig,
    table: &str,
    emb_type: &str,
    partial_name: &str,
) -> String {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    let _ = sqlx::query(&format!("ANALYZE {table}")).execute(&pool).await;
    let emb = emb_literal(DIM, 10.0);
    // Mirror SPEC-067 production Wave-2 planner bias (session-local; columns-only path).
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
    let uses_partial = plan.to_lowercase().contains(&partial_name.to_lowercase());
    format!("uses_partial={uses_partial}\n{plan}")
}

#[tokio::test]
async fn e2e_spec066_ceiling_wave2_filtered_ann() {
    let rows = ceiling_corpus_rows();
    let cliff = ceiling_hang_cliff_ms(rows);
    let clients = stress_clients();
    let mult = stress_mult();
    let pool = stress_pool_max(clients);
    let step = std::env::var("EQ_CEILING_STEP").unwrap_or_else(|_| "L2".into());

    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE", "1");
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS", "1000");
    // Battle knee tip (ops) — keep product default path honest for latency.
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");

    let Some(base) = postgres_test_config::require_or_skip_postgres("ceil066_ann") else {
        return;
    };
    let config = with_stress_pool(base, clients).with_vector_index(VectorIndexType::None);
    let storage = Arc::new(
        PgVectorStorage::with_dimension(config.clone(), DIM).with_storage_mode(VectorStorageMode::Half),
    );
    storage.initialize().await.expect("init");

    let seed_ms = seed_ws_split(
        &storage,
        rows,
        DIM,
        1000,
        "ceil066",
        TENANT,
        WS,
        "ws-b",
    )
    .await;

    let index_wall = Instant::now();
    // Wave-2 battle shape: partial for hot WS; drop global so planner cannot prefer
    // btree→exact on the 20% slice (SPEC-064). Prod keeps global as cold/small fallback.
    let created = storage
        .ensure_hot_workspace_ann(WS)
        .await
        .expect("ensure_hot_workspace_ann");
    assert!(
        storage
            .partial_ann_index_exists(WS)
            .await
            .expect("partial probe"),
        "Wave-2 partial HNSW must exist for hot workspace"
    );
    storage
        .drop_global_ann_index()
        .await
        .expect("drop global to force partial path");
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    emit(
        "ceiling_wave2_index",
        index_ms,
        true,
        "hnsw_create",
        format!(
            "step={step} rows={rows} dim={DIM} seed_ms={seed_ms:.0} partial_created={created}"
        ),
        &[Duration::from_secs_f64(index_ms / 1000.0)],
    );

    let partial_name = storage.partial_ann_index_name(WS);
    let explain = explain_partial_plan(
        &config,
        storage.vectors_table_name(),
        storage.embedding_sql_type(),
        &partial_name,
    )
    .await;
    emit(
        "ceiling_wave2_explain",
        0.0,
        explain.contains("uses_partial=true"),
        "explain",
        explain.chars().take(4000).collect::<String>(),
        &[],
    );

    let mf = workspace_filter(WS, TENANT);

    // Recall gates (two axes; SPEC-064 used halfvec-vs-full ANN, not exact):
    // 1) vs high-ef ANN reference (product-comparable quality)
    // 2) vs exact filtered top-k (stricter physics; may cliff earlier)
    let mut recalls_ann = Vec::new();
    let mut recalls_exact = Vec::new();
    for s in 0..5 {
        let seed = (s + 42) as f32;
        let q = emb(DIM, seed);
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", "400");
        let hi = storage
            .query_filtered(&q, TOP_K, None, Some(&mf))
            .await
            .expect("ann hi");
        std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");
        let lo = storage
            .query_filtered(&q, TOP_K, None, Some(&mf))
            .await
            .expect("ann lo");
        let hi_ids: Vec<_> = hi.iter().map(|h| h.id.clone()).collect();
        let lo_ids: Vec<_> = lo.iter().map(|h| h.id.clone()).collect();
        recalls_ann.push(recall_at_k(&hi_ids, &lo_ids));

        let exact_ids = exact_topk_ids(
            &config,
            storage.vectors_table_name(),
            storage.embedding_sql_type(),
            seed,
        )
        .await;
        recalls_exact.push(recall_at_k(&exact_ids, &lo_ids));
    }
    let recall_ann_mean = recalls_ann.iter().sum::<f64>() / recalls_ann.len() as f64;
    let recall_exact_mean = recalls_exact.iter().sum::<f64>() / recalls_exact.len() as f64;
    // Product gate matches SPEC-064: ANN-relative ≥0.99. Exact is honesty-only.
    let recall_ok = recall_ann_mean >= RECALL_GATE;
    emit(
        "ceiling_wave2_recall",
        recall_ann_mean * 1000.0,
        recall_ok,
        "recall_vs_ef400",
        format!(
            "recall@20_ann_mean={recall_ann_mean:.4} recall@20_exact_mean={recall_exact_mean:.4} \
             gate={RECALL_GATE} ann_samples={recalls_ann:?} exact_samples={recalls_exact:?}"
        ),
        &[],
    );
    if !recall_ok {
        eprintln!(
            "WARN SPEC-066: ANN-relative recall@20 {recall_ann_mean:.4} < {RECALL_GATE} at rows={rows} — quality cliff"
        );
    }
    if recall_exact_mean < RECALL_GATE {
        eprintln!(
            "NOTE SPEC-066: exact recall@20 {recall_exact_mean:.4} < {RECALL_GATE} (honesty; not sole promote gate)"
        );
    }

    let (single_p95, single) = measure_single(&storage, DIM, &mf, TOP_K).await;
    let slo_pass = single_p95 < Q1D_SLO_MS;
    let under_cliff = single_p95 < cliff;
    let rung_green = slo_pass && recall_ok;
    emit(
        "ceiling_wave2_single",
        single_p95,
        rung_green,
        "hnsw_partial_ws",
        format!(
            "step={step} rows={rows} dim={DIM} pool={pool} q1d_slo_ms={Q1D_SLO_MS} cliff_ms={cliff} \
             slo_pass={slo_pass} recall_ok={recall_ok} rung_green={rung_green} \
             storage=halfvec index=partial_ws"
        ),
        &single,
    );
    assert!(
        under_cliff,
        "single p95 {single_p95:.2}ms exceeds hang cliff {cliff}ms — FORBIDDEN / host undersized"
    );
    if !slo_pass {
        eprintln!(
            "WARN SPEC-066: Q1-d SLO miss at rows={rows} (p95={single_p95:.2}ms) — measured latency cliff; do not promote"
        );
    }

    let qpc = 20usize;
    let (stress_p95, all, stress_wall) =
        measure_stress(Arc::clone(&storage), DIM, mf.clone(), clients, qpc, TOP_K).await;
    let abs_ok = stress_p95 < Q1D_SLO_MS;
    let rel_budget = (single_p95 * mult).max(50.0);
    let rel_ok = stress_p95 < rel_budget;
    let stress_green = abs_ok && rung_green;
    emit(
        "ceiling_wave2_stress",
        stress_p95,
        stress_green,
        "hnsw_partial_ws",
        format!(
            "step={step} rows={rows} clients={clients} q/client={qpc} pool={pool} \
             single_p95={single_p95:.2} mult={mult} abs_ok={abs_ok} rel_ok={rel_ok} \
             rel_budget={rel_budget:.2} wall={stress_wall:?}"
        ),
        &all,
    );
    if rung_green && !abs_ok {
        eprintln!(
            "WARN SPEC-066: concurrent absolute Q1-d miss p95={stress_p95:.2}ms — measured concurrent cliff"
        );
    }

    // Ephemeral PG is discarded by the harness — skip clear() (500k DELETE is a multi-minute hang).
    let _ = percentile_p95_ms(&single);

    // Soft-fail product gates: hang cliff already asserted. Cargo exit 0 so harness archives cliffs.
    if !rung_green || !abs_ok {
        eprintln!(
            "NOTE SPEC-066: rung not green (rung_green={rung_green} abs_ok={abs_ok}) — archive as cliff"
        );
    }
}
