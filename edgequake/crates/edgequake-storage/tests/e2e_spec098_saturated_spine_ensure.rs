//! SPEC-098 e2e: missing relational spine → miss; after ensure → fleet resolves.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::FleetEmbeddingIndex;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn e2e_spec098_saturated_spine_ensure() {
    let Some(cfg) = require_or_skip_postgres("e2e_spec098_spine") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();

    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(&pool)
    .await
    .expect("workspace");

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "e2e-spec098");
    let emb = vec![0.05f32; 1024];
    let legacy = [(
        "entity:DRAFT_ENTITY".into(),
        emb.clone(),
        json!({ "workspace_id": workspace.to_string() }),
    )];

    // Simulate saturated KEEP with AGE present but no relational spine.
    let before = index
        .mirror_legacy_batch(&legacy, true, None)
        .await
        .expect("mirror before");
    assert_eq!(before.resolved, 0);
    assert!(!before.is_complete());

    // Spine ensure (what merger now does on saturated KEEP).
    sqlx::query(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ('DRAFT_ENTITY', 'CONCEPT', 'ensured', $1, $2, 'synced') \
         ON CONFLICT (tenant_id, workspace_id, name) DO UPDATE SET sync_status = 'synced'",
    )
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("spine ensure");

    let after = index
        .mirror_legacy_batch(&legacy, true, None)
        .await
        .expect("mirror after");
    assert_eq!(after.resolved, 1, "spine ensure must unlock fleet FK");
    assert!(after.is_complete());

    let fleet_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM entity_embeddings ee \
         JOIN entities e ON e.id = ee.entity_id \
         WHERE ee.workspace_id = $1 AND e.name = 'DRAFT_ENTITY'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("fleet count");
    assert!(fleet_count >= 1, "entity_embeddings row expected");
}
