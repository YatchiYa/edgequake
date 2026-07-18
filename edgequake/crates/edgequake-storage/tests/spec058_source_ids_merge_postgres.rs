//! SPEC-058 Wave 2 — concurrent-style source_ids union on native AGE upsert.

#[path = "support/postgres_test_config.rs"]
#[cfg(feature = "postgres")]
mod postgres_test_config;

#[cfg(feature = "postgres")]
mod postgres_integration {
    use super::postgres_test_config;
    use std::collections::HashMap;

    use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
    use edgequake_storage::PostgresAGEGraphStorage;

    #[tokio::test]
    async fn spec058_sequential_upserts_union_source_ids() {
        let Some(config) =
            postgres_test_config::contract_postgres_config("spec058_source_ids_merge")
        else {
            eprintln!("SKIP spec058_source_ids_merge: DATABASE_URL or POSTGRES_PASSWORD not set");
            return;
        };

        let storage = PostgresAGEGraphStorage::new(config);
        storage.initialize().await.expect("graph init");

        let node_id = "SPEC058_SHARED_ENTITY";
        storage
            .upsert_node(
                node_id,
                HashMap::from([
                    ("entity_type".to_string(), serde_json::json!("PERSON")),
                    (
                        "source_ids".to_string(),
                        serde_json::json!(["doc-a"]),
                    ),
                    (
                        "source_chunk_ids".to_string(),
                        serde_json::json!(["doc-a-chunk-0"]),
                    ),
                    ("description".to_string(), serde_json::json!("from A")),
                ]),
            )
            .await
            .expect("upsert A");

        storage
            .upsert_node(
                node_id,
                HashMap::from([
                    ("entity_type".to_string(), serde_json::json!("PERSON")),
                    (
                        "source_ids".to_string(),
                        serde_json::json!(["doc-b"]),
                    ),
                    (
                        "source_chunk_ids".to_string(),
                        serde_json::json!(["doc-b-chunk-0"]),
                    ),
                    ("description".to_string(), serde_json::json!("from B")),
                ]),
            )
            .await
            .expect("upsert B");

        let node = storage
            .get_node(node_id)
            .await
            .expect("get")
            .expect("node exists");

        let sources = node
            .properties
            .get("source_ids")
            .and_then(|v| v.as_array())
            .expect("source_ids array");
        let ids: Vec<&str> = sources.iter().filter_map(|v| v.as_str()).collect();
        assert!(
            ids.contains(&"doc-a") && ids.contains(&"doc-b"),
            "eq_merge_graph_properties must union source_ids, got {ids:?}"
        );
        assert_eq!(
            node.properties
                .get("description")
                .and_then(|v| v.as_str()),
            Some("from B"),
            "incoming scalar description wins"
        );

        let _ = storage.delete_node(node_id).await;
    }
}
