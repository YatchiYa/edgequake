//! POST /api/v1/auth/switch-scope — re-bind a password session to another workspace.
//!
//! Requires: `DATABASE_URL` and `--features postgres`
//!
//! Run:
//!   cargo test -p edgequake-api --test e2e_switch_scope --features postgres

#![cfg(feature = "postgres")]

mod common;

use axum::body::Body;
use axum::http::{header, HeaderMap, Request, StatusCode};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

use edgequake_api::services::auth_bootstrap::bootstrap_auth_identity_if_needed;
use edgequake_api::state::migration_bootstrap::run_postgres_migrations;
use edgequake_api::{AppState, Server, ServerConfig};

const DEFAULT_TENANT: &str = "00000000-0000-0000-0000-000000000002";
const PASSWORD: &str = "SecurePass123!";

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    headers: &[(&str, String)],
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    for (k, v) in headers {
        builder = builder.header(*k, v);
    }
    let body = match body {
        Some(b) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .unwrap();
    (
        status,
        headers,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn refresh_cookie(headers: &HeaderMap) -> String {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|c| c.starts_with("eq_refresh="))
        .and_then(|c| c.split(';').next())
        .expect("refresh cookie")
        .to_string()
}

/// (access token, `eq_refresh=…` cookie pair)
async fn login(app: &axum::Router, username: &str) -> (String, String) {
    let (status, headers, body) = call(
        app,
        "POST",
        "/api/v1/auth/login",
        &[],
        Some(json!({ "username": username, "password": PASSWORD })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "login {username}: {body}");
    (
        body["access_token"].as_str().unwrap().to_string(),
        refresh_cookie(&headers),
    )
}

fn bearer(token: &str, workspace: Option<&str>) -> Vec<(&'static str, String)> {
    let mut h = vec![
        ("authorization", format!("Bearer {token}")),
        ("x-tenant-id", DEFAULT_TENANT.to_string()),
    ];
    if let Some(w) = workspace {
        h.push(("x-workspace-id", w.to_string()));
    }
    h
}

#[tokio::test]
async fn switch_scope_rebinds_session_and_enforces_membership() {
    let Some(url) = common::spec013_postgres::try_database_url() else {
        eprintln!("SKIP switch_scope e2e: DATABASE_URL not set");
        return;
    };
    let Ok(pool) = PgPoolOptions::new().max_connections(3).connect(&url).await else {
        eprintln!("SKIP switch_scope e2e: connect failed");
        return;
    };
    run_postgres_migrations(&pool).await.expect("migrations");

    let admin = format!("scope_admin_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    std::env::set_var("EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME", &admin);
    std::env::set_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD", PASSWORD);
    std::env::set_var(
        "EDGEQUAKE_BOOTSTRAP_ADMIN_EMAIL",
        format!("{admin}@example.com"),
    );

    let mut state = AppState::test_state_with_pg_pool(pool.clone());
    // Memberships/workspaces must come from PostgreSQL, as in production.
    state.workspace_service =
        std::sync::Arc::new(edgequake_core::WorkspaceServiceImpl::new(pool.clone()));
    state.auth.config.auth_enabled = true;
    state.auth.config.dev_mode = false;
    state.security.kv_identity_mirror = false;
    state.initialize_defaults().await.expect("defaults");
    bootstrap_auth_identity_if_needed(&state)
        .await
        .expect("bootstrap");
    let app = Server::new(
        ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
            enable_cors: false,
            enable_compression: false,
            enable_swagger: false,
        },
        state,
    )
    .build_router();

    let (admin_token, admin_cookie) = login(&app, &admin).await;
    let slug = format!("scope-e2e-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (status, _, ws) = call(
        &app,
        "POST",
        &format!("/api/v1/tenants/{DEFAULT_TENANT}/workspaces"),
        &bearer(&admin_token, None),
        Some(json!({ "name": slug, "slug": slug })),
    )
    .await;
    assert!(status.is_success(), "create workspace: {status} {ws}");
    let workspace = ws["id"].as_str().unwrap().to_string();

    // Upstream #498: a platform admin selects any active scope by header, past the claim pin.
    let (status, _, _) = call(
        &app,
        "GET",
        "/api/v1/documents",
        &bearer(&admin_token, Some(&workspace)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let switch_body = json!({ "tenant_id": DEFAULT_TENANT, "workspace_id": workspace });
    // A bare access token is not enough (no refresh family → no session minting).
    let (status, _, _) = call(
        &app,
        "POST",
        "/api/v1/auth/switch-scope",
        &[("authorization", format!("Bearer {admin_token}"))],
        Some(switch_body.clone()),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "switch without refresh cookie"
    );

    // Platform admin switches its own session family to the workspace (no membership row).
    let (status, headers, switched) = call(
        &app,
        "POST",
        "/api/v1/auth/switch-scope",
        &[
            ("authorization", format!("Bearer {admin_token}")),
            ("cookie", admin_cookie.clone()),
        ],
        Some(switch_body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "switch: {switched}");
    assert_eq!(switched["workspace_id"], workspace);
    let scoped = switched["access_token"].as_str().unwrap().to_string();
    let (status, _, _) = call(
        &app,
        "GET",
        "/api/v1/documents",
        &bearer(&scoped, Some(&workspace)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "switched token reads its workspace");

    // Refresh keeps the switched scope.
    let cookie = refresh_cookie(&headers);
    let (status, refreshed_headers, refreshed) = call(
        &app,
        "POST",
        "/api/v1/auth/refresh",
        &[("cookie", cookie)],
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "refresh: {refreshed}");
    let renewed = refreshed["access_token"].as_str().unwrap();
    let (status, _, _) = call(
        &app,
        "GET",
        "/api/v1/documents",
        &bearer(renewed, Some(&workspace)),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "refreshed token stays in the switched workspace"
    );

    // A plain user without membership is refused.
    let member = format!("scope_user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (status, _, created) = call(
        &app,
        "POST",
        "/api/v1/users",
        &bearer(&admin_token, None),
        Some(json!({ "username": member, "email": format!("{member}@example.com"), "password": PASSWORD, "role": "user" })),
    )
    .await;
    assert!(status.is_success(), "create user: {status} {created}");
    let (user_token, user_cookie) = login(&app, &member).await;
    // For everyone else the default-bound token still cannot read the new workspace.
    let (status, _, _) = call(
        &app,
        "GET",
        "/api/v1/documents",
        &bearer(&user_token, Some(&workspace)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = call(
        &app,
        "POST",
        "/api/v1/auth/switch-scope",
        &[
            ("authorization", format!("Bearer {user_token}")),
            ("cookie", user_cookie.clone()),
        ],
        Some(switch_body.clone()),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "non-member must not enter the workspace"
    );
    // A refused switch must not consume the session (authorize before rotating).
    let (status, _, _) = call(
        &app,
        "POST",
        "/api/v1/auth/refresh",
        &[("cookie", user_cookie)],
        Some(json!({})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "refused switch keeps the user's session"
    );

    // Logout ends the switched session family.
    let live_cookie = refresh_cookie(&refreshed_headers);
    call(
        &app,
        "POST",
        "/api/v1/auth/logout",
        &[
            ("authorization", format!("Bearer {renewed}")),
            ("cookie", live_cookie.clone()),
        ],
        Some(json!({})),
    )
    .await;
    let (status, _, _) = call(
        &app,
        "POST",
        "/api/v1/auth/refresh",
        &[("cookie", live_cookie)],
        Some(json!({})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "logout revokes the switched family"
    );
    // The pre-switch refresh token was rotated by the switch (not cloned into a 2nd session).
    let (status, _, _) = call(
        &app,
        "POST",
        "/api/v1/auth/switch-scope",
        &[
            ("authorization", format!("Bearer {admin_token}")),
            ("cookie", admin_cookie.clone()),
        ],
        Some(switch_body.clone()),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "rotated refresh token must not be reusable"
    );

    // Cleanup.
    let user_id = created["user_id"]
        .as_str()
        .or(created["user"]["user_id"].as_str())
        .map(str::to_string);
    if let Some(id) = user_id {
        call(
            &app,
            "DELETE",
            &format!("/api/v1/users/{id}"),
            &bearer(&admin_token, None),
            None,
        )
        .await;
    }
    call(
        &app,
        "DELETE",
        &format!("/api/v1/workspaces/{workspace}"),
        &bearer(&scoped, Some(&workspace)),
        None,
    )
    .await;
    for username in [&admin, &member] {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&pool)
            .await;
    }
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME");
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD");
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_EMAIL");
}
