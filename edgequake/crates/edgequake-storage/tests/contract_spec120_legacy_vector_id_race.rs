//! SPEC-120: same-workspace dual-FK `legacy_vector_id` collisions absorb (LAW-120-1..3).
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::embedding_family::EmbeddingFamily;
use edgequake_storage::traits::{
    FleetEmbeddingIndex, FleetEmbeddingKey, FleetEmbeddingRow, ModelId, WorkspaceId,
};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use uuid::Uuid;

const DIM: usize = 1024;

fn emb(seed: f32) -> Vec<f32> {
    vec![seed; DIM]
}

async fn seed_tenant_ws(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
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

async fn insert_entity(pool: &sqlx::PgPool, id: Uuid, name: &str, tenant: Uuid, workspace: Uuid) {
    sqlx::query(
        "INSERT INTO entities (id, name, entity_type, description, tenant_id, workspace_id, sync_status) \
         VALUES ($1, $2, 'PERSON', 'x', $3, $4, 'synced')",
    )
    .bind(id)
    .bind(name)
    .bind(tenant)
    .bind(workspace)
    .execute(pool)
    .await
    .expect("entity");
}

fn entity_row(ws: Uuid, eid: Uuid, lid: &str, seed: f32) -> FleetEmbeddingRow {
    FleetEmbeddingRow {
        workspace_id: WorkspaceId(ws),
        embedding: emb(seed),
        dimensions: DIM as i32,
        key: FleetEmbeddingKey::Entity(eid),
        legacy_vector_id: Some(lid.to_string()),
    }
}

/// T1 / EC-01 / EC-02: two FKs, same lid, concurrent upsert → both Ok; one owner.
#[tokio::test]
async fn contract_spec120_dual_fk_same_lid_concurrent_absorb() {
    let Some(cfg) = require_or_skip_postgres("spec120_dual_fk") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_tenant_ws(&pool).await;
    let e1 = Uuid::new_v4();
    let e2 = Uuid::new_v4();
    insert_entity(&pool, e1, "JOHN_SMITH", tenant, workspace).await;
    insert_entity(&pool, e2, "John Smith", tenant, workspace).await;

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "spec120-race");
    let lid = "entity:JOHN_SMITH";
    let rows_a = [entity_row(workspace, e1, lid, 0.01)];
    let rows_b = [entity_row(workspace, e2, lid, 0.02)];

    let (r1, r2) = tokio::join!(
        index.upsert_batch(EmbeddingFamily::Entity, ModelId(Uuid::nil()), &rows_a),
        index.upsert_batch(EmbeddingFamily::Entity, ModelId(Uuid::nil()), &rows_b),
    );
    let r1 = r1.expect("winner/loser upsert A must not 23505");
    let r2 = r2.expect("winner/loser upsert B must not 23505");
    assert!(
        r1.absorbed_legacy_collisions + r2.absorbed_legacy_collisions >= 1
            || (r1.upserted >= 1 && r2.upserted >= 1),
        "expected absorb or both stamped; got r1={r1:?} r2={r2:?}"
    );

    let owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM entity_embeddings \
         WHERE workspace_id = $1 AND legacy_vector_id = $2",
    )
    .bind(workspace)
    .bind(lid)
    .fetch_one(&pool)
    .await
    .expect("count owners");
    assert_eq!(owners, 1, "exactly one typed owner for lid");
}

/// T4: stamp-once — existing non-null lid is not overwritten.
#[tokio::test]
async fn contract_spec120_stamp_once_preserves_existing_lid() {
    let Some(cfg) = require_or_skip_postgres("spec120_stamp_once") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_tenant_ws(&pool).await;
    let eid = Uuid::new_v4();
    insert_entity(&pool, eid, "STAMP_ONCE", tenant, workspace).await;
    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "spec120-stamp");

    let first = entity_row(workspace, eid, "entity:STAMP_ONCE", 0.03);
    index
        .upsert_batch(EmbeddingFamily::Entity, ModelId(Uuid::nil()), &[first])
        .await
        .expect("first stamp");

    let second = entity_row(workspace, eid, "entity:OTHER_LID", 0.04);
    index
        .upsert_batch(EmbeddingFamily::Entity, ModelId(Uuid::nil()), &[second])
        .await
        .expect("second upsert");

    let lid: Option<String> =
        sqlx::query_scalar("SELECT legacy_vector_id FROM entity_embeddings WHERE entity_id = $1")
            .bind(eid)
            .fetch_one(&pool)
            .await
            .expect("lid");
    assert_eq!(lid.as_deref(), Some("entity:STAMP_ONCE"));
}

/// T3: multi-workspace same lid still allowed (migration 144).
#[tokio::test]
async fn contract_spec120_multi_ws_same_lid_allowed() {
    let Some(cfg) = require_or_skip_postgres("spec120_multi_ws") else {
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

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "spec120-multi-ws");
    let lid = "entity:SHARED_NAME";
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
        let eid = Uuid::new_v4();
        insert_entity(&pool, eid, "SHARED_NAME", tenant, workspace).await;
        let report = index
            .upsert_batch(
                EmbeddingFamily::Entity,
                ModelId(Uuid::nil()),
                &[entity_row(workspace, eid, lid, 0.05)],
            )
            .await
            .expect("cross-ws lid must succeed");
        assert!(report.upserted >= 1, "{report:?}");
        assert_eq!(report.absorbed_legacy_collisions, 0);
    }
}

/// T5: relationship family absorb.
#[tokio::test]
async fn contract_spec120_relationship_dual_fk_absorb() {
    let Some(cfg) = require_or_skip_postgres("spec120_rel_dual") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_tenant_ws(&pool).await;
    let src = Uuid::new_v4();
    let tgt = Uuid::new_v4();
    insert_entity(&pool, src, "SRC_A", tenant, workspace).await;
    insert_entity(&pool, tgt, "TGT_A", tenant, workspace).await;

    let r1 = Uuid::new_v4();
    let r2 = Uuid::new_v4();
    // Two relationship rows: distinct relation_type so UNIQUE allows both FKs.
    for (rid, rel_type) in [(r1, "LINKS"), (r2, "RELATED")] {
        sqlx::query(
            "INSERT INTO relationships \
             (id, source_id, target_id, tenant_id, workspace_id, relation_type, description, weight, sync_status) \
             VALUES ($1, $2, $3, $4, $5, $6, 'x', 1.0, 'synced')",
        )
        .bind(rid)
        .bind(src)
        .bind(tgt)
        .bind(tenant)
        .bind(workspace)
        .bind(rel_type)
        .execute(&pool)
        .await
        .expect("rel");
    }

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "spec120-rel");
    let lid = "SRC_A->TGT_A:LINKS";
    let mk = |rid: Uuid, seed: f32| FleetEmbeddingRow {
        workspace_id: WorkspaceId(workspace),
        embedding: emb(seed),
        dimensions: DIM as i32,
        key: FleetEmbeddingKey::Relationship(rid),
        legacy_vector_id: Some(lid.to_string()),
    };
    let rows_a = [mk(r1, 0.1)];
    let rows_b = [mk(r2, 0.2)];

    let (a, b) = tokio::join!(
        index.upsert_batch(EmbeddingFamily::Relationship, ModelId(Uuid::nil()), &rows_a),
        index.upsert_batch(EmbeddingFamily::Relationship, ModelId(Uuid::nil()), &rows_b),
    );
    a.expect("rel upsert A");
    b.expect("rel upsert B");

    let owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM relationship_embeddings \
         WHERE workspace_id = $1 AND legacy_vector_id = $2",
    )
    .bind(workspace)
    .bind(lid)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(owners, 1);
}

/// T5: report family absorb.
#[tokio::test]
async fn contract_spec120_report_dual_fk_absorb() {
    let Some(cfg) = require_or_skip_postgres("spec120_report_dual") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (_tenant, workspace) = seed_tenant_ws(&pool).await;
    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "spec120-report");
    // report_id is part of the global PK (model_id, report_id) — must be unique per run.
    let run = Uuid::new_v4();
    let lid = format!("community_report:cluster-{run}");
    let mk = |report_id: String, seed: f32| FleetEmbeddingRow {
        workspace_id: WorkspaceId(workspace),
        embedding: emb(seed),
        dimensions: DIM as i32,
        key: FleetEmbeddingKey::Report(report_id),
        legacy_vector_id: Some(lid.clone()),
    };
    let rows_a = [mk(format!("community_report:cluster-{run}#a"), 0.3)];
    let rows_b = [mk(format!("community_report:cluster-{run}#b"), 0.4)];

    let (a, b) = tokio::join!(
        index.upsert_batch(EmbeddingFamily::Report, ModelId(Uuid::nil()), &rows_a),
        index.upsert_batch(EmbeddingFamily::Report, ModelId(Uuid::nil()), &rows_b),
    );
    a.expect("report A");
    b.expect("report B");

    let owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM report_embeddings \
         WHERE workspace_id = $1 AND legacy_vector_id = $2",
    )
    .bind(workspace)
    .bind(&lid)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(owners, 1);
}

/// Mixed batch: colliding lid + fresh lid in one unnest (EC-07).
#[tokio::test]
async fn contract_spec120_mixed_collide_and_fresh() {
    let Some(cfg) = require_or_skip_postgres("spec120_mixed") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_tenant_ws(&pool).await;
    let e1 = Uuid::new_v4();
    let e2 = Uuid::new_v4();
    let e3 = Uuid::new_v4();
    insert_entity(&pool, e1, "MIX_A", tenant, workspace).await;
    insert_entity(&pool, e2, "Mix A", tenant, workspace).await;
    insert_entity(&pool, e3, "MIX_B", tenant, workspace).await;

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "spec120-mixed");
    // Pre-stamp winner for MIX_A lid
    index
        .upsert_batch(
            EmbeddingFamily::Entity,
            ModelId(Uuid::nil()),
            &[entity_row(workspace, e1, "entity:MIX_A", 0.1)],
        )
        .await
        .expect("prestamp");

    let report = index
        .upsert_batch(
            EmbeddingFamily::Entity,
            ModelId(Uuid::nil()),
            &[
                entity_row(workspace, e2, "entity:MIX_A", 0.2),
                entity_row(workspace, e3, "entity:MIX_B", 0.3),
            ],
        )
        .await
        .expect("mixed batch");
    assert!(
        report.absorbed_legacy_collisions >= 1,
        "loser MIX_A must absorb: {report:?}"
    );
    assert!(report.upserted >= 1, "fresh MIX_B must insert: {report:?}");

    let mix_b: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM entity_embeddings \
         WHERE entity_id = $1 AND legacy_vector_id = 'entity:MIX_B'",
    )
    .bind(e3)
    .fetch_one(&pool)
    .await
    .expect("mix_b");
    assert_eq!(mix_b, 1);
}
