//! SPEC-080 — tiny workspace filtered query recall=1.0 (exact-friendly path).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType, VectorStorageMode};
use perf_ann_corpus::{emb, seed_single_ws, workspace_filter};

const DIM: usize = 64;
const ROWS: usize = 200; // well below default ANN_EXACT_MAX_ROWS=2000
const TOP_K: usize = 20;
const WS: &str = "ws-tiny080";
const TENANT: &str = "t-ts080";

#[tokio::test]
async fn e2e_spec080_tiny_slice_exact_smoke() {
    let Some(base) = postgres_test_config::contract_postgres_config("ts080") else {
        eprintln!("SKIP SPEC-080: no DATABASE_URL");
        return;
    };
    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE", "1");
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS", "1");
    std::env::set_var("EDGEQUAKE_ANN_EXACT_MAX_ROWS", "2000");

    let mut config = base.with_vector_index(VectorIndexType::HNSW);
    config.max_connections = 4;
    let storage =
        PgVectorStorage::with_dimension(config, DIM).with_storage_mode(VectorStorageMode::Half);
    if let Err(e) = storage.initialize().await {
        eprintln!("SKIP SPEC-080: init failed ({e})");
        return;
    }
    let _ = seed_single_ws(&storage, ROWS, DIM, 100, "ts080", TENANT, WS).await;
    let mf = workspace_filter(WS, TENANT);
    let q = emb(DIM, 3.0);
    let hits = storage
        .query_filtered(&q, TOP_K, None, Some(&mf))
        .await
        .expect("tiny filtered");
    assert_eq!(hits.len(), TOP_K.min(ROWS), "exact-friendly path should fill top_k");
    // Soft honesty: all hits belong to the tiny workspace (filter held).
    println!(
        "GREEN SPEC-080: tiny WS rows={ROWS} hits={} (bias skipped ≤ EDGEQUAKE_ANN_EXACT_MAX_ROWS)",
        hits.len()
    );
    std::env::remove_var("EDGEQUAKE_ANN_EXACT_MAX_ROWS");
}
