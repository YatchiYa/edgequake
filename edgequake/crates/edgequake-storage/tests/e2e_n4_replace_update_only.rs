//! N4 regression: lineage prune (`GraphPropertyWriteMode::Replace`) must never
//! re-create a node/edge deleted concurrently, and `clear_workspace` must remove
//! `{workspace_uuid}::` vertices even when they lack the `workspace_id` property.
//!
//! Run: `cargo test -p edgequake-storage --features postgres --test e2e_n4_replace_update_only`
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{
    GraphPropertyWriteMode, GraphStorage, GraphStorageMutateOps, GraphStorageReadOps,
};
use edgequake_storage::PostgresAGEGraphStorage;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn lineage_only() -> HashMap<String, serde_json::Value> {
    // Exact shape of the N4 orphans (apply_retained_sources on empty props).
    [
        ("source_ids".to_string(), json!(["doc-chunk-0"])),
        ("source_chunk_ids".to_string(), json!(["doc-chunk-0"])),
        ("source_document_id".to_string(), json!("doc")),
        ("source_document_ids".to_string(), json!(["doc"])),
    ]
    .into()
}

#[tokio::test]
async fn n4_replace_on_missing_node_or_edge_does_not_create() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("n4replace") else {
        return;
    };
    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("graph init");
    let ws = Uuid::new_v4();
    let a = format!("{ws}::ALPHA");
    let b = format!("{ws}::BETA");
    let gone = format!("{ws}::GONE");

    let mut full = lineage_only();
    full.insert("workspace_id".into(), json!(ws.to_string()));
    full.insert("tenant_id".into(), json!("t-n4"));
    full.insert("entity_type".into(), json!("TEST"));
    graph
        .upsert_nodes_batch(&[(a.clone(), full.clone()), (b.clone(), full.clone())])
        .await
        .unwrap();

    // Missing node: prune write is a no-op.
    graph
        .upsert_nodes_batch_with_mode(
            &[(gone.clone(), lineage_only())],
            GraphPropertyWriteMode::Replace,
        )
        .await
        .unwrap();
    assert!(
        graph.get_node(&gone).await.unwrap().is_none(),
        "Replace must not create"
    );

    // Missing edge between existing nodes: prune write is a no-op.
    graph
        .upsert_edges_batch_with_mode(
            &[(a.clone(), b.clone(), lineage_only())],
            GraphPropertyWriteMode::Replace,
        )
        .await
        .unwrap();
    assert!(
        graph.get_edge(&a, &b).await.unwrap().is_none(),
        "Replace must not create edge"
    );

    // Existing node: Replace still replaces (subtractive prune sticks).
    let mut pruned = full.clone();
    pruned.insert("source_ids".into(), json!(["other-chunk-1"]));
    graph
        .upsert_nodes_batch_with_mode(&[(a.clone(), pruned)], GraphPropertyWriteMode::Replace)
        .await
        .unwrap();
    let node = graph.get_node(&a).await.unwrap().expect("alpha kept");
    assert_eq!(
        node.properties.get("source_ids"),
        Some(&json!(["other-chunk-1"]))
    );
    assert_eq!(
        node.properties.get("workspace_id"),
        Some(&json!(ws.to_string()))
    );

    // Existing edge: Replace updates it.
    graph.upsert_edge(&a, &b, full.clone()).await.unwrap();
    let mut edge_pruned = full.clone();
    edge_pruned.insert("source_ids".into(), json!(["other-chunk-1"]));
    graph
        .upsert_edges_batch_with_mode(
            &[(a.clone(), b.clone(), edge_pruned)],
            GraphPropertyWriteMode::Replace,
        )
        .await
        .unwrap();
    let edge = graph.get_edge(&a, &b).await.unwrap().expect("edge kept");
    assert_eq!(
        edge.properties.get("source_ids"),
        Some(&json!(["other-chunk-1"]))
    );

    graph.clear_workspace(&ws).await.unwrap();
    assert!(graph.get_node(&a).await.unwrap().is_none());
}

#[tokio::test]
async fn n4_clear_workspace_removes_prefix_only_orphans() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("n4clear") else {
        return;
    };
    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("graph init");
    let ws = Uuid::new_v4();
    let other = Uuid::new_v4();
    let orphan = format!("{ws}::PHYSICIST");
    let tagged = format!("{ws}::MARIE_CURIE");
    let foreign = format!("{other}::PHYSICIST");

    let mut tagged_props = lineage_only();
    tagged_props.insert("workspace_id".into(), json!(ws.to_string()));
    let mut foreign_props = lineage_only();
    foreign_props.insert("workspace_id".into(), json!(other.to_string()));
    graph
        .upsert_nodes_batch(&[
            (orphan.clone(), lineage_only()), // no workspace_id — the N4 shape
            (tagged.clone(), tagged_props),
            (foreign.clone(), foreign_props),
        ])
        .await
        .unwrap();
    graph
        .upsert_edge(&orphan, &tagged, HashMap::new())
        .await
        .unwrap();

    let (nodes, _edges) = graph.clear_workspace(&ws).await.expect("clear_workspace");
    assert_eq!(nodes, 2, "property match + prefix-only orphan");
    assert!(
        graph.get_node(&orphan).await.unwrap().is_none(),
        "orphan must be gone"
    );
    assert!(graph.get_node(&tagged).await.unwrap().is_none());
    assert!(graph.get_edge(&orphan, &tagged).await.unwrap().is_none());
    assert!(
        graph.get_node(&foreign).await.unwrap().is_some(),
        "other workspace untouched"
    );

    graph.clear_workspace(&other).await.unwrap();
    assert!(graph.get_node(&foreign).await.unwrap().is_none());
}
