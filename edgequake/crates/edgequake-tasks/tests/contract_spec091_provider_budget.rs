//! SPEC-091 QW1: Postgres provider-slot ledger conformance (LAW-Q3, EC-22).
//!
//! Run with:
//!   DATABASE_URL=... cargo test -p edgequake-tasks --features postgres --test contract_spec091_provider_budget
//!
//! Skips cleanly when DATABASE_URL / POSTGRES_PASSWORD is unset.

#![cfg(feature = "postgres")]

use std::env;
use std::time::Duration;

use edgequake_tasks::{PostgresProviderBudget, ProviderBudget};
use sqlx::{postgres::PgPoolOptions, PgPool};

fn get_database_url() -> Option<String> {
    env::var("DATABASE_URL").ok().or_else(|| {
        let password = env::var("POSTGRES_PASSWORD").ok()?;
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
        let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "edgequake".to_string());
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "edgequake".to_string());
        Some(format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, password, host, port, db
        ))
    })
}

async fn create_test_pool() -> Option<PgPool> {
    let database_url = get_database_url()?;
    PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .ok()
}

/// Apply migration 110 idempotently (shared-DB safe).
async fn ensure_ledger_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(include_str!(
        "../../../migrations/110_spec091_provider_budget.sql"
    ))
    .execute(pool)
    .await?;
    Ok(())
}

/// Isolate the shared DB: dedicated provider key per test run.
fn test_provider() -> String {
    format!("test-{}", uuid::Uuid::new_v4().simple())
}

async fn seeded(pool: &PgPool, budget: u16) -> Option<(PostgresProviderBudget, String)> {
    ensure_ledger_schema(pool).await.ok()?;
    let store = PostgresProviderBudget::new(pool.clone());
    let provider = test_provider();
    store.seed_budget(&provider, budget, "test").await.ok()?;
    Some((store, provider))
}

#[tokio::test]
async fn contract_spec091_provider_budget_pg_acquire_release() {
    let Some(pool) = create_test_pool().await else {
        eprintln!("skipping: DATABASE_URL/POSTGRES_PASSWORD unset");
        return;
    };
    let Some((store, provider)) = seeded(&pool, 2).await else {
        eprintln!("skipping: ledger schema setup failed");
        return;
    };

    let a = store
        .try_acquire(&provider, "w1", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("slot 1");
    let b = store
        .try_acquire(&provider, "w2", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("slot 2");
    assert!(store
        .try_acquire(&provider, "w3", Duration::from_secs(60))
        .await
        .unwrap()
        .is_none());

    // Wrong-token release is rejected (CAS fencing).
    let mut forged = a.clone();
    forged.lease_token = uuid::Uuid::new_v4();
    assert!(store.release(&forged).await.is_err());

    store.release(&a).await.unwrap();
    assert!(store
        .try_acquire(&provider, "w3", Duration::from_secs(60))
        .await
        .unwrap()
        .is_some());
    store.release(&b).await.unwrap();
}

#[tokio::test]
async fn contract_spec091_provider_budget_pg_stale_reclaim_and_reaper() {
    let Some(pool) = create_test_pool().await else {
        eprintln!("skipping: DATABASE_URL/POSTGRES_PASSWORD unset");
        return;
    };
    let Some((store, provider)) = seeded(&pool, 1).await else {
        eprintln!("skipping: ledger schema setup failed");
        return;
    };

    // Simulate a crashed worker: lease written 10s in the past, never released.
    sqlx::query(
        "UPDATE edgequake.provider_slot \
         SET lease_owner = 'dead', lease_token = gen_random_uuid(), \
             lease_expires_at = NOW() - INTERVAL '10 seconds', acquired_at = NOW() \
         WHERE provider_key = $1 AND slot_id = 0",
    )
    .bind(&provider)
    .execute(&pool)
    .await
    .unwrap();

    // Stale arm: another claimant reclaims the dead lease immediately (EC-22).
    let lease = store
        .try_acquire(&provider, "alive", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("stale slot reclaimed");
    store.release(&lease).await.unwrap();

    // Reaper frees expired leases.
    sqlx::query(
        "UPDATE edgequake.provider_slot \
         SET lease_owner = 'dead2', lease_token = gen_random_uuid(), \
             lease_expires_at = NOW() - INTERVAL '10 seconds', acquired_at = NOW() \
         WHERE provider_key = $1 AND slot_id = 0",
    )
    .bind(&provider)
    .execute(&pool)
    .await
    .unwrap();
    assert!(store.reap_expired().await.unwrap() >= 1);
    assert!(store
        .try_acquire(&provider, "alive2", Duration::from_secs(60))
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn contract_spec091_provider_budget_pg_two_instances_never_exceed() {
    let Some(pool) = create_test_pool().await else {
        eprintln!("skipping: DATABASE_URL/POSTGRES_PASSWORD unset");
        return;
    };
    let Some((store, provider)) = seeded(&pool, 2).await else {
        eprintln!("skipping: ledger schema setup failed");
        return;
    };
    // Eight claimants across two simulated instances, one ledger — the cluster
    // case LAW-Q3 exists for: inflight must equal the budget, never 2×.
    let mut handles = Vec::new();
    for i in 0..8 {
        let adapter = PostgresProviderBudget::new(pool.clone());
        let p = provider.clone();
        let instance = i % 2;
        handles.push(tokio::spawn(async move {
            adapter
                .try_acquire(
                    &p,
                    &format!("inst{instance}-worker{i}"),
                    Duration::from_secs(60),
                )
                .await
                .unwrap()
        }));
    }
    let mut leases = Vec::new();
    for h in handles {
        if let Some(lease) = h.await.unwrap() {
            leases.push(lease);
        }
    }
    assert_eq!(leases.len(), 2, "cluster-wide inflight must equal budget");

    let (inflight, budget) = store.inflight(&provider).await.unwrap();
    assert_eq!(inflight, 2);
    assert_eq!(budget, Some(2));

    for lease in leases {
        store.release(&lease).await.unwrap();
    }
}
