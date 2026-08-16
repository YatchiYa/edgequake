//! SPEC-133: typed fleet mirror resolves when **target** entity names contain `->`.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::FleetEmbeddingIndex;
use edgequake_storage::{format_relationship_legacy_key, parse_relationship_legacy_key};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

/// UI / zz-raw miss class: target contains `->` (naive rsplit invents wrong endpoints).
#[tokio::test]
async fn contract_spec133_fleet_mirror_target_arrow_in_name() {
    let Some(cfg) = require_or_skip_postgres("fleet_mirror_arrow_tgt") else {
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

    let src_name = "FLOW_DIRECTION";
    let tgt_name = "ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)";
    let legacy_id = format_relationship_legacy_key(src_name, tgt_name, "RELATED_TO");

    // Prove naive parse is wrong for this class (reproduction gate).
    let naive = parse_relationship_legacy_key(&legacy_id).expect("naive parse");
    assert_ne!(
        (naive.0.as_str(), naive.1.as_str()),
        (src_name, tgt_name),
        "fixture must exercise target-arrow ambiguity"
    );

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
         VALUES ($1, $2, $3, $4, 'RELATED_TO', 'link', 1.0, 'synced')",
    )
    .bind(src)
    .bind(tgt)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("rel");

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "fleet-mirror-arrow-tgt");
    let emb = vec![0.05f32; 1024];
    // No known map — forces SPEC-133 index-guided parse fallback (map-miss path).
    let report = index
        .mirror_legacy_batch(
            &[(
                legacy_id.clone(),
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
        "target-arrow must resolve without known map (was 995/1000 miss class): {report:?}"
    );
    assert!(report.misses.is_empty(), "{report:?}");
    assert!(report.is_complete());
}

/// Fail-closed: target-arrow key with no matching entities stays unresolved.
#[tokio::test]
async fn contract_spec133_fleet_mirror_target_arrow_fail_closed() {
    let Some(cfg) = require_or_skip_postgres("fleet_mirror_arrow_tgt_miss") else {
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

    let legacy_id = format_relationship_legacy_key(
        "FLOW_DIRECTION",
        "ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)",
        "RELATED_TO",
    );
    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool, "fleet-mirror-arrow-miss");
    let emb = vec![0.05f32; 1024];
    let report = index
        .mirror_legacy_batch(
            &[(
                legacy_id.clone(),
                emb,
                json!({ "workspace_id": workspace.to_string() }),
            )],
            false,
            None,
        )
        .await
        .expect("rel mirror");

    assert_eq!(report.eligible, 1, "{report:?}");
    assert_eq!(report.resolved, 0, "{report:?}");
    assert!(!report.is_complete());
    assert!(
        report.misses.iter().any(|m| m == &legacy_id),
        "{report:?}"
    );
}
