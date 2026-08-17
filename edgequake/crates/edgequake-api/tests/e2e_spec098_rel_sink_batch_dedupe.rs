//! SPEC-098 LAW-098-8: relationship sink dedupes arbiter keys before ON CONFLICT.
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use edgequake_api::postgres_entity_sink::PostgresEntitySink;
use edgequake_pipeline::{EntitySinkRow, RelationalEntitySink, RelationshipSinkRow};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

fn require_db() -> Option<String> {
    let base = std::env::var("DATABASE_URL").ok()?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

#[tokio::test]
async fn e2e_spec098_rel_sink_batch_dedupe() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
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

    let sink = PostgresEntitySink::new_fail_closed(Arc::new(pool.clone()));
    sink.upsert_entities_batch(&[
        EntitySinkRow {
            name: "SRC_DEDUP".into(),
            entity_type: "ORG".into(),
            description: "s".into(),
            tenant_id: Some(tenant.to_string()),
            workspace_id: Some(workspace.to_string()),
            source_chunk_ids: vec![],
        },
        EntitySinkRow {
            name: "TGT_DEDUP".into(),
            entity_type: "ORG".into(),
            description: "t".into(),
            tenant_id: Some(tenant.to_string()),
            workspace_id: Some(workspace.to_string()),
            source_chunk_ids: vec![],
        },
    ])
    .await
    .expect("entities");

    // Duplicate arbiter key with mixed-case relation_type — must not raise 21000.
    let rels = vec![
        RelationshipSinkRow {
            source_name: "SRC_DEDUP".into(),
            target_name: "TGT_DEDUP".into(),
            relation_type: "knows".into(),
            description: "first".into(),
            weight: 0.5,
            tenant_id: Some(tenant.to_string()),
            workspace_id: Some(workspace.to_string()),
        },
        RelationshipSinkRow {
            source_name: "SRC_DEDUP".into(),
            target_name: "TGT_DEDUP".into(),
            relation_type: "KNOWS".into(),
            description: "second-wins".into(),
            weight: 1.0,
            tenant_id: Some(tenant.to_string()),
            workspace_id: Some(workspace.to_string()),
        },
    ];
    sink.upsert_relationships_batch(&rels)
        .await
        .expect("duplicate rel sink batch must not fail ON CONFLICT cardinality");

    let (desc, weight, rel_type): (String, f32, String) = sqlx::query_as(
        "SELECT description, weight, relation_type FROM relationships \
         WHERE workspace_id = $1 AND relation_type = 'KNOWS'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("one KNOWS row");
    assert_eq!(desc, "second-wins");
    assert!((weight - 1.0).abs() < f32::EPSILON);
    assert_eq!(rel_type, "KNOWS");

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM relationships WHERE workspace_id = $1 AND relation_type = 'KNOWS'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(n, 1);
}
