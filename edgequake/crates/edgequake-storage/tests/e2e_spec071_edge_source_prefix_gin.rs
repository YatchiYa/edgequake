//! SPEC-071 — edge source-prefix discovery uses child `"EDGE"` GIN + wall budget.
//!
//! Run:
//! ```bash
//! export DATABASE_URL=...
//! cargo test -p edgequake-storage --features postgres --test e2e_spec071_edge_source_prefix_gin -- --nocapture
//! ```

#![cfg(feature = "postgres")]

#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{
    EdgeListFilter, GraphScanOps, GraphStorage, GraphStorageMutateOps,
};
use edgequake_storage::{PostgresAGEGraphStorage, PostgresConfig};
use perf_harness::{assert_plan_uses_index, PlanKind};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DOC_A: &str = "spec071-doc-a";
const DOC_B: &str = "spec071-doc-b";
const WALL_BUDGET_MS: u128 = 2000;

fn node_props(id: &str, sources: &[&str]) -> (String, HashMap<String, serde_json::Value>) {
    let mut props = HashMap::new();
    props.insert("node_id".into(), json!(id));
    props.insert("entity_type".into(), json!("CONCEPT"));
    props.insert(
        "source_ids".into(),
        json!(sources.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
    );
    props.insert("tenant_id".into(), json!("t-spec071"));
    props.insert("workspace_id".into(), json!("ws-spec071"));
    (id.to_string(), props)
}

fn edge_props(sources: &[&str]) -> HashMap<String, serde_json::Value> {
    let mut props = HashMap::new();
    props.insert(
        "source_ids".into(),
        json!(sources.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
    );
    props.insert("keywords".into(), json!("related"));
    props
}

async fn assert_edge_source_ids_gin_plan(config: &PostgresConfig, graph: &str) {
    let pool = postgres_test_config::contract_pg_pool(config).await;
    // Same shape as modern edge discovery on child "EDGE" (SPEC-071).
    let sql = format!(
        r#"EXPLAIN (FORMAT TEXT)
           SELECT DISTINCT e.id
           FROM {graph}."EDGE" e
           CROSS JOIN LATERAL (
             SELECT unnest($1::text[]) AS probe_id
             UNION ALL
             SELECT p || '-chunk-' || g
             FROM unnest($1::text[]) AS p
             CROSS JOIN generate_series(0, 3) AS g
           ) pr
           WHERE ((ag_catalog.agtype_to_json(e.properties))::jsonb -> 'source_ids')
                 @> to_jsonb(pr.probe_id)
           LIMIT 100"#
    );
    let probes = vec![DOC_A.to_string()];
    let plan_rows: Vec<(String,)> = sqlx::query_as(&sql)
        .bind(&probes)
        .fetch_all(&pool)
        .await
        .expect("EXPLAIN edge GIN discovery");
    let plan = plan_rows
        .into_iter()
        .map(|r| r.0)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !plan.contains("_ag_label_edge"),
        "EXPLAIN must target child EDGE, not AGE parent: {plan}"
    );
    assert_plan_uses_index(&plan, &[PlanKind::Gin, PlanKind::Bitmap, PlanKind::Index]);
    eprintln!("OK SPEC-071 EXPLAIN edge source_ids GIN discovery:\n{plan}");
}

#[tokio::test]
async fn e2e_edge_source_prefix_gin_plan_and_wall() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("spec071_edge") else {
        return;
    };

    // Ensure default request path skips legacy SeqScan.
    std::env::remove_var("EDGEQUAKE_SOURCE_PREFIX_LEGACY");

    let storage = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    storage.initialize().await.expect("graph init");

    let nodes = vec![
        node_props("SPEC071_A1", &[DOC_A, &format!("{DOC_A}-chunk-0")]),
        node_props("SPEC071_A2", &[DOC_A, &format!("{DOC_A}-chunk-1")]),
        node_props("SPEC071_B1", &[DOC_B, &format!("{DOC_B}-chunk-0")]),
        node_props("SPEC071_B2", &[DOC_B, &format!("{DOC_B}-chunk-1")]),
    ];
    storage
        .upsert_nodes_batch(&nodes)
        .await
        .expect("seed nodes");

    let edges = vec![
        (
            "SPEC071_A1".into(),
            "SPEC071_A2".into(),
            edge_props(&[DOC_A, &format!("{DOC_A}-chunk-0")]),
        ),
        (
            "SPEC071_B1".into(),
            "SPEC071_B2".into(),
            edge_props(&[DOC_B, &format!("{DOC_B}-chunk-0")]),
        ),
    ];
    storage
        .upsert_edges_batch(&edges)
        .await
        .expect("seed edges");

    let graph = storage.graph_name().to_string();
    assert_edge_source_ids_gin_plan(&config, &graph).await;

    let filter = EdgeListFilter {
        tenant_id: Some("t-spec071".into()),
        workspace_id: Some("ws-spec071".into()),
        relationship_type: None,
    };
    let prefixes_a = vec![DOC_A.to_string()];
    let prefixes_b = vec![DOC_B.to_string()];

    let start = Instant::now();
    let s1 = Arc::clone(&storage);
    let s2 = Arc::clone(&storage);
    let f1 = filter.clone();
    let f2 = filter.clone();
    let (r1, r2) = tokio::join!(
        async move {
            s1.find_edges_by_source_prefixes(&f1, &prefixes_a)
                .await
                .expect("discover edges A")
        },
        async move {
            s2.find_edges_by_source_prefixes(&f2, &prefixes_b)
                .await
                .expect("discover edges B")
        }
    );
    let elapsed = start.elapsed();
    assert!(
        !r1.is_empty(),
        "expected at least one edge for DOC_A via source_ids GIN"
    );
    assert!(
        !r2.is_empty(),
        "expected at least one edge for DOC_B via source_ids GIN"
    );
    assert!(
        elapsed < Duration::from_millis(WALL_BUDGET_MS as u64),
        "concurrent find_edges_by_source_prefixes wall {:?} exceeds {}ms budget",
        elapsed,
        WALL_BUDGET_MS
    );

    eprintln!(
        "OK SPEC-071: concurrent edge discovery {:.0}ms (hits A={}, B={})",
        elapsed.as_secs_f64() * 1000.0,
        r1.len(),
        r2.len()
    );
}
