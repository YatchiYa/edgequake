//! SPEC-101 — setup status + initialize idempotency (requires postgres).
//!
//! Run:
//!   cargo test -p edgequake-api --test e2e_spec101_setup --features postgres

#![cfg(feature = "postgres")]

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

use edgequake_api::state::migration_bootstrap::run_postgres_migrations;
use edgequake_api::{AppState, Server, ServerConfig};

fn server_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: true,
    }
}

fn build_app(state: AppState) -> axum::Router {
    Server::new(server_config(), state).build_router()
}

async fn connect_pool() -> Option<sqlx::PgPool> {
    let database_url = common::spec013_postgres::try_database_url()?;
    let pool = match PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("SKIP spec101 setup: connect failed: {error}");
            return None;
        }
    };
    if run_postgres_migrations(&pool).await.is_err() {
        eprintln!("SKIP spec101 setup: migrations failed");
        return None;
    }
    Some(pool)
}

#[tokio::test]
async fn setup_status_is_public_and_shaped() {
    let pool = match connect_pool().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP setup_status_is_public_and_shaped: DATABASE_URL not set");
            return;
        }
    };

    let mut state = AppState::test_state_with_pg_pool(pool);
    state.auth.config.auth_enabled = true;
    state.auth.config.dev_mode = false;
    let app = build_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("status");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(body.get("needs_setup").is_some());
    assert!(body.get("tenant_count").is_some());
    assert!(body.get("has_login_users").is_some());
}

#[tokio::test]
async fn setup_initialize_then_409_on_repeat() {
    let pool = match connect_pool().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP setup_initialize_then_409_on_repeat: DATABASE_URL not set");
            return;
        }
    };

    // Isolate: use a fresh schema is hard; instead only run initialize when needs_setup.
    let mut state = AppState::test_state_with_pg_pool(pool);
    state.auth.config.auth_enabled = true;
    state.auth.config.dev_mode = false;
    // Clear bootstrap env so initialize requires password
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD");

    let app = build_app(state.clone());
    let status_res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_bytes = axum::body::to_bytes(status_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&status_bytes).unwrap();

    if status["needs_setup"] != true {
        eprintln!("SKIP initialize: DB already has tenants/users (needs_setup=false)");
        return;
    }

    let username = format!("spec101_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let app = build_app(state.clone());
    let create_res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/setup/initialize")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "admin_username": username,
                        "admin_password": "SecurePass123!",
                        "tenant_name": format!("Org {username}"),
                        "workspace_name": "Main Workspace"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        create_res.status(),
        StatusCode::CREATED,
        "initialize should succeed on empty install"
    );

    let app = build_app(state);
    let repeat = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/setup/initialize")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "admin_username": "other",
                        "admin_password": "SecurePass123!",
                        "tenant_name": "Other",
                        "workspace_name": "Other"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(repeat.status(), StatusCode::CONFLICT);
}
