//! SPEC-006 GraphScanOps proof tests.

use edgequake_storage::adapters::memory::MemoryGraphStorage;
use edgequake_storage::traits::{
    collect_source_references, is_topology_entity_ref, EdgeListFilter, GraphScanOps,
    GraphStorageMutateOps, NodeListFilter,
};
use serde_json::json;
use std::collections::HashMap;

const TENANT: &str = "scan-tenant";
const WORKSPACE: &str = "scan-workspace";

async fn seed_nodes(storage: &MemoryGraphStorage, count: usize) {
    for i in 0..count {
        let mut props = HashMap::new();
        props.insert("tenant_id".to_string(), json!(TENANT));
        props.insert("workspace_id".to_string(), json!(WORKSPACE));
        props.insert(
            "entity_type".to_string(),
            json!(if i % 2 == 0 { "PERSON" } else { "ORG" }),
        );
        storage
            .upsert_node(&format!("NODE_{:05}", i), props)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn graph_scan_ops_list_nodes_pagination() {
    let storage = MemoryGraphStorage::new("scan-test");
    seed_nodes(&storage, 500).await;

    let filter = NodeListFilter {
        tenant_id: Some(TENANT.to_string()),
        workspace_id: Some(WORKSPACE.to_string()),
        entity_type: Some("PERSON".to_string()),
        search: None,
        community_ids: None,
    };

    let page = storage.list_nodes_filtered(&filter, 0, 20).await.unwrap();

    assert_eq!(page.total, 250);
    assert_eq!(page.items.len(), 20);
}

#[tokio::test]
async fn graph_scan_ops_list_edges_empty_workspace() {
    let storage = MemoryGraphStorage::new("scan-test");
    let filter = EdgeListFilter {
        tenant_id: Some(TENANT.to_string()),
        workspace_id: Some(WORKSPACE.to_string()),
        relationship_type: None,
    };

    let page = storage.list_edges_filtered(&filter, 0, 10).await.unwrap();
    assert_eq!(page.total, 0);
    assert!(page.items.is_empty());
}

#[tokio::test]
async fn graph_scan_ops_find_by_source_prefix() {
    let storage = MemoryGraphStorage::new("scan-test");
    let mut props = HashMap::new();
    props.insert("tenant_id".to_string(), json!(TENANT));
    props.insert("workspace_id".to_string(), json!(WORKSPACE));
    props.insert(
        "source_ids".to_string(),
        json!(["doc-abc-chunk-1", "doc-xyz-chunk-2"]),
    );
    storage.upsert_node("SOURCED_NODE", props).await.unwrap();

    let filter = NodeListFilter {
        tenant_id: Some(TENANT.to_string()),
        workspace_id: Some(WORKSPACE.to_string()),
        entity_type: None,
        search: None,
        community_ids: None,
    };

    let found = storage
        .find_nodes_by_source_prefixes(&filter, &["doc-abc".to_string()])
        .await
        .unwrap();

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "SOURCED_NODE");
}

#[tokio::test]
async fn graph_scan_ops_find_edge_by_relationship_id() {
    let storage = MemoryGraphStorage::new("scan-test");
    let mut props = HashMap::new();
    props.insert("tenant_id".to_string(), json!(TENANT));
    props.insert("workspace_id".to_string(), json!(WORKSPACE));
    props.insert("keywords".to_string(), json!("works_at"));
    storage
        .upsert_node(
            "ALICE",
            HashMap::from([
                ("tenant_id".to_string(), json!(TENANT)),
                ("workspace_id".to_string(), json!(WORKSPACE)),
            ]),
        )
        .await
        .unwrap();
    storage
        .upsert_node(
            "GOOGLE",
            HashMap::from([
                ("tenant_id".to_string(), json!(TENANT)),
                ("workspace_id".to_string(), json!(WORKSPACE)),
            ]),
        )
        .await
        .unwrap();
    storage.upsert_edge("ALICE", "GOOGLE", props).await.unwrap();

    let filter = EdgeListFilter {
        tenant_id: Some(TENANT.to_string()),
        workspace_id: Some(WORKSPACE.to_string()),
        relationship_type: None,
    };

    let found = storage
        .find_edge_by_relationship_id(&filter, "ALICE_GOOGLE")
        .await
        .unwrap()
        .expect("edge");

    assert_eq!(found.source, "ALICE");
    assert_eq!(found.target, "GOOGLE");
}

/// SPEC-098 Symptom F: edge topology `source_id` must not count as provenance.
#[test]
fn collect_source_references_ignores_edge_endpoint_topology() {
    let mut props = HashMap::new();
    props.insert(
        "source_id".to_string(),
        json!("510ac733-c3b8-45c8-874c-7b811d209261::JSON"),
    );
    props.insert(
        "target_id".to_string(),
        json!("510ac733-c3b8-45c8-874c-7b811d209261::CONFIG"),
    );
    props.insert(
        "source_ids".to_string(),
        json!(["510ac733-c3b8-45c8-874c-7b811d209261::JSON"]),
    );
    props.insert(
        "source_chunk_id".to_string(),
        json!("019fbb91-5ac3-7e67-86a9-49015aa06eed-chunk-27"),
    );
    props.insert(
        "source_document_id".to_string(),
        json!("019fbb91-5ac3-7e67-86a9-49015aa06eed"),
    );

    assert!(is_topology_entity_ref(
        "510ac733-c3b8-45c8-874c-7b811d209261::JSON"
    ));
    let refs = collect_source_references(&props);
    assert!(
        refs.iter().all(|r| !r.contains("::")),
        "topology entity ids must be filtered: {refs:?}"
    );
    assert!(refs.iter().any(|r| r.ends_with("-chunk-27")));
    assert!(refs
        .iter()
        .any(|r| r == "019fbb91-5ac3-7e67-86a9-49015aa06eed"));
}

#[test]
fn collect_source_references_keeps_legacy_node_pipe_join() {
    let mut props = HashMap::new();
    props.insert("source_id".to_string(), json!("docA-chunk-0|docB-chunk-1"));
    let refs = collect_source_references(&props);
    assert_eq!(
        refs,
        vec!["docA-chunk-0".to_string(), "docB-chunk-1".to_string()]
    );
}

#[tokio::test]
async fn find_edges_by_source_prefixes_matches_singular_chunk_id() {
    let storage = MemoryGraphStorage::new("singular-prov");
    storage
        .upsert_node(
            "A",
            HashMap::from([
                ("tenant_id".to_string(), json!(TENANT)),
                ("workspace_id".to_string(), json!(WORKSPACE)),
            ]),
        )
        .await
        .unwrap();
    storage
        .upsert_node(
            "B",
            HashMap::from([
                ("tenant_id".to_string(), json!(TENANT)),
                ("workspace_id".to_string(), json!(WORKSPACE)),
            ]),
        )
        .await
        .unwrap();
    let mut props = HashMap::new();
    props.insert("tenant_id".to_string(), json!(TENANT));
    props.insert("workspace_id".to_string(), json!(WORKSPACE));
    props.insert("relation_type".to_string(), json!("USES"));
    // Poisoned arrays + singular citation (science_one shape).
    props.insert("source_id".to_string(), json!("ws::A"));
    props.insert("source_ids".to_string(), json!(["ws::A"]));
    props.insert("source_chunk_id".to_string(), json!("doc-singular-chunk-3"));
    storage.upsert_edge("A", "B", props).await.unwrap();

    let filter = EdgeListFilter {
        tenant_id: Some(TENANT.to_string()),
        workspace_id: Some(WORKSPACE.to_string()),
        relationship_type: None,
    };
    let found = storage
        .find_edges_by_source_prefixes(&filter, &["doc-singular".to_string()])
        .await
        .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].source, "A");
}
