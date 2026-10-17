//! Endpoint sweep F10: DELETE of a nonexistent tenant must be 404 (documented), not 204.
//!
//! Run: `cargo test -p edgequake-api --test e2e_tenant_delete_not_found`

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::json;
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

async fn delete_tenant(app: &axum::Router, id: &str) -> StatusCode {
    app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/tenants/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn delete_missing_tenant_is_404_existing_is_204() {
    let app = test_app();
    assert_eq!(
        delete_tenant(&app, &Uuid::new_v4().to_string()).await,
        StatusCode::NOT_FOUND
    );

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"name": "F10 tenant", "slug": format!("f10-{}", Uuid::new_v4()), "plan": "free"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create tenant: {}",
        resp.status()
    );
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let id = body["id"].as_str().expect("tenant id");

    assert_eq!(delete_tenant(&app, id).await, StatusCode::NO_CONTENT);
    assert_eq!(delete_tenant(&app, id).await, StatusCode::NOT_FOUND);
}
