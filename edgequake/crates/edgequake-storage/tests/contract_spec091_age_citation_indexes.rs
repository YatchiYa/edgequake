//! SPEC-091 RM3 — AGE citation indexes present after ensure_indexes (RM-AC-09).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps};
use edgequake_storage::PostgresAGEGraphStorage;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn contract_spec091_age_citation_indexes_exist() {
    let Some(cfg) = require_or_skip_postgres("spec091_rm3_age_indexes") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let graph = PostgresAGEGraphStorage::new(cfg);
    graph.initialize().await.expect("graph init");

    let chunk = format!("{}-chunk-0", Uuid::new_v4());
    let mut props = HashMap::new();
    props.insert("source_chunk_ids".into(), serde_json::json!([chunk]));
    props.insert(
        "workspace_id".into(),
        serde_json::json!(Uuid::new_v4().to_string()),
    );
    props.insert("entity_type".into(), serde_json::json!("person"));

    graph
        .upsert_node(&format!("RM3_CITE_{}", Uuid::new_v4().simple()), props)
        .await
        .expect("upsert triggers ensure_indexes");

    let all: Vec<String> = sqlx::query_scalar(
        "SELECT indexname::text FROM pg_indexes \
         WHERE indexname LIKE '%source_chunk%' \
            OR indexname LIKE '%edge_props_gin%' \
            OR indexname LIKE '%edge_workspace_id%'",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    assert!(
        all.iter().any(|n| n.contains("source_chunk")),
        "RM-AC-09: expected source_chunk_ids GIN after ensure_indexes; got {all:?}"
    );
}
