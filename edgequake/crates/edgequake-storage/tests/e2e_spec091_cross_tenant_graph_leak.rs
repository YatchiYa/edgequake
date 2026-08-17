//! SPEC-091 IW5: cross-tenant graph isolation (Postgres AGE).
//!
//! Strict list/count APIs must not leak workspace B into workspace A queries.
//! LegacyNullAsWildcard discovery path is exercised for document cascade compat.
//!
//! Run:
//!   cargo test -p edgequake-storage --features postgres \
//!     --test e2e_spec091_cross_tenant_graph_leak -- --test-threads=1
#![cfg(feature = "postgres")]

#[path = "support/graph_workspace_contract.rs"]
#[allow(dead_code)]
mod graph_workspace_contract;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{
    GraphScanOps, GraphStorage, GraphStorageAnalyticsOps, GraphStorageMutateOps, NodeListFilter,
};
use edgequake_storage::PostgresAGEGraphStorage;
use postgres_test_config::require_or_skip_postgres;
use std::collections::HashMap;
use uuid::Uuid;

fn props(map: &[(&str, &str)]) -> HashMap<String, serde_json::Value> {
    graph_workspace_contract::props(map)
}

#[tokio::test]
async fn e2e_spec091_cross_tenant_graph_strict_filter_no_leak() {
    let Some(cfg) = require_or_skip_postgres("spec091_graph_leak") else {
        return;
    };
    let graph = PostgresAGEGraphStorage::new(cfg);
    graph.initialize().await.expect("graph init");

    let tenant_a = "tenant-a-iw5";
    let tenant_b = "tenant-b-iw5";
    let ws_a = Uuid::new_v4();
    let ws_b = Uuid::new_v4();

    for (id, ws, tenant) in [
        ("NODE_A1", ws_a, tenant_a),
        ("NODE_A2", ws_a, tenant_a),
        ("NODE_B1", ws_b, tenant_b),
    ] {
        graph
            .upsert_node(
                id,
                props(&[
                    ("entity_type", "person"),
                    ("tenant_id", tenant),
                    ("workspace_id", &ws.to_string()),
                ]),
            )
            .await
            .expect("upsert node");
    }

    // Unscoped legacy vertex — must NOT appear in strict workspace A list.
    graph
        .upsert_node("LEGACY_NULL_WS", HashMap::new())
        .await
        .expect("legacy node");

    assert_eq!(
        graph.node_count_by_workspace(&ws_a).await.expect("count a"),
        2,
        "workspace A counts only its nodes"
    );
    assert_eq!(
        graph.node_count_by_workspace(&ws_b).await.expect("count b"),
        1,
        "workspace B counts only its nodes"
    );

    let filter_a = NodeListFilter {
        tenant_id: Some(tenant_a.into()),
        workspace_id: Some(ws_a.to_string()),
        ..Default::default()
    };
    let page_a = graph
        .list_nodes_filtered(&filter_a, 0, 100)
        .await
        .expect("list a");
    assert_eq!(page_a.total, 2);
    let ids_a: Vec<_> = page_a.items.iter().map(|n| n.id.as_str()).collect();
    assert!(ids_a.contains(&"NODE_A1"));
    assert!(ids_a.contains(&"NODE_A2"));
    assert!(!ids_a.contains(&"NODE_B1"));
    assert!(
        !ids_a.contains(&"LEGACY_NULL_WS"),
        "strict filter excludes NULL workspace_id vertices"
    );

    let filter_b = NodeListFilter {
        tenant_id: Some(tenant_b.into()),
        workspace_id: Some(ws_b.to_string()),
        ..Default::default()
    };
    let page_b = graph
        .list_nodes_filtered(&filter_b, 0, 100)
        .await
        .expect("list b");
    assert_eq!(page_b.total, 1);
    assert_eq!(page_b.items[0].id, "NODE_B1");
}

#[tokio::test]
async fn e2e_spec091_cross_tenant_graph_legacy_null_wildcard_discovery() {
    let Some(cfg) = require_or_skip_postgres("spec091_graph_legacy") else {
        return;
    };
    let graph = PostgresAGEGraphStorage::new(cfg);
    graph.initialize().await.expect("graph init");

    let tenant = "tenant-legacy-iw5";
    let ws_a = Uuid::new_v4();
    let ws_b = Uuid::new_v4();
    let doc_a = format!("doc-a-{}", ws_a.as_simple());

    // Legacy-null node linked to workspace A document via source_ids.
    let mut legacy = HashMap::new();
    legacy.insert(
        "source_ids".into(),
        serde_json::json!([format!("{doc_a}-chunk-0")]),
    );
    legacy.insert("tenant_id".into(), serde_json::json!(tenant));
    graph
        .upsert_node("LEGACY_FOR_A", legacy)
        .await
        .expect("legacy upsert");

    graph
        .upsert_node("SCOPED_B", {
            let mut p = props(&[
                ("entity_type", "org"),
                ("tenant_id", tenant),
                ("workspace_id", &ws_b.to_string()),
            ]);
            p.insert(
                "source_ids".into(),
                serde_json::json!([format!("{doc_a}-chunk-0")]),
            );
            p
        })
        .await
        .expect("scoped b");

    let filter_a = NodeListFilter {
        tenant_id: Some(tenant.into()),
        workspace_id: Some(ws_a.to_string()),
        ..Default::default()
    };
    let found_a = graph
        .find_nodes_by_source_prefixes(&filter_a, std::slice::from_ref(&doc_a))
        .await
        .expect("discover a");
    assert!(
        found_a.iter().any(|n| n.id == "LEGACY_FOR_A"),
        "LegacyNullAsWildcard: null workspace matches cascade discovery for A"
    );
    assert!(
        !found_a.iter().any(|n| n.id == "SCOPED_B"),
        "workspace B scoped node must not leak into A discovery"
    );

    let filter_b = NodeListFilter {
        tenant_id: Some(tenant.into()),
        workspace_id: Some(ws_b.to_string()),
        ..Default::default()
    };
    let found_b = graph
        .find_nodes_by_source_prefixes(&filter_b, std::slice::from_ref(&doc_a))
        .await
        .expect("discover b");
    assert!(
        found_b.iter().any(|n| n.id == "SCOPED_B"),
        "workspace B sees its scoped node"
    );
}
