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
            None,
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
            None,
        )
        .await
        .expect("scoped mirror");
    assert_eq!(
        resolved_scoped.resolved, 1,
        "legacy scoped entities.name must still resolve via tolerant lookup"
    );
    assert!(resolved_scoped.is_complete());
}

/// Argus / SPEC-098 miss class: source entity name contains `->`.
#[tokio::test]
async fn contract_spec091_fleet_mirror_arrow_in_source_name() {
    let Some(cfg) = require_or_skip_postgres("fleet_mirror_arrow_src") else {
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

    let src_name = "27_->_25_STRENGTHENING";
    let tgt_name = "CLAIM_FRONTIER";
    sqlx::query(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ($1, 'CONCEPT', 'a', $2, $3, 'synced'), ($4, 'CONCEPT', 'b', $2, $3, 'synced')",
    )
    .bind(src_name)
    .bind(tenant)
    .bind(workspace)
    .bind(tgt_name)
    .execute(&pool)
    .await
    .expect("entities");

    let src: Uuid =
        sqlx::query_scalar("SELECT id FROM entities WHERE workspace_id = $1 AND name = $2")
            .bind(workspace)
            .bind(src_name)
            .fetch_one(&pool)
            .await
            .expect("src id");
    let tgt: Uuid =
        sqlx::query_scalar("SELECT id FROM entities WHERE workspace_id = $1 AND name = $2")
            .bind(workspace)
            .bind(tgt_name)
            .fetch_one(&pool)
            .await
            .expect("tgt id");

    sqlx::query(
        "INSERT INTO relationships \
         (source_id, target_id, tenant_id, workspace_id, relation_type, description, weight, sync_status) \
         VALUES ($1, $2, $3, $4, 'STRENGTHENS', 'link', 1.0, 'synced')",
    )
    .bind(src)
    .bind(tgt)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("rel");

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "fleet-mirror-arrow");
    let emb = vec![0.05f32; 1024];
    let legacy_id = format!("{src_name}->{tgt_name}:STRENGTHENS");
    let report = index
        .mirror_legacy_batch(
            &[(
                legacy_id,
                emb,
                json!({ "workspace_id": workspace.to_string() }),
            )],
            false,
            None,
        )
        .await
        .expect("rel mirror");

    assert_eq!(report.eligible, 1, "{report:?}");
    assert_eq!(
        report.resolved, 1,
        "arrow-in-source must resolve (was 999/1000 miss class): {report:?}"
    );
    assert!(report.is_complete());
}

/// Cross-workspace same `entity:NAME` legacy id must not collide (migration 144).
#[tokio::test]
async fn contract_spec091_fleet_mirror_legacy_id_ws_scoped() {
    let Some(cfg) = require_or_skip_postgres("fleet_mirror_legacy_ws") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let tenant = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .expect("tenant");

    let mut workspaces = Vec::new();
    for _ in 0..2 {
        let workspace = Uuid::new_v4();
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
             VALUES ('SHARED_NAME', 'ORG', 'x', $1, $2, 'synced')",
        )
        .bind(tenant)
        .bind(workspace)
        .execute(&pool)
        .await
        .expect("entity");
        workspaces.push(workspace);
    }

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "fleet-mirror-ws-legacy");
    let emb = vec![0.07f32; 1024];
    for ws in &workspaces {
        let report = index
            .mirror_legacy_batch(
                &[(
                    "entity:SHARED_NAME".into(),
                    emb.clone(),
                    json!({ "workspace_id": ws.to_string() }),
                )],
                true,
                None,
            )
            .await
            .expect("mirror must not hit global legacy_vector_id unique");
        assert_eq!(report.resolved, 1, "{report:?}");
        assert!(report.is_complete());
    }
}
