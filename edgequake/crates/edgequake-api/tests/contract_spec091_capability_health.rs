//! SPEC-091 IW4 (GAP-091-32, LAW-I6) — `/health` capability matrix matches probe SSOT.
//!
//! Proves `schema.postgres_capabilities` on `/health` is derived from
//! `PostgresCapabilityProbe` in `edgequake-storage` (no second version gate).
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake_test:test_password_123@localhost:5432/edgequake_test \
//!     cargo test -p edgequake-api --features postgres --test contract_spec091_capability_health -- --nocapture

#![cfg(feature = "postgres")]

use axum::body::Body;
use axum::http::Request;
use edgequake_api::handlers::health::{derive_postgres_capability_health, health_check};
use edgequake_api::handlers::health_types::PostgresCapabilityHealth;
use edgequake_api::AppState;
use edgequake_storage::adapters::postgres::PostgresCapabilityProbe;
use serial_test::serial;
use sqlx::PgPool;
use tower::ServiceExt;

#[path = "common/test_db.rs"]
mod test_db;

fn with_chunk_embeddings_backend() {
    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "chunk_embeddings");
}

fn base_url() -> Option<String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return Some(url);
        }
    }
    let password = std::env::var("POSTGRES_PASSWORD").ok()?;
    let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "edgequake".to_string());
    let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "edgequake".to_string());
    Some(format!("postgresql://{user}:{password}@{host}:{port}/{db}"))
}

async fn ensure_extensions(pool: &PgPool) {
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS age")
        .execute(pool)
        .await;
}

#[tokio::test]
#[serial]
async fn contract_spec091_capability_probe_roundtrip() {
    with_chunk_embeddings_backend();
    let Some(base) = base_url() else {
        eprintln!("SKIP contract_spec091_capability_probe_roundtrip: no DATABASE_URL");
        return;
    };
    let url = test_db::isolated_test_url(&base);
    let pool = PgPool::connect(&url).await.expect("connect scratch db");
    ensure_extensions(&pool).await;

    let live = PostgresCapabilityProbe::detect(&pool).await;
    assert!(
        live.postgres_major >= 16,
        "expected PG16+ test rig, got major={}",
        live.postgres_major
    );
    assert!(
        live.pgvector_version.is_some(),
        "pgvector extversion must be present"
    );
    assert!(
        live.iterative_scan_available,
        "CI images pin pgvector ≥0.8.5; iterative_scan must be available (got {:?})",
        live.pgvector_version
    );

    let expected: PostgresCapabilityHealth = live.clone().into();
    let state = AppState::new_postgres(&url, "")
        .await
        .expect("postgres AppState");
    let derived = derive_postgres_capability_health(&state)
        .expect("postgres_capabilities on migrated AppState");
    assert_eq!(
        derived, expected,
        "derive_postgres_capability_health must match probe"
    );

    pool.close().await;
}

#[tokio::test]
#[serial]
async fn contract_spec091_health_endpoint_capability_matrix() {
    with_chunk_embeddings_backend();
    let Some(base) = base_url() else {
        eprintln!("SKIP contract_spec091_health_endpoint_capability_matrix: no DATABASE_URL");
        return;
    };
    let url = test_db::isolated_test_url(&base);
    let pool = PgPool::connect(&url).await.expect("connect scratch db");
    ensure_extensions(&pool).await;
    let expected: PostgresCapabilityHealth = PostgresCapabilityProbe::detect(&pool).await.into();

    let state = AppState::new_postgres(&url, "")
        .await
        .expect("postgres AppState");
    let response = health_check(axum::extract::State(state))
        .await
        .expect("health_check")
        .0;
    let schema = response
        .schema
        .expect("/health.schema on postgres AppState");
    let caps = schema
        .postgres_capabilities
        .expect("schema.postgres_capabilities must be populated on postgres");
    assert_eq!(
        caps, expected,
        "/health schema matrix must match live probe"
    );

    pool.close().await;
}

#[tokio::test]
#[serial]
async fn contract_spec091_health_http_json_capability_matrix() {
    with_chunk_embeddings_backend();
    let Some(base) = base_url() else {
        eprintln!("SKIP contract_spec091_health_http_json_capability_matrix: no DATABASE_URL");
        return;
    };
    let url = test_db::isolated_test_url(&base);
    let pool = PgPool::connect(&url).await.expect("connect scratch db");
    ensure_extensions(&pool).await;
    let expected: PostgresCapabilityHealth = PostgresCapabilityProbe::detect(&pool).await.into();

    let state = AppState::new_postgres(&url, "")
        .await
        .expect("postgres AppState");
    let app = edgequake_api::Server::new(
        edgequake_api::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            enable_cors: false,
            enable_compression: false,
            enable_swagger: false,
        },
        state,
    )
    .build_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("GET /health");
    assert!(response.status().is_success());
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    let node = &json["schema"]["postgres_capabilities"];
    assert!(
        !node.is_null(),
        "schema.postgres_capabilities must be present: {json}"
    );
    let caps: PostgresCapabilityHealth =
        serde_json::from_value(node.clone()).expect("deserialize capability matrix");
    assert_eq!(caps, expected, "HTTP /health JSON must match live probe");

    pool.close().await;
}
