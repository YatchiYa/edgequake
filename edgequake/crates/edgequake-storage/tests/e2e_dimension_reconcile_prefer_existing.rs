//! Live Postgres e2e: dimension reconcile First Principles (SPEC-058).
//!
//! - Empty mismatch → RecreateEmpty (no ALLOW flag)
//! - Non-empty FailClosed → error
//! - Non-empty PreferExisting → KeptExisting (no DROP)
//!
//! Run:
//! ```bash
//! export DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake
//! cargo test -p edgequake-storage --features postgres --test e2e_dimension_reconcile_prefer_existing -- --nocapture
//! ```

#![cfg(feature = "postgres")]

use edgequake_storage::adapters::postgres::{PgVectorStorage, PostgresConfig, PostgresPool};
use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{DimensionEnsureOutcome, DimensionReconcilePolicy};
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

const DIM_A: usize = 8;
const DIM_B: usize = 16;

fn test_config(namespace: &str) -> Option<PostgresConfig> {
    if let Ok(url) = env::var("DATABASE_URL") {
        let without_scheme = url.split("://").nth(1)?;
        let (auth, host_path) = without_scheme.split_once('@')?;
        let (user, password) = auth.split_once(':')?;
        let (host_port, db_path) = host_path.split_once('/')?;
        let db = db_path.split('?').next().unwrap_or(db_path).to_string();
        let (host, port) = match host_port.split_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().ok()?),
            None => (host_port.to_string(), 5432),
        };
        return Some(
            PostgresConfig::new(host, port, db, user.to_string(), password.to_string())
                .with_namespace(namespace),
        );
    }
    let password = env::var("POSTGRES_PASSWORD").ok()?;
    Some(
        PostgresConfig::new(
            env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()),
            env::var("POSTGRES_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5432),
            env::var("POSTGRES_DB").unwrap_or_else(|_| "edgequake".to_string()),
            env::var("POSTGRES_USER").unwrap_or_else(|_| "edgequake".to_string()),
            password,
        )
        .with_namespace(namespace),
    )
}

fn config_or_skip(prefix: &str) -> Option<PostgresConfig> {
    if let Some(cfg) = test_config(prefix) {
        return Some(cfg);
    }
    if env::var("EDGEQUAKE_REQUIRE_POSTGRES_TESTS")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        panic!("EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1 but DATABASE_URL missing");
    }
    eprintln!("SKIP: DATABASE_URL / POSTGRES_PASSWORD not set");
    None
}

async fn pool(config: &PostgresConfig) -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET search_path TO public")
                    .execute(conn)
                    .await
                    .map(|_| ())
            })
        })
        .connect(&config.connection_url())
        .await
        .expect("postgres pool")
}

#[tokio::test]
async fn empty_mismatch_recreates_without_allow_flag() {
    let Some(base) = config_or_skip("default") else {
        return;
    };
    let suffix = &Uuid::new_v4().to_string().replace('-', "")[..12];
    let config = base.with_namespace(format!("dim_empty_{suffix}"));
    let raw = pool(&config).await;
    let shared = PostgresPool::from_existing(raw.clone(), config.clone());

    let store_a = PgVectorStorage::with_pool_and_dimension(shared.clone(), config.clone(), DIM_A);
    store_a.initialize().await.expect("create dim A table");
    // Empty table at DIM_A — switch required to DIM_B without ALLOW.
    let store_b = PgVectorStorage::with_pool_and_dimension(shared, config, DIM_B);
    let outcome = store_b
        .reconcile_dimension(DIM_B, DimensionReconcilePolicy::FailClosed)
        .await
        .expect("empty recreate");
    assert_eq!(outcome, DimensionEnsureOutcome::Recreated);
    assert_eq!(store_b.get_stored_dimension().await.unwrap(), Some(DIM_B));
}

#[tokio::test]
async fn nonempty_fail_closed_refuses_drop() {
    let Some(base) = config_or_skip("default") else {
        return;
    };
    let suffix = &Uuid::new_v4().to_string().replace('-', "")[..12];
    let config = base.with_namespace(format!("dim_fc_{suffix}"));
    let raw = pool(&config).await;
    let shared = PostgresPool::from_existing(raw.clone(), config.clone());

    let store_a = PgVectorStorage::with_pool_and_dimension(shared.clone(), config.clone(), DIM_A);
    store_a.initialize().await.expect("init");
    store_a
        .upsert(&[("keep-me".into(), vec![0.1; DIM_A], serde_json::json!({}))])
        .await
        .expect("seed row");

    let store_b = PgVectorStorage::with_pool_and_dimension(shared, config, DIM_B);
    let err = store_b
        .reconcile_dimension(DIM_B, DimensionReconcilePolicy::FailClosed)
        .await
        .expect_err("must refuse wipe");
    let msg = err.to_string();
    assert!(msg.contains("Refusing DROP TABLE"), "{msg}");
    assert_eq!(store_b.get_stored_dimension().await.unwrap(), Some(DIM_A));
    assert_eq!(store_a.count().await.unwrap(), 1);
}

#[tokio::test]
async fn nonempty_prefer_existing_keeps_schema() {
    let Some(base) = config_or_skip("default") else {
        return;
    };
    let suffix = &Uuid::new_v4().to_string().replace('-', "")[..12];
    let config = base.with_namespace(format!("dim_pe_{suffix}"));
    let raw = pool(&config).await;
    let shared = PostgresPool::from_existing(raw.clone(), config.clone());

    let store_a = PgVectorStorage::with_pool_and_dimension(shared.clone(), config.clone(), DIM_A);
    store_a.initialize().await.expect("init");
    store_a
        .upsert(&[("keep-me".into(), vec![0.2; DIM_A], serde_json::json!({}))])
        .await
        .expect("seed row");

    let store_b = PgVectorStorage::with_pool_and_dimension(shared, config, DIM_B);
    let outcome = store_b
        .reconcile_dimension(DIM_B, DimensionReconcilePolicy::PreferExisting)
        .await
        .expect("prefer existing");
    assert_eq!(
        outcome,
        DimensionEnsureOutcome::KeptExisting {
            stored: DIM_A,
            required: DIM_B
        }
    );
    assert_eq!(store_b.get_stored_dimension().await.unwrap(), Some(DIM_A));
    assert_eq!(store_a.count().await.unwrap(), 1);
}
