//! SPEC-076 A3 — opt-in exact reorder smoke (soft-skip without DB).
//!
//! Verifies filtered query succeeds with reorder on/off at small N.
//! Does not raise floors; hang cliff hard-fails.
#![cfg(feature = "postgres")]

#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType, VectorStorageMode};
use perf_ann_corpus::{emb, seed_ws_split, workspace_filter};
use std::time::Instant;

const DIM: usize = 64;
const ROWS: usize = 400;
const TOP_K: usize = 10;
const HOT_WS: &str = "ws-a";
const COLD_WS: &str = "ws-b";
const TENANT: &str = "t-pr076";
const HANG_CLIFF_MS: f64 = 5_000.0;

async fn run_arm(label: &str, reorder: bool) {
    let Some(base) = postgres_test_config::contract_postgres_config(&format!("pr076_{label}"))
    else {
        eprintln!("SKIP SPEC-076: DATABASE_URL / POSTGRES_PASSWORD not set");
        return;
    };

    if reorder {
        std::env::set_var("EDGEQUAKE_ANN_EXACT_REORDER", "1");
        std::env::set_var("EDGEQUAKE_ANN_REORDER_CANDIDATE_K", "40");
    } else {
        std::env::remove_var("EDGEQUAKE_ANN_EXACT_REORDER");
        std::env::remove_var("EDGEQUAKE_ANN_REORDER_CANDIDATE_K");
    }

    let mut config = base.with_vector_index(VectorIndexType::HNSW);
    config.max_connections = 4;
    config.hnsw_m = 8;
    config.hnsw_ef_construction = 32;

    let storage =
        PgVectorStorage::with_dimension(config, DIM).with_storage_mode(VectorStorageMode::Half);
    if let Err(e) = storage.initialize().await {
        eprintln!("SKIP SPEC-076 arm={label}: init failed ({e})");
        return;
    }

    let _ = seed_ws_split(&storage, ROWS, DIM, 100, "pr076", TENANT, HOT_WS, COLD_WS).await;

    let mf = workspace_filter(HOT_WS, TENANT);
    let q = emb(DIM, 3.0);
    let start = Instant::now();
    let hits = storage
        .query_filtered(&q, TOP_K, None, Some(&mf))
        .await
        .expect("filtered query");
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    assert!(ms < HANG_CLIFF_MS, "hang cliff: arm={label} took {ms:.1}ms");
    assert_eq!(
        hits.len(),
        TOP_K.min(ROWS / 5),
        "arm={label} expected up to {TOP_K} filtered hits, got {}",
        hits.len()
    );
    println!(
        "GREEN SPEC-076: arm={label} reorder={reorder} hits={} p95≈{ms:.1}ms",
        hits.len()
    );

    std::env::remove_var("EDGEQUAKE_ANN_EXACT_REORDER");
    std::env::remove_var("EDGEQUAKE_ANN_REORDER_CANDIDATE_K");
}

#[tokio::test]
async fn e2e_spec076_exact_reorder_smoke() {
    run_arm("off", false).await;
    run_arm("on", true).await;
}
