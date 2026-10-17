//! /api/v1/tenants/{tenant_id}/workspace-members — membership management drives real isolation.
//!
//! Requires: `DATABASE_URL` and `--features postgres`
//!
//! Run:
//!   cargo test -p edgequake-api --test e2e_tenant_members --features postgres

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

fn auth(token: &str, tenant: &str, workspace: Option<&str>) -> Vec<(&'static str, String)> {
    let mut h = vec![
        ("authorization", format!("Bearer {token}")),
        ("x-tenant-id", tenant.to_string()),
    ];
    if let Some(w) = workspace {
        h.push(("x-workspace-id", w.to_string()));
    }
    h
}

/// Switch the user's session into `workspace`; returns (status, scoped token, refresh cookie).
async fn switch(
    app: &axum::Router,
    token: &str,
    cookie: &str,
    tenant: &str,
    workspace: &str,
) -> (StatusCode, Option<(String, String)>) {
    let (status, headers, body) = call(
        app,
        "POST",
        "/api/v1/auth/switch-scope",
        &[
            ("authorization", format!("Bearer {token}")),
            ("cookie", cookie.to_string()),
        ],
        Some(json!({ "tenant_id": tenant, "workspace_id": workspace })),
    )
    .await;
    if status != StatusCode::OK {
        return (status, None);
    }
    (
        status,
        Some((
            body["access_token"].as_str().unwrap().to_string(),
            refresh_cookie(&headers),
        )),
    )
}

/// Workspace ids the caller can list in `tenant` (None = tenant itself refused).
async fn visible_workspaces(app: &axum::Router, token: &str, tenant: &str) -> Option<Vec<String>> {
    let (status, _, body) = call(
        app,
        "GET",
        &format!("/api/v1/tenants/{tenant}/workspaces"),
        &auth(token, DEFAULT_TENANT, None),
        None,
    )
    .await;
    if status == StatusCode::FORBIDDEN {
        return None;
    }
    assert_eq!(status, StatusCode::OK, "list workspaces: {body}");
    Some(
        body["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w["id"].as_str().unwrap().to_string())
            .collect(),
    )
}

#[tokio::test]
async fn members_api_grants_and_revokes_workspace_isolation() {
    let Some(url) = common::spec013_postgres::try_database_url() else {
        eprintln!("SKIP tenant members e2e: DATABASE_URL not set");
        return;
    };
    let Ok(pool) = PgPoolOptions::new().max_connections(3).connect(&url).await else {
        eprintln!("SKIP tenant members e2e: connect failed");
        return;
    };
    run_postgres_migrations(&pool).await.expect("migrations");

    let tag = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let admin = format!("members_admin_{tag}");
    std::env::set_var("EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME", &admin);
    std::env::set_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD", PASSWORD);
    std::env::set_var(
        "EDGEQUAKE_BOOTSTRAP_ADMIN_EMAIL",
        format!("{admin}@example.com"),
    );

    let mut state = AppState::test_state_with_pg_pool(pool.clone());
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

    let (admin_token, _) = login(&app, &admin).await;
    let admin_h = auth(&admin_token, DEFAULT_TENANT, None);

    // Fresh tenant with two workspaces X and Y.
    let (status, _, tenant) = call(
        &app,
        "POST",
        "/api/v1/tenants",
        &admin_h,
        Some(json!({ "name": format!("Members {tag}"), "slug": format!("members-{tag}") })),
    )
    .await;
    assert!(status.is_success(), "create tenant: {status} {tenant}");
    let t = tenant["id"].as_str().unwrap().to_string();
    let mut ws = Vec::new();
    for name in ["x", "y"] {
        let slug = format!("members-{tag}-{name}");
        let (status, _, body) = call(
            &app,
            "POST",
            &format!("/api/v1/tenants/{t}/workspaces"),
            &admin_h,
            Some(json!({ "name": slug, "slug": slug })),
        )
        .await;
        assert!(status.is_success(), "create workspace: {status} {body}");
        ws.push(body["id"].as_str().unwrap().to_string());
    }
    let (x, y) = (ws[0].clone(), ws[1].clone());

    let names: Vec<String> = ["alice", "bob", "carol", "dave"]
        .iter()
        .map(|n| format!("m_{n}_{tag}"))
        .collect();
    for name in &names {
        let (status, _, body) = call(
            &app,
            "POST",
            "/api/v1/users",
            &admin_h,
            Some(json!({ "username": name, "email": format!("{name}@example.com"), "password": PASSWORD, "role": "user" })),
        )
        .await;
        assert!(status.is_success(), "create user {name}: {status} {body}");
    }
    let (alice, bob, carol, dave) = (&names[0], &names[1], &names[2], &names[3]);
    let members = format!("/api/v1/tenants/{t}/workspace-members");

    // Platform admin grants: alice→X, bob→Y (by email), carol tenant admin, dave tenant owner.
    let mut ids = std::collections::HashMap::new();
    for (user, role, workspace) in [
        (alice.clone(), "member", Some(x.clone())),
        (format!("{bob}@example.com"), "member", Some(y.clone())),
        (carol.clone(), "admin", None),
        (dave.clone(), "owner", None),
    ] {
        let (status, _, body) = call(
            &app,
            "POST",
            &members,
            &admin_h,
            Some(json!({ "user": user, "role": role, "workspace_id": workspace })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "add {user}: {body}");
        ids.insert(
            body["username"].as_str().unwrap().to_string(),
            body["membership_id"].as_str().unwrap().to_string(),
        );
    }
    // Same (user, tenant, workspace) again → role update, 200.
    let (status, _, body) = call(
        &app,
        "POST",
        &members,
        &admin_h,
        Some(json!({ "user": alice, "role": "member", "workspace_id": x })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "re-add is an update: {body}");
    assert_eq!(body["membership_id"].as_str().unwrap(), ids[alice]);
    // Validation: workspace of another tenant, unknown user.
    let (status, _, _) = call(&app, "POST", &members, &admin_h, Some(json!({ "user": alice, "role": "member", "workspace_id": "00000000-0000-0000-0000-000000000003" }))).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "foreign workspace rejected"
    );
    let (status, _, _) = call(
        &app,
        "POST",
        &members,
        &admin_h,
        Some(json!({ "user": format!("nobody_{tag}"), "role": "member" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown user rejected");

    let (status, _, list) = call(&app, "GET", &members, &admin_h, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["items"].as_array().unwrap().len(), 4, "{list}");
    let alice_row = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["username"] == alice.as_str())
        .unwrap();
    assert_eq!(alice_row["workspace_id"], x.as_str());
    assert_eq!(alice_row["workspace_name"], format!("members-{tag}-x"));

    // Isolation: alice in X only, bob in Y only.
    for (user, own, other) in [(alice, &x, &y), (bob, &y, &x)] {
        let (token, cookie) = login(&app, user).await;
        assert_eq!(
            visible_workspaces(&app, &token, &t).await,
            Some(vec![own.clone()]),
            "{user} sees only own workspace"
        );
        let (status, _, _) = call(
            &app,
            "GET",
            &members,
            &auth(&token, DEFAULT_TENANT, None),
            None,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "plain member cannot manage members"
        );
        let (status, scoped) = switch(&app, &token, &cookie, &t, own).await;
        assert_eq!(status, StatusCode::OK, "{user} switches to own workspace");
        let (scoped_token, scoped_cookie) = scoped.unwrap();
        let (status, _, _) = call(
            &app,
            "GET",
            "/api/v1/documents",
            &auth(&scoped_token, &t, Some(own)),
            None,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{user} reads own workspace documents"
        );
        let (status, _, _) = call(
            &app,
            "GET",
            "/api/v1/documents",
            &auth(&scoped_token, &t, Some(other)),
            None,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{user} cannot read the other workspace"
        );
        let (status, _) = switch(&app, &scoped_token, &scoped_cookie, &t, other).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{user} cannot switch into the other workspace"
        );

        if user == alice {
            // Revocation is immediate: next request and next refresh fail.
            let (status, _, _) = call(
                &app,
                "DELETE",
                &format!("{members}/{}", ids[alice]),
                &admin_h,
                None,
            )
            .await;
            assert_eq!(status, StatusCode::NO_CONTENT);
            let (status, _, _) = call(
                &app,
                "GET",
                "/api/v1/documents",
                &auth(&scoped_token, &t, Some(own)),
                None,
            )
            .await;
            assert_eq!(
                status,
                StatusCode::FORBIDDEN,
                "removed member's live token is refused"
            );
            let (status, _, body) = call(
                &app,
                "POST",
                "/api/v1/auth/refresh",
                &[("cookie", scoped_cookie)],
                Some(json!({})),
            )
            .await;
            assert!(
                !status.is_success(),
                "refresh in a revoked scope must fail: {status} {body}"
            );
            assert_eq!(
                visible_workspaces(&app, &token, &t).await,
                None,
                "ex-member no longer sees the tenant"
            );
        }
    }

    // Tenant admin carol manages members but cannot touch owners.
    let (carol_token, _) = login(&app, carol).await;
    let carol_h = auth(&carol_token, DEFAULT_TENANT, None);
    let (status, _, list) = call(&app, "GET", &members, &carol_h, None).await;
    assert_eq!(status, StatusCode::OK, "tenant admin lists members: {list}");
    assert_eq!(list["can_manage_owners"], false);
    let (status, _, _) = call(
        &app,
        "POST",
        &members,
        &carol_h,
        Some(json!({ "user": alice, "role": "owner" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "tenant admin cannot grant owner"
    );
    let (status, _, _) = call(
        &app,
        "PATCH",
        &format!("{members}/{}", ids[dave]),
        &carol_h,
        Some(json!({ "role": "member" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "tenant admin cannot demote an owner"
    );
    let (status, _, body) = call(
        &app,
        "PATCH",
        &format!("{members}/{}", ids[bob]),
        &carol_h,
        Some(json!({ "role": "readonly" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "tenant admin changes a member role: {body}"
    );
    assert_eq!(body["role"], "readonly");

    // The last owner can be neither removed nor demoted — even by a platform admin.
    let dave_url = format!("{members}/{}", ids[dave]);
    let (status, _, _) = call(&app, "DELETE", &dave_url, &admin_h, None).await;
    assert_eq!(status, StatusCode::CONFLICT, "last owner cannot be removed");
    let (status, _, _) = call(
        &app,
        "PATCH",
        &dave_url,
        &admin_h,
        Some(json!({ "role": "admin" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "last owner cannot be demoted");
    // Owner dave may promote carol; then dave can leave.
    let (dave_token, _) = login(&app, dave).await;
    let (status, _, _) = call(
        &app,
        "PATCH",
        &format!("{members}/{}", ids[carol]),
        &auth(&dave_token, DEFAULT_TENANT, None),
        Some(json!({ "role": "owner" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "owner grants owner");
    let (status, _, _) = call(&app, "DELETE", &dave_url, &admin_h, None).await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "non-last owner can be removed"
    );

    // Cleanup.
    let _ = sqlx::query("DELETE FROM memberships WHERE tenant_id = $1::uuid")
        .bind(&t)
        .execute(&pool)
        .await;
    let (status, _, body) = call(
        &app,
        "DELETE",
        &format!("/api/v1/tenants/{t}"),
        &admin_h,
        None,
    )
    .await;
    assert!(status.is_success(), "delete tenant: {status} {body}");
    for sql in [
        "DELETE FROM workspaces WHERE tenant_id = $1::uuid",
        "DELETE FROM tenants WHERE tenant_id = $1::uuid",
    ] {
        let _ = sqlx::query(sql).bind(&t).execute(&pool).await;
    }
    for username in names.iter().chain([&admin]) {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&pool)
            .await;
    }
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME");
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD");
    std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_EMAIL");
}
