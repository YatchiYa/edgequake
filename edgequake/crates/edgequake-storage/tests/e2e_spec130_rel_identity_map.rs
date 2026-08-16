//! SPEC-130 e2e: when name resolve cannot find endpoints, sink UUID map still mirrors.
//!
//! (Bare duplicate names are impossible under `entities_unique_name`; this proves
//! the same identity class — RelVectors must not depend on re-resolve.)
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::format_relationship_legacy_key;
use edgequake_storage::migration_engine::coverage::{
    load_entity_name_index_pool, resolve_relationship_id_pool,
};
use edgequake_storage::traits::FleetEmbeddingIndex;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn e2e_spec130_rel_identity_map() {
    let Some(cfg) = require_or_skip_postgres("e2e_spec130_rel_map") else {
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

    let src: Uuid = sqlx::query_scalar(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ('MELISSA_BOTHA', 'PERSON', 'm', $1, $2, 'synced') RETURNING id",
    )
    .bind(tenant)
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("src");

    let tgt: Uuid = sqlx::query_scalar(
        "INSERT INTO entities (name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ('FLAT_4', 'PLACE', 'f', $1, $2, 'synced') RETURNING id",
    )
    .bind(tenant)
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("tgt");

    let rel_id: Uuid = sqlx::query_scalar(
        "INSERT INTO relationships \
         (source_id, target_id, tenant_id, workspace_id, relation_type, description, weight, sync_status) \
         VALUES ($1, $2, $3, $4, 'OWNER_OF', 'owns', 1.0, 'synced') \
         RETURNING id",
    )
    .bind(src)
    .bind(tgt)
    .bind(tenant)
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("rel");

    // Break name→id resolution (rename endpoints) while keeping the edge UUID.
    sqlx::query("UPDATE entities SET name = name || '_RENAMED' WHERE id = ANY($1::uuid[])")
        .bind(&[src, tgt][..])
        .execute(&pool)
        .await
        .expect("rename");

    let index = load_entity_name_index_pool(&pool, workspace)
        .await
        .expect("index");
    assert!(
        index.resolve("MELISSA_BOTHA").is_none(),
        "renamed source must not resolve by old bare name"
    );

    let name_miss = resolve_relationship_id_pool(
        &pool,
        workspace,
        "MELISSA_BOTHA",
        "FLAT_4",
        "OWNER_OF",
        &index,
    )
    .await
    .expect("resolve");
    assert!(
        name_miss.is_none(),
        "name resolve must miss after endpoint rename"
    );

    let legacy = format_relationship_legacy_key("MELISSA_BOTHA", "FLAT_4", "OWNER_OF");
    let mut known = HashMap::new();
    known.insert(legacy.clone(), rel_id);

    let fleet = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "e2e-spec130");
    let emb = vec![0.07f32; 1024];

    let miss_report = fleet
        .mirror_legacy_batch(
            &[(
                legacy.clone(),
                emb.clone(),
                json!({ "workspace_id": workspace.to_string() }),
            )],
            false,
            None,
        )
        .await
        .expect("mirror without map");
    assert_eq!(miss_report.resolved, 0, "{miss_report:?}");
    assert!(!miss_report.is_complete());

    let report = fleet
        .mirror_legacy_batch(
            &[(
                legacy,
                emb,
                json!({ "workspace_id": workspace.to_string() }),
            )],
            false,
            Some(&known),
        )
        .await
        .expect("mirror with map");

    assert_eq!(report.resolved, 1, "{report:?}");
    assert!(report.is_complete(), "{report:?}");
    assert!(report.misses.is_empty());
}
