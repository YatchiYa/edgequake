//! SPEC-091 Doc 23 KVH-AC-04 / AC-07 / health zero-SQL: honest `/health` SSOT.

#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use axum::extract::State;
use edgequake_api::handlers::health::health_check;
use edgequake_api::state::AppState;
use edgequake_storage::adapters::postgres::PostgresKVStorage;
use edgequake_storage::traits::KVStorage;
use edgequake_storage::PostgresConfig;
use serial_test::serial;
use sqlx::PgPool;

fn base_url() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .or_else(|| {
            std::fs::read_to_string("/tmp/edgequake-db-url")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
}

fn allow_mock_provider() {
    // AppState::new_postgres refuses mock as the default server LLM unless opted in.
    std::env::set_var("EDGEQUAKE_ALLOW_MOCK_PROVIDER", "1");
    std::env::set_var("EDGEQUAKE_LLM_PROVIDER", "mock");
}

fn kv_config_from_url(url: &str) -> PostgresConfig {
    let without = url
        .trim()
        .strip_prefix("postgresql://")
        .or_else(|| url.trim().strip_prefix("postgres://"))
        .unwrap_or(url.trim());
    let (creds, hostdb) = without
        .split_once('@')
        .expect("DATABASE_URL must include user@host");
    let (user, pass) = creds.split_once(':').unwrap_or((creds, ""));
    let (hostport, db) = hostdb
        .split_once('/')
        .expect("DATABASE_URL must include /database");
    let db = db.split('?').next().unwrap_or(db);
    let (host, port) = match hostport.split_once(':') {
        Some((h, p)) => (h, p.parse().unwrap_or(5432)),
        None => (hostport, 5432u16),
    };
    PostgresConfig {
        host: host.to_string(),
        port,
        database: db.to_string(),
        user: user.to_string(),
        password: pass.to_string(),
        namespace: "default".to_string(),
        ..Default::default()
    }
}

#[tokio::test]
#[serial]
async fn contract_spec091_health_chunk_text_ssot_relational() {
    allow_mock_provider();
    let Some(base) = base_url() else {
        eprintln!("SKIP: no DATABASE_URL");
        return;
    };
    let url = test_db::isolated_test_url(&base);
    let pool = PgPool::connect(&url).await.expect("connect");
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&pool)
        .await;
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS age")
        .execute(&pool)
        .await;

    std::env::remove_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY");
    let state = AppState::new_postgres(&url, "").await.expect("AppState");
    let response = health_check(State(state)).await.expect("health").0;
    let operational = response
        .operational
        .as_ref()
        .expect("operational snapshot on postgres AppState");
    let storage = &operational.storage;
    assert_ne!(
        storage.chunk_text_ssot, "kv",
        "default authority is relational — health must not hardcode kv"
    );
    assert_eq!(storage.chunk_text_ssot, "relational");
    assert!(
        !storage.chunk_kv_in_persister,
        "relational authority ⇒ chunk_kv_in_persister false"
    );
    assert_eq!(
        operational.read_model.merge_strategy,
        edgequake_api::document_read_model::MERGE_STRATEGY
    );
    assert!(
        edgequake_api::document_read_model::MERGE_STRATEGY.contains("relational"),
        "MERGE_STRATEGY must be relational-primary post-drop"
    );

    pool.close().await;
}

#[tokio::test]
#[serial]
async fn e2e_spec091_health_no_kv_sql_post_drop() {
    allow_mock_provider();
    let Some(base) = base_url() else {
        eprintln!("SKIP: no DATABASE_URL");
        return;
    };
    let url = test_db::isolated_test_url(&base);
    let state = AppState::new_postgres(&url, "")
        .await
        .expect("AppState seeds Absent from cutover census");
    let response = health_check(State(state)).await.expect("health").0;
    assert_eq!(
        response
            .operational
            .as_ref()
            .map(|o| o.storage.chunk_text_ssot.as_str()),
        Some("relational")
    );

    let kv = PostgresKVStorage::new(kv_config_from_url(&url));
    kv.seed_relation_from_dropped(true);
    kv.reset_kv_raw_sql_attempts();
    for _ in 0..10 {
        kv.ping().await.expect("ping");
    }
    assert_eq!(
        kv.kv_raw_sql_attempts(),
        0,
        "deep health / ping after drop must not SQL eq_*_kv"
    );
}
