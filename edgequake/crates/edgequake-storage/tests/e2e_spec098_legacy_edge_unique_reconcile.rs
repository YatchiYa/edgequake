//! SPEC-098: boot reconcile drops legacy EDGE UNIQUEs so multigraph upserts succeed.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps};
use edgequake_storage::PostgresAGEGraphStorage;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn e2e_spec098_legacy_edge_unique_reconcile() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("e2e098_legacy") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let storage = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    storage.initialize().await.expect("init");
    let graph = storage.graph_name().to_string();
    let pool = postgres_test_config::contract_pg_pool(&config).await;

    // Seed a legacy endpoint-only expression UNIQUE (pre-D-30 hazard).
    // May fail if data already violates it — ignore create errors after drop attempt.
    let _ = sqlx::query(&format!(
        r#"DROP INDEX IF EXISTS {graph}.idx_edge_source_target_unique"#
    ))
    .execute(&pool)
    .await;
    let create = sqlx::query(&format!(
        r#"
        CREATE UNIQUE INDEX idx_edge_source_target_unique
        ON {graph}."EDGE" (
          (ag_catalog.agtype_to_json(properties)->>'source_id'),
          (ag_catalog.agtype_to_json(properties)->>'target_id')
        )
        "#
    ))
    .execute(&pool)
    .await;
    if let Err(e) = create {
        eprintln!("SKIP legacy index seed (non-fatal): {e}");
        match prev {
            Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
            None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
        }
        let _ = storage.clear().await;
        return;
    }

    // Re-init must reconcile and drop the legacy UNIQUE (LAW-098-7).
    let upgraded = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    upgraded.initialize().await.expect("re-init reconcile");

    let legacy_gone: bool = sqlx::query_scalar(
        r#"
        SELECT NOT EXISTS (
          SELECT 1 FROM pg_indexes
          WHERE schemaname = $1 AND indexname = 'idx_edge_source_target_unique'
        )
        "#,
    )
    .bind(&graph)
    .fetch_one(&pool)
    .await
    .expect("legacy check");
    assert!(
        legacy_gone,
        "reconcile_legacy_graph_arbiters must drop idx_edge_source_target_unique"
    );

    let rel_ok: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM pg_indexes
          WHERE schemaname = $1 AND indexname = 'idx_edge_eq_source_target_rel'
        )
        "#,
    )
    .bind(&graph)
    .fetch_one(&pool)
    .await
    .expect("3-col check");
    assert!(rel_ok, "3-col arbiter must remain");

    let mut a = HashMap::new();
    a.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    let mut b = HashMap::new();
    b.insert("entity_type".into(), serde_json::json!("CONCEPT"));
    upgraded
        .upsert_nodes_batch(&[("LEG_A".into(), a), ("LEG_B".into(), b)])
        .await
        .expect("nodes");

    let mut knows = HashMap::new();
    knows.insert("relation_type".into(), serde_json::json!("KNOWS"));
    let mut works = HashMap::new();
    works.insert("relation_type".into(), serde_json::json!("WORKS_WITH"));
    upgraded
        .upsert_edges_batch(&[
            ("LEG_A".into(), "LEG_B".into(), knows),
            ("LEG_A".into(), "LEG_B".into(), works),
        ])
        .await
        .expect("multigraph after legacy drop");

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
    let _ = upgraded.clear().await;
}
