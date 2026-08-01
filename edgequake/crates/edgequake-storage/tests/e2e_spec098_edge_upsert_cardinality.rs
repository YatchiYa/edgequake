//! SPEC-098: native AGE edge upsert must never raise SQLSTATE 21000
//! (`ON CONFLICT DO UPDATE cannot affect row a second time`).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
use edgequake_storage::PostgresAGEGraphStorage;
use std::collections::HashMap;

fn props_rel(rel: &str, desc: &str) -> HashMap<String, serde_json::Value> {
    let mut m = HashMap::new();
    m.insert("relation_type".into(), serde_json::json!(rel));
    m.insert("description".into(), serde_json::json!(desc));
    m
}

#[tokio::test]
async fn e2e_spec098_edge_upsert_cardinality() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e098_card") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");

    let mut na = HashMap::new();
    na.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    let mut nb = HashMap::new();
    nb.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    graph
        .upsert_nodes_batch(&[("CARD_A".into(), na), ("CARD_B".into(), nb)])
        .await
        .expect("nodes");

    // Duplicate (src,tgt,rel) + multigraph + mixed-case + empty rel in one batch.
    let edges = vec![
        (
            "CARD_A".into(),
            "CARD_B".into(),
            props_rel("knows", "v1"),
        ),
        (
            "CARD_A".into(),
            "CARD_B".into(),
            props_rel("KNOWS", "v2-wins"),
        ),
        (
            "CARD_A".into(),
            "CARD_B".into(),
            props_rel("WORKS_WITH", "works"),
        ),
        (
            "CARD_A".into(),
            "CARD_B".into(),
            props_rel("", "empty-rel"),
        ),
        (
            "CARD_A".into(),
            "CARD_B".into(),
            props_rel("RELATED_TO", "related-wins"),
        ),
    ];

    graph
        .upsert_edges_batch(&edges)
        .await
        .expect("SPEC-098: batch must not raise ON CONFLICT cardinality_violation");

    let all = graph.get_node_edges("CARD_A").await.expect("edges");
    let ab: Vec<_> = all
        .into_iter()
        .filter(|e| e.source == "CARD_A" && e.target == "CARD_B")
        .collect();
    // KNOWS + WORKS_WITH + RELATED_TO (empty collapsed into RELATED_TO)
    assert_eq!(
        ab.len(),
        3,
        "expected KNOWS + WORKS_WITH + RELATED_TO, got {:?}",
        ab.iter()
            .map(|e| e.properties.get("relation_type").cloned())
            .collect::<Vec<_>>()
    );

    let knows = ab
        .iter()
        .find(|e| e.properties.get("relation_type").and_then(|v| v.as_str()) == Some("KNOWS"))
        .expect("KNOWS");
    assert_eq!(
        knows.properties.get("description").and_then(|v| v.as_str()),
        Some("v2-wins"),
        "mixed-case knows/KNOWS must LWW"
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}
