//! SPEC-106 / Issue #356 — `get_edges_for_nodes_batch` must not compare raw
//! `ag_catalog.graphid` values (42883: operator does not exist).
//!
//! E2E-106-01: AGE upsert nodes+edge → batch edge read succeeds.
//! E2E-106-03: empty input → empty (no SQL).
//! E2E-106-02: static source guard (companion unit in this file).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
use edgequake_storage::PostgresAGEGraphStorage;
use std::collections::HashMap;
use std::path::PathBuf;

/// E2E-106-02: source must use ::text joins, never raw graphid JOIN predicates.
#[test]
fn e2e_106_02_source_guard_no_raw_graphid_join() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/adapters/postgres/graph/nodes_ops/read.rs");
    let src = std::fs::read_to_string(&path).expect("read.rs readable");
    assert!(
        !src.contains("JOIN vids src ON src.vid = e.start_id"),
        "SPEC-106: pg_get_edges_for_nodes_batch must not join raw graphid (Issue #356)"
    );
    assert!(
        !src.contains("JOIN vids tgt ON tgt.vid = e.end_id"),
        "SPEC-106: pg_get_edges_for_nodes_batch must not join raw graphid (Issue #356)"
    );
    assert!(
        src.contains("JOIN vids src ON src.vid_text = e.start_id::text"),
        "SPEC-106: expect LAW-G1 ::text join on start_id"
    );
    assert!(
        src.contains("JOIN vids tgt ON tgt.vid_text = e.end_id::text"),
        "SPEC-106: expect LAW-G1 ::text join on end_id"
    );
}

#[tokio::test]
async fn e2e_106_03_empty_node_ids_returns_empty() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e106_empty") else {
        return;
    };
    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");
    let edges = graph
        .get_edges_for_nodes_batch(&[])
        .await
        .expect("empty batch must not hit graphid operator");
    assert!(edges.is_empty());
    let _ = graph.clear().await;
}

#[tokio::test]
async fn e2e_106_01_get_edges_for_nodes_batch_no_graphid_operator_error() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e106_edges") else {
        return;
    };
    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");

    let mut na = HashMap::new();
    na.insert("entity_type".into(), serde_json::json!("PERSON"));
    let mut nb = HashMap::new();
    nb.insert("entity_type".into(), serde_json::json!("PERSON"));
    graph
        .upsert_nodes_batch(&[("EQ106_A".into(), na), ("EQ106_B".into(), nb)])
        .await
        .expect("nodes");

    let mut edge_props = HashMap::new();
    edge_props.insert("relation_type".into(), serde_json::json!("KNOWS"));
    edge_props.insert("description".into(), serde_json::json!("spec106"));
    graph
        .upsert_edges_batch(&[("EQ106_A".into(), "EQ106_B".into(), edge_props)])
        .await
        .expect("edge");

    let ids = vec!["EQ106_A".to_string(), "EQ106_B".to_string()];
    let edges = graph
        .get_edges_for_nodes_batch(&ids)
        .await
        .expect("SPEC-106: get_edges_for_nodes_batch must not raise graphid = graphid");

    assert!(
        edges
            .iter()
            .any(|e| e.source == "EQ106_A" && e.target == "EQ106_B"),
        "expected EQ106_A→EQ106_B in batch result, got {edges:?}"
    );

    let _ = graph.clear().await;
}
