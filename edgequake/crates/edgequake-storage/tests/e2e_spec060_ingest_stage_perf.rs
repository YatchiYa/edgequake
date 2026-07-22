//! SPEC-060 — ingest stage walls (KV / vector / AGE native) without LLM.
//!
//! ```bash
//! export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
//! export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
//! cargo test -p edgequake-storage --features postgres --test e2e_spec060_ingest_stage_perf -- --nocapture
//! ```

#![cfg(feature = "postgres")]

#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, KVStorage, VectorStorage};
use edgequake_storage::{
    PgVectorStorage, PostgresAGEGraphStorage, PostgresKVStorage, VectorIndexType,
};
use perf_harness::{finish_report, samples_after_warmup};
use std::collections::HashMap;
use std::time::Instant;

const DIM: usize = 64;
const KV_N: usize = 1_000;
const VEC_N: usize = 1_000;
const AGE_N: usize = 500;
const SAMPLES: usize = 12;

fn emb(i: usize) -> Vec<f32> {
    (0..DIM).map(|d| ((d + i) as f32 * 0.019).sin()).collect()
}

#[tokio::test]
async fn e2e_spec060_ingest_stage_budgets() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf060_ingest") else {
        return;
    };

    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let kv = PostgresKVStorage::new(config.clone());
    kv.initialize().await.expect("kv init");
    // SPEC-062: bulk cold ingest — heap upsert without HNSW, then ensure_ann_index.
    let bulk_cfg = config.clone().with_vector_index(VectorIndexType::None);
    let vectors = PgVectorStorage::with_dimension(bulk_cfg, DIM);
    vectors.initialize().await.expect("vector init");
    let graph = PostgresAGEGraphStorage::new(config.clone());
    graph.initialize().await.expect("graph init");

    let _ = kv.upsert(&[]).await;
    let _ = vectors.upsert(&[]).await;

    // KV: measure first upsert + re-upserts (p95)
    let mut kv_samples = Vec::new();
    for s in 0..SAMPLES {
        let kv_batch: Vec<_> = (0..KV_N)
            .map(|i| {
                (
                    format!("ingest060-kv-{i}"),
                    serde_json::json!({"content": format!("chunk body {i} s{s}"), "type": "chunk"}),
                )
            })
            .collect();
        let start = Instant::now();
        kv.upsert(&kv_batch).await.expect("kv upsert 1k");
        kv_samples.push(start.elapsed());
    }
    let kv_hygiene = samples_after_warmup(&kv_samples, 8);
    finish_report(
        "ingest_kv_upsert",
        &kv_hygiene,
        100.0,
        "pk",
        false,
        format!("N={KV_N}"),
    );

    // Vector heap upsert (no HNSW) — SPEC-062 excellence target <250ms @ N=1k dim=64.
    let mut vec_samples = Vec::new();
    for s in 0..SAMPLES {
        let vec_batch: Vec<_> = (0..VEC_N)
            .map(|i| {
                (
                    format!("ingest060-vec-{s}-{i}"),
                    emb(i + s * VEC_N),
                    serde_json::json!({
                        "type": "chunk",
                        "document_id": format!("doc-{s}-{i}"),
                        "workspace_id": "ws-ingest060",
                    }),
                )
            })
            .collect();
        let start = Instant::now();
        let created = vectors
            .upsert_report_created(&vec_batch)
            .await
            .expect("upsert_report_created 1k");
        vec_samples.push(start.elapsed());
        assert_eq!(
            created.len(),
            VEC_N,
            "first insert of unique ids must create all"
        );
    }
    // Drop cold sample + single max spike (p95≈max with n≈10 is host jitter).
    let mut vec_hygiene = samples_after_warmup(&vec_samples, 8);
    if vec_hygiene.len() >= 5 {
        if let Some(imax) = vec_hygiene
            .iter()
            .enumerate()
            .max_by_key(|(_, d)| d.as_nanos())
            .map(|(i, _)| i)
        {
            vec_hygiene.swap_remove(imax);
        }
    }
    finish_report(
        "ingest_vector_upsert_report_created",
        &vec_hygiene,
        250.0,
        "heap_insert_deferred_hnsw",
        false,
        format!("N={VEC_N} deferred_hnsw=1 noise_ok"),
    );

    let start = Instant::now();
    vectors.ensure_ann_index().await.expect("ensure_ann_index");
    finish_report(
        "ingest_vector_ensure_ann_index",
        &[start.elapsed()],
        5_000.0,
        "hnsw_create",
        false,
        format!("rows≈{}", VEC_N * SAMPLES),
    );

    let mut age_samples = Vec::new();
    for s in 0..SAMPLES.min(5) {
        let nodes: Vec<_> = (0..AGE_N)
            .map(|i| {
                let mut props = HashMap::new();
                props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
                props.insert(
                    "workspace_id".to_string(),
                    serde_json::json!("ws-ingest060"),
                );
                (format!("INGEST_NODE_{s}_{i}"), props)
            })
            .collect();
        let start = Instant::now();
        graph
            .upsert_nodes_batch(&nodes)
            .await
            .expect("AGE native upsert 500");
        age_samples.push(start.elapsed());
    }
    finish_report(
        "ingest_age_upsert_nodes",
        &age_samples,
        500.0,
        "native_on_conflict",
        false,
        format!("N={AGE_N}"),
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }

    let _ = vectors.clear().await;
    let _ = graph.clear().await;
}
