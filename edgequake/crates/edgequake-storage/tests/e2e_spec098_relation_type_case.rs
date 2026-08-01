//! SPEC-098 e2e: mixed-case relation type in vector id resolves after uppercase SSOT.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::FleetEmbeddingIndex;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn e2e_spec098_relation_type_case() {
    let Some(cfg) = require_or_skip_postgres("e2e_spec098_rel") else {
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

    sqlx::query(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ('SRC_A', 'ORG', 'a', $1, $2, 'synced'), ('TGT_B', 'ORG', 'b', $1, $2, 'synced')",
    )
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("entities");

    let src: Uuid =
        sqlx::query_scalar("SELECT id FROM entities WHERE workspace_id = $1 AND name = 'SRC_A'")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    let tgt: Uuid =
        sqlx::query_scalar("SELECT id FROM entities WHERE workspace_id = $1 AND name = 'TGT_B'")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();

    sqlx::query(
        "INSERT INTO relationships \
         (source_id, target_id, tenant_id, workspace_id, relation_type, description, weight, sync_status) \
         VALUES ($1, $2, $3, $4, 'RELATED_TO', 'link', 1.0, 'synced')",
    )
    .bind(src)
    .bind(tgt)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("rel");

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "e2e-rel-case");
    let emb = vec![0.06f32; 1024];
    let report = index
        .mirror_legacy_batch(
            &[(
                "SRC_A->TGT_B:related_to".into(),
                emb,
                json!({ "workspace_id": workspace.to_string() }),
            )],
            false,
        )
        .await
        .expect("mirror");

    assert_eq!(report.resolved, 1, "{report:?}");
    assert!(report.is_complete());

    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM relationship_embeddings WHERE workspace_id = $1",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(n >= 1);
}
