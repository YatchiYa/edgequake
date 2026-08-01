//! SPEC-098: fleet mirror report — miss evidence, invalid workspace, rel type case.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::FleetEmbeddingIndex;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

async fn seed_tenant_workspace(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(pool)
    .await
    .expect("workspace");
    (tenant, workspace)
}

#[tokio::test]
async fn contract_spec098_fleet_mirror_miss_sample() {
    let Some(cfg) = require_or_skip_postgres("spec098_mirror_miss") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (_tenant, workspace) = seed_tenant_workspace(&pool).await;

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "spec098-miss");
    let emb = vec![0.02f32; 1024];
    let report = index
        .mirror_legacy_batch(
            &[(
                "entity:MISSING_ORG".into(),
                emb,
                json!({ "workspace_id": workspace.to_string() }),
            )],
            true,
        )
        .await
        .expect("mirror");

    assert_eq!(report.eligible, 1);
    assert_eq!(report.resolved, 0);
    assert!(!report.is_complete());
    assert!(
        report.misses.iter().any(|m| m.contains("MISSING_ORG")),
        "miss sample: {:?}",
        report.misses
    );
}

#[tokio::test]
async fn contract_spec098_invalid_workspace_loud() {
    let Some(cfg) = require_or_skip_postgres("spec098_invalid_ws") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "spec098-ws");
    let emb = vec![0.03f32; 1024];
    let report = index
        .mirror_legacy_batch(
            &[(
                "entity:ANY".into(),
                emb,
                json!({ "workspace_id": "not-a-uuid" }),
            )],
            true,
        )
        .await
        .expect("mirror");

    assert_eq!(report.eligible, 0);
    assert_eq!(report.resolved, 0);
    assert!(
        !report.invalid_workspace.is_empty(),
        "expected invalid_workspace sample, got {report:?}"
    );
}

#[tokio::test]
async fn contract_spec098_relation_type_uppercase_resolve() {
    let Some(cfg) = require_or_skip_postgres("spec098_rel_case") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_tenant_workspace(&pool).await;

    sqlx::query(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ('ALPHA', 'ORG', 'a', $1, $2, 'synced'), ('BETA', 'ORG', 'b', $1, $2, 'synced')",
    )
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("entities");

    let src: Uuid = sqlx::query_scalar(
        "SELECT id FROM entities WHERE workspace_id = $1 AND name = 'ALPHA'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("src id");
    let tgt: Uuid = sqlx::query_scalar(
        "SELECT id FROM entities WHERE workspace_id = $1 AND name = 'BETA'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("tgt id");

    sqlx::query(
        "INSERT INTO relationships \
         (source_id, target_id, tenant_id, workspace_id, relation_type, description, weight, sync_status) \
         VALUES ($1, $2, $3, $4, 'WORKS_WITH', 'link', 1.0, 'synced')",
    )
    .bind(src)
    .bind(tgt)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("rel");

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "spec098-rel");
    let emb = vec![0.04f32; 1024];
    // Mixed-case type in legacy id — mirror normalizes to WORKS_WITH.
    let report = index
        .mirror_legacy_batch(
            &[(
                "ALPHA->BETA:Works_With".into(),
                emb,
                json!({ "workspace_id": workspace.to_string() }),
            )],
            false,
        )
        .await
        .expect("rel mirror");

    assert_eq!(report.eligible, 1);
    assert_eq!(report.resolved, 1, "uppercase SSOT must resolve: {report:?}");
    assert!(report.is_complete());
}
