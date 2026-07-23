//! SPEC-083 D-30 — pre-multigraph EDGE schema upgrades on ensure, then native upsert works.
//!
//! Simulates the production split-brain: 2-col `eq_source_id`/`eq_target_id` present
//! (so old readiness looked green) but `eq_rel_type` missing → native INSERT failed.
//!
//! Run:
//! ```bash
//! export DATABASE_URL=...
//! cargo test -p edgequake-storage --features postgres --test e2e_d30_eq_rel_type_upgrade -- --nocapture
//! ```

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps};
use edgequake_storage::PostgresAGEGraphStorage;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

async fn strip_to_pre_d30(pool: &sqlx::PgPool, graph: &str) {
    // Drop D-30 pieces and restore legacy 2-col unique (pre-split-brain state).
    let drop_rel = format!(r#"DROP INDEX IF EXISTS {graph}.idx_edge_eq_source_target_rel"#);
    let drop_col = format!(r#"ALTER TABLE {graph}."EDGE" DROP COLUMN IF EXISTS eq_rel_type"#);
    let create_legacy = format!(
        r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_edge_eq_source_target
           ON {graph}."EDGE" (eq_source_id, eq_target_id)
           WHERE eq_source_id IS NOT NULL AND eq_target_id IS NOT NULL"#
    );
    sqlx::query(&drop_rel)
        .execute(pool)
        .await
        .expect("drop _rel index");
    sqlx::query(&drop_col)
        .execute(pool)
        .await
        .expect("drop eq_rel_type");
    sqlx::query(&create_legacy)
        .execute(pool)
        .await
        .expect("recreate 2-col unique");
}

async fn assert_d30_ready(pool: &sqlx::PgPool, graph: &str) {
    let ready: bool = sqlx::query_scalar(
        r#"
        SELECT
          EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = $1 AND table_name = 'EDGE' AND column_name = 'eq_rel_type'
          )
          AND EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE schemaname = $1 AND indexname = 'idx_edge_eq_source_target_rel'
          )
        "#,
    )
    .bind(graph)
    .fetch_one(pool)
    .await
    .expect("catalog probe");
    assert!(
        ready,
        "graph {graph} must have eq_rel_type + idx_edge_eq_source_target_rel"
    );
}

#[tokio::test]
async fn e2e_pre_d30_schema_upgrades_then_edge_upsert_succeeds() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("d30_rel") else {
        return;
    };

    let storage = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    storage.initialize().await.expect("initial graph init");
    let graph = storage.graph_name().to_string();

    let pool = postgres_test_config::contract_pg_pool(&config).await;
    strip_to_pre_d30(&pool, &graph).await;

    let missing: bool = sqlx::query_scalar(
        r#"
        SELECT NOT EXISTS (
          SELECT 1 FROM information_schema.columns
          WHERE table_schema = $1 AND table_name = 'EDGE' AND column_name = 'eq_rel_type'
        )
        "#,
    )
    .bind(&graph)
    .fetch_one(&pool)
    .await
    .expect("precondition probe");
    assert!(missing, "fixture must strip eq_rel_type before upgrade");

    // Fresh handle: indexes_verified=false so ensure_eq_id_columns runs again.
    let upgraded = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    upgraded.initialize().await.expect("upgrade init");
    assert_d30_ready(&pool, &graph).await;

    let mut a = HashMap::new();
    a.insert("node_id".into(), json!("D30_A"));
    a.insert("entity_type".into(), json!("CONCEPT"));
    let mut b = HashMap::new();
    b.insert("node_id".into(), json!("D30_B"));
    b.insert("entity_type".into(), json!("CONCEPT"));
    upgraded
        .upsert_nodes_batch(&[("D30_A".into(), a), ("D30_B".into(), b)])
        .await
        .expect("nodes");

    let mut edge = HashMap::new();
    edge.insert("source_id".into(), json!("D30_A"));
    edge.insert("target_id".into(), json!("D30_B"));
    edge.insert("relation_type".into(), json!("RELATED_TO"));
    edge.insert("keywords".into(), json!("d30"));
    edge.insert("source_ids".into(), json!(["d30-doc"]));

    upgraded
        .upsert_edges_batch(&[("D30_A".into(), "D30_B".into(), edge)])
        .await
        .expect("native edge upsert after D-30 upgrade must succeed");

    eprintln!("OK D-30: pre-multigraph schema upgraded; edge upsert succeeded on {graph}");
}
