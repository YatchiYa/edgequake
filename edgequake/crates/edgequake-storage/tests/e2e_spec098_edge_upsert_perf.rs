//! SPEC-098: measure native edge upsert wall time with duplicate-heavy batches.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps};
use edgequake_storage::PostgresAGEGraphStorage;
use std::collections::HashMap;
use std::time::Instant;

const SIZES: &[usize] = &[500, 2_000, 5_000];
/// CI soft budget (ms) for p95 on the largest batch after warmup — warn-level in prod is 800ms
/// for smaller adaptive chunks; 5k unique edges may exceed that; gate keeps a hard upper bound.
const P95_BUDGET_MS_5K: u128 = 8_000;

#[tokio::test]
async fn e2e_spec098_edge_upsert_perf() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e098_perf") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");

    let max_n = *SIZES.last().unwrap();
    let mut nodes = Vec::with_capacity(max_n + 1);
    let mut hub = HashMap::new();
    hub.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    nodes.push(("PERF_HUB".into(), hub));
    for i in 0..max_n {
        let mut p = HashMap::new();
        p.insert("entity_type".into(), serde_json::json!("CONCEPT"));
        nodes.push((format!("PERF_LEAF_{i}"), p));
    }
    for chunk in nodes.chunks(250) {
        graph.upsert_nodes_batch(chunk).await.expect("nodes");
    }

    for &n in SIZES {
        // 2× duplicates of each (src,tgt,rel) to exercise DISTINCT ON + Rust dedupe.
        let mut edges = Vec::with_capacity(n * 2);
        for i in 0..n {
            let mut p1 = HashMap::new();
            p1.insert("relation_type".into(), serde_json::json!("RELATED"));
            p1.insert("description".into(), serde_json::json!("first"));
            let mut p2 = HashMap::new();
            p2.insert("relation_type".into(), serde_json::json!("related"));
            p2.insert("description".into(), serde_json::json!("second-wins"));
            edges.push(("PERF_HUB".into(), format!("PERF_LEAF_{i}"), p1));
            edges.push(("PERF_HUB".into(), format!("PERF_LEAF_{i}"), p2));
        }

        // Warmup
        graph.upsert_edges_batch(&edges).await.expect("warmup");

        let mut samples = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            graph
                .upsert_edges_batch(&edges)
                .await
                .expect("perf upsert");
            samples.push(start.elapsed().as_millis());
        }
        samples.sort_unstable();
        let p50 = samples[samples.len() / 2];
        let p95 = samples[(samples.len() * 95 / 100).min(samples.len() - 1)];
        eprintln!("SPEC-098 edge upsert perf N={n} (2x dups): p50={p50}ms p95={p95}ms samples={samples:?}");

        if n == 5_000 {
            assert!(
                p95 <= P95_BUDGET_MS_5K,
                "p95 {p95}ms exceeds CI budget {P95_BUDGET_MS_5K}ms for N=5000"
            );
        }
    }

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}
