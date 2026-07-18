//! SPEC-060 — scoped AGE expand / incident-edges p95 + index EXPLAIN.
//!
//! ```bash
//! export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
//! export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
//! cargo test -p edgequake-storage --features postgres --test e2e_spec060_age_expand_perf -- --nocapture
//! ```

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;

use edgequake_storage::traits::{
    GraphStorage, GraphStorageMutateOps, GraphStorageReadOps,
};
use edgequake_storage::PostgresAGEGraphStorage;
use perf_harness::finish_report;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const NODE_COUNT: usize = 200;
const EDGES_PER_HUB: usize = 25; // 200 * 25 = 5_000 edges
const SAMPLES: usize = 20;
const WS: &str = "ws-expand060";
const TENANT: &str = "t-expand060";

fn percentile_p95(sorted: &[Duration]) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    sorted[idx.min(sorted.len() - 1)]
}

#[tokio::test]
async fn e2e_spec060_scoped_expand_p95_and_explain() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf060_expand") else {
        return;
    };

    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = PostgresAGEGraphStorage::new(config.clone());
    graph.initialize().await.expect("graph init");
    let graph_name = graph.graph_name().to_string();

    eprintln!(
        "SPEC-060 expand: seeding {NODE_COUNT} nodes + {} edges…",
        NODE_COUNT * EDGES_PER_HUB
    );
    let mut nodes = Vec::with_capacity(NODE_COUNT + NODE_COUNT * EDGES_PER_HUB);
    for i in 0..NODE_COUNT {
        let mut props = HashMap::new();
        props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
        props.insert("workspace_id".to_string(), serde_json::json!(WS));
        props.insert("tenant_id".to_string(), serde_json::json!(TENANT));
        nodes.push((format!("HUB_{i}"), props));
    }
    for i in 0..NODE_COUNT {
        for j in 0..EDGES_PER_HUB {
            let mut props = HashMap::new();
            props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
            props.insert("workspace_id".to_string(), serde_json::json!(WS));
            props.insert("tenant_id".to_string(), serde_json::json!(TENANT));
            nodes.push((format!("LEAF_{i}_{j}"), props));
        }
    }
    // Dedup by inserting hubs then leaves in batches
    for batch in nodes.chunks(500) {
        graph
            .upsert_nodes_batch(batch)
            .await
            .expect("upsert nodes");
    }

    let mut edges = Vec::with_capacity(NODE_COUNT * EDGES_PER_HUB);
    for i in 0..NODE_COUNT {
        for j in 0..EDGES_PER_HUB {
            let mut props = HashMap::new();
            props.insert("relation_type".to_string(), serde_json::json!("RELATED"));
            props.insert("workspace_id".to_string(), serde_json::json!(WS));
            props.insert("tenant_id".to_string(), serde_json::json!(TENANT));
            edges.push((
                format!("HUB_{i}"),
                format!("LEAF_{i}_{j}"),
                props,
            ));
        }
    }
    for batch in edges.chunks(500) {
        graph.upsert_edges_batch(batch).await.expect("upsert edges");
    }

    let probe: Vec<String> = (0..10).map(|i| format!("HUB_{i}")).collect();
    let _ = graph
        .get_incident_edges_batch(&probe, Some(TENANT), Some(WS))
        .await
        .expect("warmup");

    let mut samples = Vec::with_capacity(SAMPLES);
    for s in 0..SAMPLES {
        let ids: Vec<String> = (0..10).map(|i| format!("HUB_{}", (s + i) % NODE_COUNT)).collect();
        let start = Instant::now();
        let found = graph
            .get_incident_edges_batch(&ids, Some(TENANT), Some(WS))
            .await
            .expect("expand");
        samples.push(start.elapsed());
        assert!(
            found.len() >= EDGES_PER_HUB,
            "scoped expand must return incident edges; got {}",
            found.len()
        );
    }
    finish_report(
        "graph_get_incident_edges_batch",
        &samples,
        100.0,
        "eq_edge_btree",
        true,
        format!("edges={}", NODE_COUNT * EDGES_PER_HUB),
    );
    samples.sort();
    let p95 = percentile_p95(&samples);
    eprintln!("OK Q2-expand: p95={p95:?} max={:?}", samples.last());

    assert_incident_edge_index_plan(&config, &graph_name).await;

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}

async fn assert_incident_edge_index_plan(
    config: &edgequake_storage::PostgresConfig,
    graph: &str,
) {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    let sql = format!(
        r#"EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
           SELECT ag_catalog.agtype_to_json(e.properties) AS props
           FROM {graph}."EDGE" e
           WHERE e.eq_source_id = $1 OR e.eq_target_id = $1
           LIMIT 50"#
    );
    let plan_rows: Vec<(String,)> = sqlx::query_as(&sql)
        .bind("HUB_0")
        .fetch_all(&pool)
        .await
        .expect("EXPLAIN expand");
    let plan = plan_rows
        .into_iter()
        .map(|r| r.0)
        .collect::<Vec<_>>()
        .join("\n");
    let lower = plan.to_lowercase();
    assert!(
        lower.contains("index")
            || lower.contains("bitmap")
            || plan.contains("idx_edge_source")
            || plan.contains("idx_edge_target"),
        "expand EXPLAIN must use Index/Bitmap on edge ends; plan was:\n{plan}"
    );
    assert!(
        !lower.contains("seq scan on") || lower.contains("index"),
        "expand EXPLAIN must not be plain Seq Scan; plan was:\n{plan}"
    );
    eprintln!("OK EXPLAIN expand:\n{plan}");
}
