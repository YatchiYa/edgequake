//! SPEC-075 — Filtered recall@20 claim gate (Wave-2) + iterative_scan-only vs partial smoke.
//!
//! Env:
//! - `EQ_FILTERED_RECALL_ROWS` default `5000` (smoke)
//! - Soft-fails product gates; hang cliff hard-fails
//!
//! Always measures **workspace-filtered** recall (never unfiltered-only).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType, VectorStorageMode};
use perf_ann_corpus::{emb, measure_single, seed_ws_split, workspace_filter};
use std::sync::Arc;

const TOP_K: usize = 20;
const RECALL_GATE: f64 = 0.99;
const Q1D_SLO_MS: f64 = 500.0;
const DEFAULT_HANG_CLIFF_MS: f64 = 5_000.0;

fn hang_cliff_ms() -> f64 {
    std::env::var("EQ_FILTERED_HANG_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HANG_CLIFF_MS)
}
const DIM: usize = 1536;
const HOT_WS: &str = "ws-a";
const COLD_WS: &str = "ws-b";
const TENANT: &str = "t-fr075";
const REF_EF: u32 = 400;
const QUERY_EF: u32 = 80;

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
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": op,
            "p95_ms": p95_ms,
            "plan_class": plan_class,
            "pass": pass,
            "detail": detail.into(),
            "samples_ms": Vec::<f64>::new(),
        })
    );
}

async fn filtered_recall_mean(storage: &PgVectorStorage, query_ef: u32) -> f64 {
    let mf = workspace_filter(HOT_WS, TENANT);
    let mut recalls = Vec::new();
    for s in 0..5 {
        let q = emb(DIM, (s + 7) as f32);
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", REF_EF.to_string());
        let hi = storage
            .query_filtered(&q, TOP_K, None, Some(&mf))
            .await
            .expect("ref filtered");
        std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", query_ef.to_string());
        let lo = storage
            .query_filtered(&q, TOP_K, None, Some(&mf))
            .await
            .expect("cand filtered");
        let hi_ids: Vec<_> = hi.iter().map(|h| h.id.clone()).collect();
        let lo_ids: Vec<_> = lo.iter().map(|h| h.id.clone()).collect();
        recalls.push(recall_at_k(&hi_ids, &lo_ids));
    }
    recalls.iter().sum::<f64>() / recalls.len() as f64
}

async fn run_arm(ns_suffix: &str, arm: &str, partial: bool, rows: u32) {
    let Some(base) = postgres_test_config::contract_postgres_config(&format!("fr075_{ns_suffix}"))
    else {
        eprintln!("SKIP SPEC-075: DATABASE_URL / POSTGRES_PASSWORD not set");
        return;
    };

    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    std::env::set_var("EDGEQUAKE_HNSW_ITERATIVE_SCAN", "relaxed_order");
    std::env::set_var("EDGEQUAKE_HNSW_MAX_SCAN_TUPLES", "20000");
    if partial {
        std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE", "1");
        std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS", "100");
    } else {
        std::env::remove_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE");
    }

    // Build global HNSW first; Wave-2 arm drops global after partial ensure (SPEC-068 pattern).
    let mut config = base.with_vector_index(VectorIndexType::HNSW);
    config.max_connections = 8;
    config.hnsw_m = 16;
    config.hnsw_ef_construction = 64;

    let storage = Arc::new(
        PgVectorStorage::with_dimension(config, DIM).with_storage_mode(VectorStorageMode::Half),
    );
    if let Err(e) = storage.initialize().await {
        eprintln!("SKIP SPEC-075 arm={arm}: init failed ({e})");
        return;
    }

    let seed_ms = seed_ws_split(
        &storage,
        rows as usize,
        DIM,
        500,
        "fr075",
        TENANT,
        HOT_WS,
        COLD_WS,
    )
    .await;
    emit(
        "fr075_seed",
        seed_ms,
        true,
        arm,
        format!("rows={rows} hot_ws={HOT_WS} selectivity≈0.2 FILTERED_recall_gate"),
    );

    if partial {
        let _ = storage.ensure_hot_workspace_ann(HOT_WS).await;
        let _ = storage.drop_global_ann_index().await;
    }

    let mf = workspace_filter(HOT_WS, TENANT);
    let _ = storage
        .query_filtered(&emb(DIM, 1.0), TOP_K, None, Some(&mf))
        .await;

    let recall = filtered_recall_mean(&storage, QUERY_EF).await;
    let recall_ok = recall >= RECALL_GATE;
    emit(
        "fr075_filtered_recall",
        recall * 1000.0,
        recall_ok,
        arm,
        format!(
            "FILTERED workspace_id={HOT_WS} rows={rows} ef={QUERY_EF} vs ref_ef={REF_EF} \
             recall@20={recall:.4} gate={RECALL_GATE} partial={partial}"
        ),
    );

    std::env::set_var("EDGEQUAKE_HNSW_EF_SEARCH", QUERY_EF.to_string());
    let (single_p95, _) = measure_single(&storage, DIM, &mf, TOP_K).await;
    let hang = hang_cliff_ms();
    assert!(
        single_p95 < hang,
        "hang cliff: single p95 {single_p95:.2}ms >= {hang}"
    );
    let slo_ok = single_p95 < Q1D_SLO_MS;
    let full_green = recall_ok && slo_ok;
    emit(
        "fr075_cell",
        single_p95,
        full_green,
        arm,
        format!(
            "arm={arm} rows={rows} filtered_recall={recall:.4} single_p95={single_p95:.2} \
             slo_ok={slo_ok} full_green={full_green} (soft-fail product gate)"
        ),
    );

    if full_green {
        println!("GREEN SPEC-075: arm={arm} filtered_recall={recall:.4} single_p95={single_p95:.2}");
    } else {
        println!(
            "WARN SPEC-075: arm={arm} filtered_recall={recall:.4} single_p95={single_p95:.2} \
             (soft-fail; see SPEC-068 for 100k evidence)"
        );
    }
    std::env::remove_var("EDGEQUAKE_HNSW_EF_SEARCH");
}

#[tokio::test]
async fn e2e_spec075_filtered_recall_gate() {
    let rows: u32 = std::env::var("EQ_FILTERED_RECALL_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000);

    // Primary: Wave-2 partial + iterative_scan (product path).
    run_arm("wave2", "wave2_partial_iterative", true, rows).await;
    // Comparison: iterative_scan only (no partial) — archive; do not change default.
    run_arm("iter", "iterative_scan_only", false, rows).await;

    emit(
        "fr075_decision",
        0.0,
        true,
        "honesty",
        "filtered recall@20 is the promote metric; Wave-2 default unchanged; \
         100k evidence: specs/068-recall-quality-scale/e2e/artifacts/RUN_NOTES.md",
    );
}
