//! SPEC-091: typed fleet mirror resolves bare entity names (and legacy scoped).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::FleetEmbeddingIndex;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn contract_spec091_fleet_mirror_bare_and_scoped_names() {
    let Some(cfg) = require_or_skip_postgres("fleet_mirror_names") else {
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

    // Bare row (typed sink SSOT)
    sqlx::query(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ('APPLE_INC', 'ORG', 'a', $1, $2, 'synced')",
    )
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("bare entity");

    // Legacy scoped row in another workspace
    let ws2 = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(ws2)
    .bind(tenant)
    .bind(format!("w-{ws2}"))
    .bind(format!("w-{ws2}"))
    .execute(&pool)
    .await
    .expect("ws2");
    let scoped_name = format!("{ws2}::BANANA_CO");
    sqlx::query(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ($1, 'ORG', 'b', $2, $3, 'synced')",
    )
    .bind(&scoped_name)
    .bind(tenant)
    .bind(ws2)
    .execute(&pool)
    .await
    .expect("scoped entity");

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "fleet-mirror-test");
    // Use 1024 so dim-scoped HNSW CHECK path is exercised if present.
    let emb = vec![0.01f32; 1024];

    let resolved_bare = index
        .mirror_legacy_batch(
            &[(
                "entity:APPLE_INC".into(),
                emb.clone(),
                json!({ "workspace_id": workspace.to_string() }),
            )],
            true,
        )
        .await
        .expect("bare mirror");
    assert_eq!(resolved_bare.resolved, 1, "bare entities.name must resolve");
    assert!(resolved_bare.is_complete());

    let resolved_scoped = index
        .mirror_legacy_batch(
            &[(
                "entity:BANANA_CO".into(),
                emb,
                json!({ "workspace_id": ws2.to_string() }),
            )],
            true,
        )
        .await
        .expect("scoped mirror");
    assert_eq!(
        resolved_scoped.resolved, 1,
        "legacy scoped entities.name must still resolve via tolerant lookup"
    );
    assert!(resolved_scoped.is_complete());
}
