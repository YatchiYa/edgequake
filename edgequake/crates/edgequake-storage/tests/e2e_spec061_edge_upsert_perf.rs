//! SPEC-061 — native `upsert_edges_batch` wall.
#![cfg(feature = "postgres")]

#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps};
use edgequake_storage::PostgresAGEGraphStorage;
use perf_harness::{finish_report, samples_after_warmup};
use std::collections::HashMap;
use std::time::Instant;

const EDGE_N: usize = 1_000;
const SAMPLES: usize = 32;

#[tokio::test]
async fn e2e_spec061_native_edge_upsert_1k() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf061_edge") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");

    let mut nodes = Vec::with_capacity(EDGE_N + 1);
    let mut props_hub = HashMap::new();
    props_hub.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
    nodes.push(("EDGE_HUB".to_string(), props_hub));
    for i in 0..EDGE_N {
        let mut p = HashMap::new();
        p.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
        nodes.push((format!("EDGE_LEAF_{i}"), p));
    }
    for chunk in nodes.chunks(250) {
        graph.upsert_nodes_batch(chunk).await.expect("nodes");
    }

    let edges: Vec<_> = (0..EDGE_N)
        .map(|i| {
            let mut p = HashMap::new();
            p.insert("relation_type".to_string(), serde_json::json!("RELATED"));
            ("EDGE_HUB".to_string(), format!("EDGE_LEAF_{i}"), p)
        })
        .collect();

    let mut samples = Vec::new();
    for s in 0..SAMPLES {
        // Slightly vary targets so ON CONFLICT updates dominate after first
        let batch: Vec<_> = edges
            .iter()
            .map(|(a, b, p)| {
                let mut props = p.clone();
                props.insert("sample".to_string(), serde_json::json!(s));
                (a.clone(), b.clone(), props)
            })
            .collect();
        let start = Instant::now();
        graph.upsert_edges_batch(&batch).await.expect("edges");
        samples.push(start.elapsed());
    }
    let hygiene = samples_after_warmup(&samples, 20);
    finish_report(
        "graph_upsert_edges_batch",
        &hygiene,
        500.0,
        "native_eq_id_on_conflict",
        false,
        format!("N={EDGE_N}"),
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}
