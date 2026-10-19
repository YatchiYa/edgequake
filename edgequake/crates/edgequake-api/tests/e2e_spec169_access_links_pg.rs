//! SPEC-169 — `AccessLinkStore` contract: memory always; PostgreSQL (migration 214) when
//! `EDGEQUAKE_SPEC169_PG_URL` points at a scratch database.
#![cfg(feature = "postgres")]

use std::sync::Arc;

use edgequake_api::services::access_links::pg_store::PgAccessLinkStore;
use edgequake_api::services::access_links::{
    AccessLink, AccessLinkStore, AccessPurpose, MemoryAccessLinkStore,
};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use uuid::Uuid;

const MIGRATION_214: &str = include_str!("../../../migrations/214_spec169_user_access_links.sql");
const SCHEMA: &str = "spec169_links";

fn link(user: Uuid, tenant: Uuid, purpose: AccessPurpose, hash: &str) -> AccessLink {
    AccessLink::new(
        user,
        tenant,
        purpose,
        hash.to_string(),
        Some((Uuid::new_v4(), "ada".into())),
    )
}

/// The behaviour every adapter must have.
async fn contract(store: Arc<dyn AccessLinkStore>, tenant: Uuid) {
    let user = Uuid::new_v4();
    let first = link(user, tenant, AccessPurpose::MagicLogin, "h-first");
    store.issue(&first).await.unwrap();
    let found = store.find_open_by_token("h-first").await.unwrap().unwrap();
    assert_eq!(found.link_id, first.link_id);
    assert_eq!(found.purpose, AccessPurpose::MagicLogin);
    assert_eq!(found.created_by_name.as_deref(), Some("ada"));

    // Re-issuing supersedes that purpose only.
    let reset = link(user, tenant, AccessPurpose::PasswordReset, "h-reset");
    store.issue(&reset).await.unwrap();
    store
        .issue(&link(user, tenant, AccessPurpose::MagicLogin, "h-second"))
        .await
        .unwrap();
    assert!(store.find_open_by_token("h-first").await.unwrap().is_none());
    assert!(store
        .find_open_by_token("h-second")
        .await
        .unwrap()
        .is_some());
    assert!(store.find_open_by_token("h-reset").await.unwrap().is_some());

    // Claim is single use, even under a race.
    let store = Arc::new(store);
    let wins: Vec<_> = futures::future::join_all((0..8).map(|_| {
        let s = Arc::clone(&store);
        tokio::spawn(async move { s.claim("h-second").await.unwrap() })
    }))
    .await
    .into_iter()
    .filter_map(|r| r.unwrap())
    .collect();
    assert_eq!(wins.len(), 1, "exactly one concurrent claim may win");
    assert!(wins[0].used_at.is_some());
    assert!(store
        .find_open_by_token("h-second")
        .await
        .unwrap()
        .is_none());
    assert!(store.claim("h-second").await.unwrap().is_none());

    // Undoing a claim makes the link usable again.
    store.release(wins[0].link_id).await.unwrap();
    assert!(store
        .find_open_by_token("h-second")
        .await
        .unwrap()
        .is_some());

    // Password change kills everything still open.
    assert_eq!(store.revoke_for_user(user, None).await.unwrap(), 2);
    assert!(store
        .find_open_by_token("h-second")
        .await
        .unwrap()
        .is_none());
    assert!(store.find_open_by_token("h-reset").await.unwrap().is_none());
    assert_eq!(store.revoke_for_user(user, None).await.unwrap(), 0);
}

#[tokio::test]
async fn memory_store_satisfies_contract() {
    contract(Arc::new(MemoryAccessLinkStore::new()), Uuid::new_v4()).await;
}

async fn scratch_pool() -> Option<PgPool> {
    let url = std::env::var("EDGEQUAKE_SPEC169_PG_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute(format!("SET search_path = {SCHEMA}").as_str())
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect EDGEQUAKE_SPEC169_PG_URL");
    for stmt in [
        format!("DROP SCHEMA IF EXISTS {SCHEMA} CASCADE"),
        format!("CREATE SCHEMA {SCHEMA}"),
    ] {
        sqlx::query(&stmt).execute(&pool).await.unwrap();
    }
    pool.execute("CREATE TABLE tenants (tenant_id UUID PRIMARY KEY DEFAULT gen_random_uuid())")
        .await
        .unwrap();
    Some(pool)
}

async fn apply_214(pool: &PgPool) {
    let sql = MIGRATION_214.replace("SET search_path = public;", "");
    pool.execute(sql.as_str())
        .await
        .expect("migration 214 applies");
}

#[tokio::test]
#[serial(spec169_pg)]
async fn postgres_store_satisfies_contract_and_migration_is_idempotent() {
    let Some(pool) = scratch_pool().await else {
        eprintln!("skip: set EDGEQUAKE_SPEC169_PG_URL to run PostgreSQL contract tests");
        return;
    };
    apply_214(&pool).await;
    apply_214(&pool).await;
    let tenant = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();

    contract(Arc::new(PgAccessLinkStore::new(pool.clone())), tenant).await;

    // The partial unique index is the backstop: two open links of one purpose cannot coexist.
    let user = Uuid::new_v4();
    let a = link(user, tenant, AccessPurpose::PasswordReset, "dup-a");
    let b = link(user, tenant, AccessPurpose::PasswordReset, "dup-b");
    PgAccessLinkStore::new(pool.clone())
        .issue(&a)
        .await
        .unwrap();
    let raw = sqlx::query(
        "INSERT INTO user_access_links (user_id, tenant_id, purpose, token_hash, expires_at) \
         VALUES ($1, $2, 'password_reset', $3, NOW() + interval '1 hour')",
    )
    .bind(user)
    .bind(tenant)
    .bind(&b.token_hash)
    .execute(&pool)
    .await;
    assert!(
        raw.is_err(),
        "uq_user_access_links_open rejects a second open link"
    );
}
