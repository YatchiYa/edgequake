//! GH-350: knowledge-graph merge `get_nodes_batch` must not fail with
//! `type "agtype" does not exist` under pool `search_path=public`.
//!
//! Fingerprint from the issue:
//!   Batch query failed: ... type "agtype" does not exist
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
use edgequake_storage::PostgresAGEGraphStorage;
use std::collections::HashMap;

#[tokio::test]
async fn e2e_spec350_agtype_batch_get_nodes() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e350_agtype") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = PostgresAGEGraphStorage::new(config);
    graph.initialize().await.expect("init graph");

    let mut props = HashMap::new();
    props.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    props.insert(
        "description".into(),
        serde_json::json!("GH-350 agtype batch get probe"),
    );

    let node_id = "GH350_AGTYPE_PROBE".to_string();
    graph
        .upsert_nodes_batch(&[(node_id.clone(), props)])
        .await
        .expect("native upsert must succeed");

    let result = graph.get_nodes_batch(std::slice::from_ref(&node_id)).await;
    match &result {
        Ok(map) => {
            assert!(
                map.contains_key(&node_id),
                "get_nodes_batch must return upserted node; keys={:?}",
                map.keys().collect::<Vec<_>>()
            );
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                !msg.contains("agtype") || !msg.contains("does not exist"),
                "GH-350 regression: get_nodes_batch failed with agtype missing: {msg}"
            );
            panic!("get_nodes_batch failed unexpectedly: {msg}");
        }
    }

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = graph.clear().await;
}
