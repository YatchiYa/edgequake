//! SPEC-059 Wave 4 — true concurrent source_ids union under READ COMMITTED (M090).

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, GraphStorageReadOps};
use edgequake_storage::PostgresAGEGraphStorage;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn database_url() -> Option<String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return Some(url);
        }
    }
    std::fs::read_to_string("/tmp/edgequake-db-url")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn spec059_concurrent_source_ids_race_unions_all() {
    let Some(config) =
        postgres_test_config::contract_postgres_config("spec059_concurrent_source_ids")
    else {
        if std::env::var("EDGEQUAKE_REQUIRE_POSTGRES_TESTS").as_deref() == Ok("1") {
            panic!("DATABASE_URL or POSTGRES_PASSWORD required");
        }
        eprintln!("SKIP spec059_concurrent_source_ids: no DATABASE_URL / POSTGRES_PASSWORD");
        return;
    };

    // Race proof requires native ON CONFLICT + eq_merge (Cypher MERGE races UNIQUE).
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    // Ensure M090 merge function exists (idempotent CREATE OR REPLACE).
    if let Some(url) = database_url() {
        let pool = sqlx::PgPool::connect(&url).await.expect("connect for M090");
        let migration = include_str!("../../../migrations/090_eq_merge_graph_properties.sql");
        sqlx::raw_sql(migration)
            .execute(&pool)
            .await
            .expect("apply M090");
    }

    let storage = Arc::new(PostgresAGEGraphStorage::new(config));
    storage.initialize().await.expect("graph init");

    let node_id = "SPEC059_RACE_ENTITY";
    let n_workers = 12usize;
    let mut handles = Vec::with_capacity(n_workers);
    for w in 0..n_workers {
        let storage = Arc::clone(&storage);
        let doc = format!("doc-{w}");
        let chunk = format!("doc-{w}-chunk-0");
        handles.push(tokio::spawn(async move {
            // Batch path uses native ON CONFLICT + eq_merge; single upsert_node is Cypher.
            let props = HashMap::from([
                ("entity_type".to_string(), serde_json::json!("PERSON")),
                ("source_ids".to_string(), serde_json::json!([doc.clone()])),
                (
                    "source_chunk_ids".to_string(),
                    serde_json::json!([chunk]),
                ),
                (
                    "description".to_string(),
                    serde_json::json!(format!("from {doc}")),
                ),
            ]);
            storage
                .upsert_nodes_batch(&[(node_id.to_string(), props)])
                .await
        }));
    }

    for h in handles {
        h.await.expect("join").expect("upsert");
    }

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
    let ids_owned: HashSet<String> = sources
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let expected_owned: HashSet<String> = (0..n_workers).map(|w| format!("doc-{w}")).collect();
    assert_eq!(
        ids_owned, expected_owned,
        "concurrent upserts must union all source_ids via eq_merge_graph_properties; got {ids_owned:?}"
    );
    let _ = storage.delete_node(node_id).await;
}
