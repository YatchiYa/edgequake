//! SPEC-061/062 — concurrent AGE expand stress.
//!
//! pg16: N=8 ≤2×; pg17/18: N=16 ≤1.5× (vs single-client p95).
//! Scale via `EDGEQUAKE_PERF_SCALE=prod|large`.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_stress.rs"]
mod perf_stress;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
use edgequake_storage::PostgresAGEGraphStorage;
use perf_harness::{finish_report, percentile_p95_ms};
use perf_stress::{
    expand_scale, perf_scale, stress_clients, stress_mult, stress_pool_max, with_stress_pool,
    PerfScale,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

const WS: &str = "ws-stress-exp";
const TENANT: &str = "t-stress-exp";

#[tokio::test]
async fn e2e_spec061_stress_concurrent_expand() {
    let scale = perf_scale();
    let exp = expand_scale(scale);
    let clients = stress_clients();
    let mult = stress_mult();
    let pool_n = stress_pool_max(clients);
    let hubs = exp.hubs;
    let leaves = exp.leaves;
    let qpc = exp.queries_per_client;

    let Some(base) = postgres_test_config::require_or_skip_postgres("stress061_exp") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let config = with_stress_pool(base, clients);
    let graph = Arc::new(PostgresAGEGraphStorage::new(config));
    graph.initialize().await.expect("init");

    let mut nodes = Vec::new();
    for i in 0..hubs {
        let mut p = HashMap::new();
        p.insert("workspace_id".to_string(), serde_json::json!(WS));
        p.insert("tenant_id".to_string(), serde_json::json!(TENANT));
        nodes.push((format!("SHUB_{i}"), p));
        for j in 0..leaves {
            let mut lp = HashMap::new();
            lp.insert("workspace_id".to_string(), serde_json::json!(WS));
            lp.insert("tenant_id".to_string(), serde_json::json!(TENANT));
            nodes.push((format!("SLEAF_{i}_{j}"), lp));
        }
    }
    for chunk in nodes.chunks(400) {
        graph.upsert_nodes_batch(chunk).await.expect("nodes");
    }
    let mut edges = Vec::new();
    for i in 0..hubs {
        for j in 0..leaves {
            let mut p = HashMap::new();
            p.insert("relation_type".to_string(), serde_json::json!("RELATED"));
            p.insert("workspace_id".to_string(), serde_json::json!(WS));
            p.insert("tenant_id".to_string(), serde_json::json!(TENANT));
            edges.push((format!("SHUB_{i}"), format!("SLEAF_{i}_{j}"), p));
        }
    }
    for chunk in edges.chunks(400) {
        graph.upsert_edges_batch(chunk).await.expect("edges");
    }

    let mut single = Vec::new();
    for q in 0..15 {
        let ids: Vec<String> = (0..8).map(|k| format!("SHUB_{}", (q + k) % hubs)).collect();
        let start = Instant::now();
        let found = graph
            .get_incident_edges_batch(&ids, Some(TENANT), Some(WS))
            .await
            .expect("expand single");
        single.push(start.elapsed());
        assert!(!found.is_empty());
    }
    let single_p95 = percentile_p95_ms(&single);

    let mut handles = Vec::new();
    for c in 0..clients {
        let graph = Arc::clone(&graph);
        handles.push(tokio::spawn(async move {
            let mut samples = Vec::new();
            for q in 0..qpc {
                let ids: Vec<String> = (0..8)
                    .map(|k| format!("SHUB_{}", (c + q + k) % hubs))
                    .collect();
                let start = Instant::now();
                let found = graph
                    .get_incident_edges_batch(&ids, Some(TENANT), Some(WS))
                    .await
                    .expect("expand");
                samples.push(start.elapsed());
                assert!(!found.is_empty());
            }
            samples
        }));
    }
    let mut all = Vec::new();
    for h in handles {
        all.extend(h.await.expect("join"));
    }
    let floor = match scale {
        PerfScale::Prod | PerfScale::Large => 200.0,
        PerfScale::Default => 100.0,
    };
    let budget = (floor * mult).max(single_p95 * mult);
    finish_report(
        "stress_concurrent_expand",
        &all,
        budget,
        "eq_edge_btree",
        false,
        format!(
            "scale={} clients={clients} hubs={hubs} leaves={leaves} edges={} q/client={qpc} pool={pool_n} single_p95={single_p95:.2} mult={mult}",
            scale.as_str(),
            hubs * leaves,
        ),
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}
