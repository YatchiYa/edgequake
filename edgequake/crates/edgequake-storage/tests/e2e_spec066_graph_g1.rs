//! SPEC-066 — Graph G1: 100k nodes store + degrees sample p95 < 100ms.
#![cfg(feature = "postgres")]

#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
use edgequake_storage::PostgresAGEGraphStorage;
use perf_harness::{finish_report, samples_after_warmup};
use std::collections::HashMap;
use std::time::Instant;

const NODE_N: usize = 100_000;
const DEGREE_SAMPLE: usize = 1_000;
const SAMPLES: usize = 20;
const DEGREE_SLO_MS: f64 = 100.0;

#[tokio::test]
async fn e2e_spec066_graph_g1_100k_nodes_degrees() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("ceil066_g1") else {
        return;
    };
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");

    let seed_wall = Instant::now();
    let mut nodes = Vec::with_capacity(1000);
    for i in 0..NODE_N {
        let mut p = HashMap::new();
        p.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
        nodes.push((format!("G1_{i}"), p));
        if nodes.len() >= 1000 {
            graph.upsert_nodes_batch(&nodes).await.expect("nodes");
            nodes.clear();
        }
    }
    if !nodes.is_empty() {
        graph.upsert_nodes_batch(&nodes).await.expect("nodes");
    }

    // Sparse edges: every 10th node is a hub for the next 9.
    let mut edges = Vec::with_capacity(1000);
    for i in 0..NODE_N {
        if i % 10 == 0 {
            continue;
        }
        let hub = format!("G1_{}", (i / 10) * 10);
        let mut p = HashMap::new();
        p.insert("relation_type".to_string(), serde_json::json!("RELATED"));
        edges.push((hub, format!("G1_{i}"), p));
        if edges.len() >= 1000 {
            graph.upsert_edges_batch(&edges).await.expect("edges");
            edges.clear();
        }
    }
    if !edges.is_empty() {
        graph.upsert_edges_batch(&edges).await.expect("edges");
    }
    let seed_ms = seed_wall.elapsed().as_secs_f64() * 1000.0;

    let ids: Vec<String> = (0..DEGREE_SAMPLE).map(|i| format!("G1_{i}")).collect();
    let _ = graph.node_degrees_batch(&ids).await.expect("warm");

    let mut samples = Vec::new();
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let deg = graph.node_degrees_batch(&ids).await.expect("degrees");
        samples.push(start.elapsed());
        assert_eq!(deg.len(), DEGREE_SAMPLE);
    }
    let hygiene = samples_after_warmup(&samples, 5);
    finish_report(
        "ceiling_graph_g1_degrees",
        &hygiene,
        DEGREE_SLO_MS,
        "eq_source_id_btree",
        false,
        format!("nodes={NODE_N} sample={DEGREE_SAMPLE} seed_ms={seed_ms:.0}"),
    );

    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "ceiling_graph_g1_seed",
            "p95_ms": seed_ms,
            "pass": true,
            "plan_class": "native_upsert",
            "detail": format!("nodes={NODE_N} edges≈{}", NODE_N * 9 / 10),
            "samples_ms": [seed_ms],
        })
    );
}
