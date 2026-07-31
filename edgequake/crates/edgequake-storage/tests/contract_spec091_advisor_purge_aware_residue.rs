//! SPEC-091 Doc 23 KVH-AC-08: typed-backed hash/wsdoc keys → residue 0.

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_fixture.rs"]
mod fixture;

use edgequake_storage::migration_engine::advisor::{self, CutoverPhase};
use postgres_test_config::require_or_skip_postgres;
use uuid::Uuid;

#[tokio::test]
async fn contract_spec091_advisor_purge_aware_residue() {
    let _guard = fixture::console_lock().lock().await;
    let Some(cfg) = require_or_skip_postgres("kvh_purge") else {
        return;
    };
    let url = fixture::predrop_fixture_url(&cfg)
        .await
        .expect("pre-drop fixture");
    let pool = sqlx::PgPool::connect(&url).await.expect("pool");
    fixture::clear_family_env();
    fixture::reset_predrop_fixture(&pool).await;

    let table = fixture::create_kv_table(&pool, "purgeaware").await;
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    let doc = Uuid::new_v4();
    let hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1,$2,$3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1,$2,$3,$4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(&pool)
    .await
    .expect("workspace");
    sqlx::query(
        "INSERT INTO public.documents (id, tenant_id, workspace_id, title, content, status) \
         VALUES ($1,$2,$3,'p','c','completed')",
    )
    .bind(doc)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("document");

    // Typed-backed staging + wsdoc keys (would be purged by migration 125).
    fixture::seed_kv_row(
        &pool,
        &table,
        &format!("staging:hash:{workspace}:{hash}"),
        serde_json::json!({"document_id": doc.to_string()}),
    )
    .await;
    fixture::seed_kv_row(
        &pool,
        &table,
        &format!("wsdoc:{workspace}:{doc}"),
        serde_json::json!({"ok": true}),
    )
    .await;
    sqlx::query(
        "INSERT INTO public.ingestion_dedup \
         (workspace_id, content_hash, pipeline_version, document_id) \
         VALUES ($1,$2,'staging',$3) \
         ON CONFLICT (workspace_id, content_hash, pipeline_version) DO NOTHING",
    )
    .bind(workspace)
    .bind(hash)
    .bind(doc)
    .execute(&pool)
    .await
    .expect("dedup staging");

    let residue = advisor::kv_durable_residue(&pool, &table)
        .await
        .expect("residue");
    assert_eq!(residue.staging_hash, 0, "typed-backed staging must not residue");
    assert_eq!(residue.wsdoc, 0, "typed-backed wsdoc must not residue");
    assert_eq!(
        advisor::guard_durable_total(&pool, &table)
            .await
            .expect("guard total"),
        0
    );

    // Flip all families relational so ReadyToDrop is reachable when residue=0.
    for var in fixture::ALL_FAMILY_ENV_VARS {
        std::env::set_var(var, "relational");
    }
    std::env::set_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY", "relational");
    let posture = advisor::posture(&pool).await.expect("posture");
    assert_eq!(posture.residue.total(), 0);
    assert_eq!(posture.cutover_phase, CutoverPhase::ReadyToDrop);

    fixture::clear_family_env();
}
