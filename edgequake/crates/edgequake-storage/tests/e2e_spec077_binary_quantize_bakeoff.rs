//! SPEC-077 — Wave-2 halfvec vs binary_quantize+rerank filtered recall bake-off.
//!
//! Soft-skip without DB. Hang cliff hard-fails. Does not raise floors.
#![cfg(feature = "postgres")]

#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{
    build_binary_hnsw_index_sql, build_binary_rerank_select_sql, BinaryQuantizePolicy,
    PgVectorStorage, VectorIndexType, VectorStorageMode,
};
use perf_ann_corpus::{emb, seed_ws_split, workspace_filter};
use sqlx::Row;
use std::time::Instant;

const DIM: usize = 64;
const DEFAULT_ROWS: u32 = 2_000;
const TOP_K: usize = 20;
const CANDIDATE_K: usize = 200;
const HOT_WS: &str = "ws-a";
const COLD_WS: &str = "ws-b";
const TENANT: &str = "t-bq077";
const DEFAULT_HANG_CLIFF_MS: f64 = 5_000.0;
const RECALL_SOFT: f64 = 0.90;

fn hang_cliff_ms() -> f64 {
    std::env::var("EQ_BQ_HANG_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HANG_CLIFF_MS)
}

fn recall_at_k(reference: &[String], candidate: &[String]) -> f64 {
    if reference.is_empty() {
        return 1.0;
    }
    let set: std::collections::HashSet<&String> = candidate.iter().collect();
    let hit = reference.iter().filter(|id| set.contains(id)).count();
    hit as f64 / reference.len() as f64
}

fn emit(op: &str, p95_ms: f64, pass: bool, plan_class: &str, detail: impl Into<String>) {
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "op": op,
            "p95_ms": p95_ms,
            "plan_class": plan_class,
            "pass": pass,
            "detail": detail.into(),
        })
    );
}

#[tokio::test]
async fn e2e_spec077_binary_quantize_bakeoff() {
    let rows: u32 = std::env::var("EQ_BQ_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_ROWS);

    let Some(base) = postgres_test_config::contract_postgres_config("bq077") else {
        eprintln!("SKIP SPEC-077: DATABASE_URL / POSTGRES_PASSWORD not set");
        return;
    };

    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    std::env::remove_var("EDGEQUAKE_BINARY_QUANTIZE");

    let mut config = base.with_vector_index(VectorIndexType::HNSW);
    config.max_connections = 8;
    config.hnsw_m = 16;
    config.hnsw_ef_construction = 64;

    let storage = PgVectorStorage::with_dimension(config.clone(), DIM)
        .with_storage_mode(VectorStorageMode::Half);
    if let Err(e) = storage.initialize().await {
        eprintln!("SKIP SPEC-077: init failed ({e})");
        return;
    }

    let seed_ms = seed_ws_split(
        &storage,
        rows as usize,
        DIM,
        400,
        "bq077",
        TENANT,
        HOT_WS,
        COLD_WS,
    )
    .await;
    emit(
        "bq077_seed",
        seed_ms,
        true,
        "wave2_halfvec",
        format!("rows={rows} FILTERED bake-off"),
    );

    let table = storage.vectors_table_name().to_string();
    let probe = postgres_test_config::contract_pg_pool(&config).await;

    let idx_sql = build_binary_hnsw_index_sql(&table, "eq_bq077_embedding_bq_idx", DIM, 16, 64);
    if let Err(e) = sqlx::query(&idx_sql).execute(&probe).await {
        eprintln!("SKIP SPEC-077: binary index create failed ({e})");
        return;
    }
    let _ = sqlx::query(&format!("ANALYZE {table}"))
        .execute(&probe)
        .await;

    let mf = workspace_filter(HOT_WS, TENANT);
    let q = emb(DIM, 11.0);

    let t0 = Instant::now();
    let wave2 = storage
        .query_filtered(&q, TOP_K, None, Some(&mf))
        .await
        .expect("wave2 filtered");
    let wave2_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let hang = hang_cliff_ms();
    assert!(wave2_ms < hang, "wave2 hang cliff {wave2_ms} >= {hang}");
    let wave2_ids: Vec<String> = wave2.into_iter().map(|h| h.id).collect();

    let policy = BinaryQuantizePolicy {
        enabled: true,
        candidate_k: CANDIDATE_K,
    };
    let sql = build_binary_rerank_select_sql(
        &table,
        "halfvec",
        DIM,
        "WHERE workspace_id = $2 AND tenant_id = $3",
        4,
        TOP_K,
        &policy,
    );
    let emb_str = format!(
        "[{}]",
        q.iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(",")
    );

    let t1 = Instant::now();
    let bq_rows = sqlx::query(&sql)
        .bind(&emb_str)
        .bind(HOT_WS)
        .bind(TENANT)
        .bind(TOP_K as i32)
        .fetch_all(&probe)
        .await;
    let bq_ms = t1.elapsed().as_secs_f64() * 1000.0;
    assert!(bq_ms < hang, "binary hang cliff {bq_ms} >= {hang}");

    let bq_ids: Vec<String> = match bq_rows {
        Ok(rs) => rs.iter().map(|r| r.get::<String, _>("id")).collect(),
        Err(e) => {
            emit(
                "bq077_cell",
                bq_ms,
                false,
                "binary_rerank",
                format!("query failed: {e}"),
            );
            eprintln!("WARN SPEC-077: binary query failed ({e})");
            return;
        }
    };

    let recall = recall_at_k(&wave2_ids, &bq_ids);
    let pass = recall >= RECALL_SOFT && !bq_ids.is_empty();
    emit(
        "bq077_filtered_recall",
        recall * 1000.0,
        pass,
        "binary_vs_wave2",
        format!(
            "FILTERED recall@20 binary_vs_wave2={recall:.4} soft={RECALL_SOFT} \
             wave2_ms={wave2_ms:.1} bq_ms={bq_ms:.1} candidate_k={CANDIDATE_K} rows={rows}"
        ),
    );
    emit(
        "bq077_cell",
        bq_ms,
        pass,
        "binary_rerank",
        format!(
            "wave2_hits={} bq_hits={} recall={recall:.4} (soft-fail; Wave-2 remains default)",
            wave2_ids.len(),
            bq_ids.len()
        ),
    );
    emit(
        "bq077_decision",
        0.0,
        true,
        "honesty",
        "binary+rerank is opt-in study; Wave-2 default + floors unchanged; \
         promote only after full gate (not this smoke)",
    );

    if pass {
        println!("GREEN SPEC-077: filtered recall binary_vs_wave2={recall:.4}");
    } else {
        println!("WARN SPEC-077: filtered recall={recall:.4} (soft; archive only)");
    }
}
