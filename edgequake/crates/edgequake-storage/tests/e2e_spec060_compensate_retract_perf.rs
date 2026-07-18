//! SPEC-060 — compensate / retract K=1k wall + shared-neighbor preservation.
//!
//! ```bash
//! export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
//! export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
//! cargo test -p edgequake-storage --features postgres --test e2e_spec060_compensate_retract_perf -- --nocapture
//! ```

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::compensation;
use edgequake_storage::traits::{
    GraphStorage, GraphStorageMutateOps, GraphStorageReadOps, VectorStorage,
};
use edgequake_storage::{PgVectorStorage, PostgresAGEGraphStorage};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const DIM: usize = 32;
const K: usize = 1_000;

fn emb(i: usize) -> Vec<f32> {
    (0..DIM).map(|d| ((d + i) as f32 * 0.021).sin()).collect()
}

#[tokio::test]
async fn e2e_spec060_compensate_retract_k1k_under_500ms() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf060_comp") else {
        return;
    };

    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let vectors = PgVectorStorage::with_dimension(config.clone(), DIM);
    vectors.initialize().await.expect("vector init");
    let graph = PostgresAGEGraphStorage::new(config.clone());
    graph.initialize().await.expect("graph init");

    // Shared neighbor that must survive compensate of doc-A orphans.
    let mut shared_props = HashMap::new();
    shared_props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
    shared_props.insert(
        "source_ids".to_string(),
        serde_json::json!(["doc-a-chunk-0", "doc-b-chunk-0"]),
    );
    shared_props.insert("workspace_id".to_string(), serde_json::json!("ws-comp060"));
    graph
        .upsert_nodes_batch(&[("SHARED_NEIGHBOR".to_string(), shared_props)])
        .await
        .expect("shared node");

    let vec_ids: Vec<String> = (0..K).map(|i| format!("comp060-vec-{i}")).collect();
    let vec_batch: Vec<_> = vec_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            (
                id.clone(),
                emb(i),
                serde_json::json!({
                    "type": "entity",
                    "document_id": "doc-a",
                    "workspace_id": "ws-comp060",
                }),
            )
        })
        .collect();
    vectors.upsert(&vec_batch).await.expect("seed vectors");

    let node_ids: Vec<String> = (0..K).map(|i| format!("COMP_ORPHAN_{i}")).collect();
    let nodes: Vec<_> = node_ids
        .iter()
        .map(|id| {
            let mut props = HashMap::new();
            props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
            props.insert(
                "source_ids".to_string(),
                serde_json::json!(["doc-a-chunk-0"]),
            );
            props.insert("workspace_id".to_string(), serde_json::json!("ws-comp060"));
            (id.clone(), props)
        })
        .collect();
    for batch in nodes.chunks(250) {
        graph.upsert_nodes_batch(batch).await.expect("seed nodes");
    }

    let start = Instant::now();
    compensation::compensate_orphan_vectors(
        &vectors,
        "doc-a",
        &vec_ids,
        &[],
        "spec060 compensate gate",
    )
    .await;
    compensation::compensate_orphan_graph_writes(
        &graph,
        "doc-a",
        &node_ids,
        &[],
        "spec060 compensate gate",
    )
    .await;
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(500),
        "C-RET FAIL: compensate K={K} took {elapsed:?} (budget 500ms)"
    );
    eprintln!("OK C-RET: compensate K={K} in {elapsed:?}");
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "compensate_retract_k1k",
            "p95_ms": elapsed.as_secs_f64() * 1000.0,
            "samples_ms": [elapsed.as_secs_f64() * 1000.0],
            "plan_class": "native_delete",
            "pass": true,
            "detail": format!("K={K}"),
        })
    );

    // Shared neighbor preserved
    let shared = graph
        .get_nodes_batch(&["SHARED_NEIGHBOR".to_string()])
        .await
        .expect("get shared");
    assert_eq!(
        shared.len(),
        1,
        "shared neighbor must survive compensate of doc-a orphans"
    );

    // Orphan vectors gone
    let remaining = vectors
        .get_by_ids(&vec_ids[..10])
        .await
        .expect("probe deleted");
    assert!(
        remaining.is_empty(),
        "compensated vector ids must be deleted; got {}",
        remaining.len()
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = vectors.clear().await;
    let _ = graph.clear().await;
}
