//! Graph batch upsert contract (SPEC-017 P2).
//!
//! Verifies `upsert_nodes_batch` / `upsert_edges_batch` semantic parity across backends.

use std::collections::HashMap;

use edgequake_storage::traits::GraphStorage;

/// Batch-insert nodes and edges; assert all entities exist and counts match.
pub async fn assert_graph_batch_upsert<G: GraphStorage + ?Sized>(storage: &G) {
    let nodes: Vec<(String, HashMap<String, serde_json::Value>)> = (0..5)
        .map(|i| {
            let mut props = HashMap::new();
            props.insert("batch_idx".to_string(), serde_json::json!(i));
            (format!("BATCH_NODE_{i}"), props)
        })
        .collect();

    storage.upsert_nodes_batch(&nodes).await.unwrap();

    for (id, _) in &nodes {
        assert!(
            storage.has_node(id).await.unwrap(),
            "batch node missing: {id}"
        );
        let node = storage.get_node(id).await.unwrap().unwrap();
        assert_eq!(node.id, *id);
    }

    let edges: Vec<(String, String, HashMap<String, serde_json::Value>)> = (0..4)
        .map(|i| {
            (
                format!("BATCH_NODE_{i}"),
                format!("BATCH_NODE_{}", i + 1),
                HashMap::new(),
            )
        })
        .collect();

    storage.upsert_edges_batch(&edges).await.unwrap();

    for (src, tgt, _) in &edges {
        assert!(
            storage.has_edge(src, tgt).await.unwrap(),
            "batch edge missing: {src} -> {tgt}"
        );
    }

    assert_eq!(storage.node_count().await.unwrap(), 5);
    assert_eq!(storage.edge_count().await.unwrap(), 4);
}

/// Duplicate `(source, target)` rows in one batch must upsert as a single edge
/// (Postgres ON CONFLICT DO UPDATE forbids affecting a row twice).
pub async fn assert_graph_batch_upsert_dedupes_duplicate_endpoints<G: GraphStorage + ?Sized>(
    storage: &G,
) {
    let mut props_a = HashMap::new();
    props_a.insert("label".to_string(), serde_json::json!("A"));
    let mut props_b = HashMap::new();
    props_b.insert("label".to_string(), serde_json::json!("B"));
    storage
        .upsert_nodes_batch(&[
            ("DEDUP_A".to_string(), props_a),
            ("DEDUP_B".to_string(), props_b),
        ])
        .await
        .unwrap();

    let mut first = HashMap::new();
    first.insert("relation_type".to_string(), serde_json::json!("KNOWS"));
    first.insert("description".to_string(), serde_json::json!("first"));
    let mut second = HashMap::new();
    second.insert("relation_type".to_string(), serde_json::json!("WORKS_WITH"));
    second.insert("description".to_string(), serde_json::json!("second-wins"));

    storage
        .upsert_edges_batch(&[
            ("DEDUP_A".to_string(), "DEDUP_B".to_string(), first),
            ("DEDUP_A".to_string(), "DEDUP_B".to_string(), second.clone()),
            ("DEDUP_A".to_string(), "DEDUP_B".to_string(), second),
        ])
        .await
        .expect("duplicate endpoint batch must not fail ON CONFLICT cardinality");

    assert!(
        storage.has_edge("DEDUP_A", "DEDUP_B").await.unwrap(),
        "deduped edge must exist"
    );
    let edge = storage
        .get_edge("DEDUP_A", "DEDUP_B")
        .await
        .unwrap()
        .expect("edge after dedupe");
    assert_eq!(
        edge.properties
            .get("description")
            .and_then(|v| v.as_str()),
        Some("second-wins"),
        "last-write-wins on duplicate endpoint upsert"
    );
}
