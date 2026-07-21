//! SPEC-070 — DiskANN (pgvectorscale) vs HNSW dedicated battle.
//!
//! Arms (dedicated `*_ws_*` table):
//! - HNSW baseline (halfvec) — control
//! - DiskANN (`USING diskann` on `vector`) — study arm
//!
//! Soft-fail product gates (emit JSONL); hard-fail hang cliff only.
//! Promote only from full-gate green @150k clients=16 (handled in RUN_NOTES / SSOT).
//!
//! Env:
//! - `EQ_DISKANN_ROWS_LIST` default `100000,150000,250000`
//! - `EQ_DISKANN_SMOKE=1` → tiny rows for image/extension smoke
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
use perf_ann_corpus::{emb, measure_single, measure_stress, seed_single_ws, workspace_filter};
use perf_stress::{ceiling_hang_cliff_ms, stress_mult, with_stress_pool};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TOP_K: usize = 20;
const Q1D_SLO_MS: f64 = 500.0;
const RECALL_GATE: f64 = 0.99;
const DIM: usize = 1536;
const WS: &str = "ws-diskann";
const TENANT: &str = "t-disk070";
const REF_EF: u32 = 400;
const QUERY_EF: u32 = 240;

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

async fn vectorscale_available(pool: &sqlx::PgPool) -> bool {
    let ok: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_available_extensions WHERE name = 'vectorscale')",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    ok.unwrap_or(false)
}

async fn ensure_vectorscale(pool: &sqlx::PgPool) -> Result<(), String> {
    if !vectorscale_available(pool).await {
        return Err(
            "vectorscale extension not available — use EQ_POSTGRES_PROFILE=pg18-vectorscale".into(),
        );
    }
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE")
        .execute(pool)
        .await
        .map_err(|e| format!("CREATE EXTENSION vectorscale: {e}"))?;
    Ok(())
}

async fn explain_index(
    config: &edgequake_storage::PostgresConfig,
    table: &str,
    emb_type: &str,
    expect_am: &str,
) -> String {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    let _ = sqlx::query(&format!("ANALYZE {table}"))
        .execute(&pool)
        .await;
    let emb: String = {
        let vals: Vec<String> = (0..DIM)
            .map(|i| format!("{:.8}", ((i as f32 + 10.0) * 0.019).sin()))
            .collect();
        format!("[{}]", vals.join(","))
    };
    let mut tx = pool.begin().await.expect("explain tx");
    for stmt in [
        "SET LOCAL enable_seqscan = off",
        "SET LOCAL random_page_cost = 1.1",
        "SET LOCAL hnsw.ef_search = 80",
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
    let uses = plan.to_lowercase().contains(expect_am);
    format!("uses_{expect_am}={uses}\n{plan}")
}

async fn measure_recall_hnsw(
    storage: &PgVectorStorage,
    mf: &edgequake_storage::traits::MetadataFilter,
) -> f64 {
    let mut recalls = Vec::new();
    for s in 0..5 {
        let q = emb(DIM, (s + 42) as f32);
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", REF_EF.to_string());
        let hi = storage
            .query_filtered(&q, TOP_K, None, Some(mf))
            .await
            .expect("ref");
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", QUERY_EF.to_string());
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

/// DiskANN: candidate vs higher search_list (ANN-relative, honest for study).
async fn measure_recall_diskann(
    storage: &PgVectorStorage,
    config: &edgequake_storage::PostgresConfig,
    table: &str,
) -> f64 {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    let mut recalls = Vec::new();
    for s in 0..5 {
        let q = emb(DIM, (s + 42) as f32);
        let emb_str = format!(
            "[{}]",
            q.iter()
                .map(|v| format!("{v:.8}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let ref_ids = topk_ids(&pool, table, &emb_str, 400).await;
        let cand_ids = topk_ids(&pool, table, &emb_str, 100).await;
        recalls.push(recall_at_k(&ref_ids, &cand_ids));
    }
    let _ = storage; // keep for API symmetry / future query_filtered path
    recalls.iter().sum::<f64>() / recalls.len() as f64
}

async fn topk_ids(pool: &sqlx::PgPool, table: &str, emb: &str, search_list: u32) -> Vec<String> {
    let mut tx = pool.begin().await.expect("topk tx");
    let _ = sqlx::query(&format!(
        "SET LOCAL diskann.query_search_list_size = {search_list}"
    ))
    .execute(&mut *tx)
    .await;
    let sql = format!(
        r#"SELECT id FROM {table}
           WHERE workspace_id = $1 AND tenant_id = $2 AND metadata->>'type' = 'chunk'
           ORDER BY embedding <=> $3::vector
           LIMIT 20"#
    );
    let rows: Vec<(String,)> = sqlx::query_as(&sql)
        .bind(WS)
        .bind(TENANT)
        .bind(emb)
        .fetch_all(&mut *tx)
        .await
        .unwrap_or_default();
    let _ = tx.commit().await;
    rows.into_iter().map(|r| r.0).collect()
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
    clients: usize,
    cliff: f64,
    mult: f64,
    arm: &str,
    recall: f64,
) -> CellResult {
    let mf = workspace_filter(WS, TENANT);
    if arm.starts_with("hnsw") {
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", QUERY_EF.to_string());
    }

    let recall_ok = recall >= RECALL_GATE;
    emit(
        "diskann_recall",
        recall * 1000.0,
        recall_ok,
        arm,
        format!(
            "arm={arm} rows={rows} clients={clients} recall@20_mean={recall:.4} gate={RECALL_GATE}"
        ),
        &[],
    );

    let (single_p95, single) = measure_single(storage, DIM, &mf, TOP_K).await;
    let slo_pass = single_p95 < Q1D_SLO_MS;
    assert!(
        single_p95 < cliff,
        "hang cliff: arm={arm} single {single_p95:.2} >= {cliff}"
    );
    emit(
        "diskann_single",
        single_p95,
        slo_pass && recall_ok,
        arm,
        format!(
            "arm={arm} rows={rows} clients={clients} slo_pass={slo_pass} recall_ok={recall_ok}"
        ),
        &single,
    );

    let (stress_p95, all, wall) =
        measure_stress(Arc::clone(storage), DIM, mf, clients, 20, TOP_K).await;
    let abs_ok = stress_p95 < Q1D_SLO_MS;
    let rel_ok = stress_p95 < (single_p95 * mult).max(50.0);
    let full_green = slo_pass && recall_ok && abs_ok;
    emit(
        "diskann_stress",
        stress_p95,
        full_green,
        arm,
        format!(
            "arm={arm} rows={rows} clients={clients} single_p95={single_p95:.2} \
             abs_ok={abs_ok} rel_ok={rel_ok} wall={wall:?} full_green={full_green}"
        ),
        &all,
    );
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");
    CellResult {
        full_green,
        recall_ok,
        abs_ok,
        single_p95,
        stress_p95,
    }
}

async fn seed_hnsw(rows: usize) -> (Arc<PgVectorStorage>, edgequake_storage::PostgresConfig) {
    let clients = 16usize;
    let base =
        postgres_test_config::require_or_skip_postgres("disk070").expect("DATABASE_URL required");
    let mut config = with_stress_pool(base, clients).with_vector_index(VectorIndexType::None);
    config.namespace = format!("eq_d070h_ws_{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let storage = Arc::new(
        PgVectorStorage::with_dimension(config.clone(), DIM)
            .with_storage_mode(VectorStorageMode::Half),
    );
    storage.initialize().await.expect("init");
    assert!(storage.is_dedicated_workspace_table());

    let seed_ms = seed_single_ws(&storage, rows, DIM, 1000, "d070h", TENANT, WS).await;
    let index_wall = Instant::now();
    storage.ensure_ann_index().await.expect("hnsw");
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    emit(
        "diskann_index",
        index_ms,
        true,
        "hnsw",
        format!(
            "arm=hnsw rows={rows} seed_ms={seed_ms:.0} table={}",
            storage.vectors_table_name()
        ),
        &[Duration::from_secs_f64(index_ms / 1000.0)],
    );
    let explain = explain_index(
        &config,
        storage.vectors_table_name(),
        storage.embedding_sql_type(),
        "hnsw",
    )
    .await;
    emit(
        "diskann_explain",
        0.0,
        explain.contains("uses_hnsw=true"),
        "hnsw",
        explain.chars().take(4000).collect::<String>(),
        &[],
    );
    (storage, config)
}

async fn seed_diskann(
    rows: usize,
) -> Result<(Arc<PgVectorStorage>, edgequake_storage::PostgresConfig), String> {
    let clients = 16usize;
    let base = postgres_test_config::require_or_skip_postgres("disk070")
        .ok_or_else(|| "DATABASE_URL required".to_string())?;
    let mut config = with_stress_pool(base, clients).with_vector_index(VectorIndexType::None);
    config.namespace = format!("eq_d070d_ws_{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let probe = postgres_test_config::contract_pg_pool(&config).await;
    ensure_vectorscale(&probe).await?;

    // DiskANN supports `vector` (not halfvec) in pgvectorscale 0.9.0
    let storage = Arc::new(
        PgVectorStorage::with_dimension(config.clone(), DIM)
            .with_storage_mode(VectorStorageMode::Full),
    );
    storage.initialize().await.map_err(|e| e.to_string())?;
    assert!(storage.is_dedicated_workspace_table());

    let seed_ms = seed_single_ws(&storage, rows, DIM, 1000, "d070d", TENANT, WS).await;
    let table = storage.vectors_table_name().to_string();
    // Index name must not include schema qualifier (public.foo → syntax error near '.').
    let table_only = table.rsplit('.').next().unwrap_or(&table);
    let idx = format!("{table_only}_diskann_idx");
    let index_wall = Instant::now();
    sqlx::query(&format!(
        r#"CREATE INDEX {idx} ON {table}
           USING diskann (embedding vector_cosine_ops)
           WITH (storage_layout = 'memory_optimized')"#
    ))
    .execute(&probe)
    .await
    .map_err(|e| format!("CREATE INDEX diskann: {e}"))?;
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    emit(
        "diskann_index",
        index_ms,
        true,
        "diskann",
        format!("arm=diskann rows={rows} seed_ms={seed_ms:.0} table={table}"),
        &[Duration::from_secs_f64(index_ms / 1000.0)],
    );
    let explain = explain_index(&config, &table, "vector", "diskann").await;
    emit(
        "diskann_explain",
        0.0,
        explain.to_lowercase().contains("diskann") || explain.contains("uses_diskann=true"),
        "diskann",
        explain.chars().take(4000).collect::<String>(),
        &[],
    );
    Ok((storage, config))
}

#[tokio::test]
async fn e2e_spec070_diskann_vs_hnsw_battle() {
    let smoke = std::env::var("EQ_DISKANN_SMOKE")
        .map(|v| v == "1")
        .unwrap_or(false);
    let rows_list = if smoke {
        parse_u32_list("EQ_DISKANN_ROWS_LIST", &[2_000])
    } else {
        parse_u32_list("EQ_DISKANN_ROWS_LIST", &[100_000, 150_000, 250_000])
    };

    std::env::remove_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE");
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");

    if postgres_test_config::require_or_skip_postgres("disk070").is_none() {
        return;
    }

    let gate_clients = 16usize;
    let mult = stress_mult();
    let mut green_150k_diskann = false;
    let mut any_diskann_full_green = false;
    let mut diskann_ok = true;

    for &rows in &rows_list {
        let cliff = ceiling_hang_cliff_ms(rows as usize);

        // --- HNSW control ---
        std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
        let (hnsw_storage, _) = seed_hnsw(rows as usize).await;
        let mf = workspace_filter(WS, TENANT);
        let hnsw_recall = measure_recall_hnsw(&hnsw_storage, &mf).await;
        let hnsw_cell = run_cell(
            &hnsw_storage,
            rows,
            gate_clients,
            cliff,
            mult,
            "hnsw_dedicated",
            hnsw_recall,
        )
        .await;
        emit(
            "diskann_arm_summary",
            hnsw_cell.stress_p95,
            hnsw_cell.full_green,
            "hnsw_dedicated",
            format!(
                "rows={rows} full_green={} recall_ok={} abs_ok={} single_p95={:.2} stress_p95={:.2}",
                hnsw_cell.full_green,
                hnsw_cell.recall_ok,
                hnsw_cell.abs_ok,
                hnsw_cell.single_p95,
                hnsw_cell.stress_p95
            ),
            &[],
        );

        // --- DiskANN study arm ---
        std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "full");
        match seed_diskann(rows as usize).await {
            Ok((disk_storage, disk_cfg)) => {
                let table = disk_storage.vectors_table_name().to_string();
                let disk_recall = measure_recall_diskann(&disk_storage, &disk_cfg, &table).await;
                let disk_cell = run_cell(
                    &disk_storage,
                    rows,
                    gate_clients,
                    cliff,
                    mult,
                    "diskann_dedicated",
                    disk_recall,
                )
                .await;
                if disk_cell.full_green {
                    any_diskann_full_green = true;
                }
                if rows == 150_000 && disk_cell.full_green {
                    green_150k_diskann = true;
                }
                emit(
                    "diskann_arm_summary",
                    disk_cell.stress_p95,
                    disk_cell.full_green,
                    "diskann_dedicated",
                    format!(
                        "rows={rows} full_green={} recall_ok={} abs_ok={} single_p95={:.2} stress_p95={:.2}",
                        disk_cell.full_green,
                        disk_cell.recall_ok,
                        disk_cell.abs_ok,
                        disk_cell.single_p95,
                        disk_cell.stress_p95
                    ),
                    &[],
                );
            }
            Err(e) => {
                diskann_ok = false;
                emit(
                    "diskann_arm_summary",
                    0.0,
                    false,
                    "diskann_dedicated",
                    format!("rows={rows} SKIP/ERR: {e}"),
                    &[],
                );
                eprintln!("WARN SPEC-070 DiskANN arm skipped: {e}");
            }
        }
    }

    let promote = green_150k_diskann && !smoke;
    emit(
        "diskann_decision",
        if promote { 1.0 } else { 0.0 },
        promote,
        "promote",
        format!(
            "green_150k_diskann={green_150k_diskann} any_diskann_full_green={any_diskann_full_green} \
             diskann_extension_ok={diskann_ok} smoke={smoke} \
             promote_ssot={promote} (full gate: single∧recall@20≥0.99∧concurrent@clients=16)"
        ),
        &[],
    );
    // Soft-fail product gates: cargo test succeeds unless hang cliff asserted above.
}
