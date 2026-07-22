//! SPEC-067 — max_documents admission fail-closed (HTTP upload → 409).

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_core::types::CreateWorkspaceRequest;
use edgequake_core::Tenant;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn e2e_max_documents_rejects_second_upload() {
    std::env::set_var("EDGEQUAKE_DEV_MODE", "true");
    std::env::set_var("EDGEQUAKE_AUTH_ENABLED", "false");

    let state = AppState::test_state();
    let tenant = Tenant::new("Quota Tenant", format!("quota-{}", Uuid::new_v4()));
    let tenant = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .expect("tenant");
    let ws = state
        .workspace_service
        .create_workspace(
            tenant.tenant_id,
            CreateWorkspaceRequest {
                name: "quota-ws".into(),
                slug: Some(format!("quota-{}", Uuid::new_v4())),
                description: None,
                max_documents: Some(1),
                ..Default::default()
            },
        )
        .await
        .expect("workspace");

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

    let tid = tenant.tenant_id.to_string();
    let wid = ws.workspace_id.to_string();

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", &tid)
                .header("X-Workspace-ID", &wid)
                .body(Body::from(
                    json!({
                        "content": "first document body",
                        "title": "doc-1"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        first.status() == StatusCode::CREATED || first.status() == StatusCode::ACCEPTED,
        "first upload status={}",
        first.status()
    );

    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", &tid)
                .header("X-Workspace-ID", &wid)
                .body(Body::from(
                    json!({
                        "content": "second document should be rejected",
                        "title": "doc-2"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        second.status(),
        StatusCode::CONFLICT,
        "second upload must hit max_documents"
    );
    let body = axum::body::to_bytes(second.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("quota exceeded") || text.contains("max_documents"),
        "body={text}"
    );
}
