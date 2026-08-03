//! SPEC-098 F-098-09: Cypher edge MERGE (native writes OFF) must key on
//! `(source_id, target_id, relation_type)` — multigraph + within-batch dups.
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
async fn e2e_spec098_cypher_edge_multigraph() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e098_cypher_mg") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    // Force Cypher path (batch + single-edge) — F-098-09 gate.
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "0");

    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init");

    let mut na = HashMap::new();
    na.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    let mut nb = HashMap::new();
    nb.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    graph
        .upsert_nodes_batch(&[("CYPH_A".into(), na), ("CYPH_B".into(), nb)])
        .await
        .expect("nodes");

    // Batch: duplicate (src,tgt,rel) + multigraph + mixed-case + empty rel.
    let edges = vec![
        ("CYPH_A".into(), "CYPH_B".into(), props_rel("knows", "v1")),
        (
            "CYPH_A".into(),
            "CYPH_B".into(),
            props_rel("KNOWS", "v2-wins"),
        ),
        (
            "CYPH_A".into(),
            "CYPH_B".into(),
            props_rel("WORKS_WITH", "works"),
        ),
        ("CYPH_A".into(), "CYPH_B".into(), props_rel("", "empty-rel")),
        (
            "CYPH_A".into(),
            "CYPH_B".into(),
            props_rel("RELATED_TO", "related-wins"),
        ),
    ];

    graph
        .upsert_edges_batch(&edges)
        .await
        .expect("SPEC-098: Cypher batch must accept dup + multigraph");

    // Single-edge Cypher path: second rel-type on same endpoints.
    graph
        .upsert_edge("CYPH_A", "CYPH_B", props_rel("mentions", "single-path"))
        .await
        .expect("SPEC-098: single-edge Cypher MERGE includes relation_type");

    let all = graph.get_node_edges("CYPH_A").await.expect("edges");
    let ab: Vec<_> = all
        .into_iter()
        .filter(|e| e.source == "CYPH_A" && e.target == "CYPH_B")
        .collect();

    // KNOWS + WORKS_WITH + RELATED_TO + MENTIONS
    assert_eq!(
        ab.len(),
        4,
        "expected KNOWS + WORKS_WITH + RELATED_TO + MENTIONS, got {:?}",
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
        "mixed-case knows/KNOWS must LWW on Cypher path"
    );

    let mentions = ab
        .iter()
        .find(|e| e.properties.get("relation_type").and_then(|v| v.as_str()) == Some("MENTIONS"))
        .expect("MENTIONS from single-edge upsert");
    assert_eq!(
        mentions
            .properties
            .get("description")
            .and_then(|v| v.as_str()),
        Some("single-path")
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}
