//! Integration tests for workspace-scoped document listing.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use edgequake_api::middleware::{default_tenant_uuid, default_workspace_uuid};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::json;
use tower::ServiceExt;

fn create_test_app(state: AppState) -> axum::Router {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: true,
    };
    let server = Server::new(config, state);
    server.build_router()
}

#[tokio::test]
async fn list_documents_shows_uuid_scoped_metadata_for_default_workspace_alias() {
    let state = AppState::test_state();
    let doc_id = "scope-test-doc-001";
    let metadata_key = format!("{}-metadata", doc_id);
    state
        .storage
        .kv_storage
        .upsert(&[(
            metadata_key,
            json!({
                "id": doc_id,
                "title": "Scoped Doc",
                "status": "completed",
                "workspace_id": default_workspace_uuid().to_string(),
                "tenant_id": default_tenant_uuid().to_string(),
            }),
        )])
        .await
        .unwrap();

    let app = create_test_app(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/documents")
                .header("X-Tenant-ID", "default")
                .header("X-Workspace-ID", "default")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let documents = json
        .get("documents")
        .and_then(|v| v.as_array())
        .expect("documents array");
    assert!(
        documents
            .iter()
            .any(|d| d.get("id").and_then(|v| v.as_str()) == Some(doc_id)),
        "expected scoped document in list, got: {}",
        json
    );
}
