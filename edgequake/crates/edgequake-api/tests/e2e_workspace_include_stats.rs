//! Workspace list `?include_stats=` opt-in (PR #389).
//!
//! Run: `cargo test -p edgequake-api --test e2e_workspace_include_stats`

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn test_app() -> axum::Router {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: false,
    };
    Server::new(config, AppState::test_state()).build_router()
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(json!({}))
}

async fn create_tenant(app: &axum::Router) -> String {
    let slug = format!("stats-{}", Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Include Stats Tenant",
                        "slug": slug,
                        "plan": "free"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::CREATED || resp.status() == StatusCode::OK,
        "create tenant: {}",
        resp.status()
    );
    json_body(resp).await["id"]
        .as_str()
        .expect("tenant id")
        .to_string()
}

#[tokio::test]
async fn list_workspaces_omits_stats_by_default() {
    let app = test_app();
    let tenant_id = create_tenant(&app).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty(), "{body}");
    for item in items {
        assert!(
            item.get("stats").is_none(),
            "default list must omit stats: {item}"
        );
    }
}

#[tokio::test]
async fn list_workspaces_include_stats_does_not_fail() {
    let app = test_app();
    let tenant_id = create_tenant(&app).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/tenants/{tenant_id}/workspaces?include_stats=true"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "include_stats must not 500");
    let body = json_body(resp).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty(), "{body}");
    for item in items {
        if let Some(stats) = item.get("stats") {
            assert!(
                stats.is_object() || stats.is_null(),
                "stats must be object or null: {item}"
            );
        }
    }
}
