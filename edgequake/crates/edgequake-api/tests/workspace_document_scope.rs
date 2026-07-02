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

#[tokio::test]
async fn orphan_content_hash_is_recycled_on_reupload() {
    let state = AppState::test_state();
    let workspace_id = default_workspace_uuid().to_string();
    let tenant_ctx = edgequake_api::middleware::TenantContext {
        tenant_id: Some("default".to_string()),
        workspace_id: Some("default".to_string()),
        user_id: None,
    };

    let content = "SPEC-040 orphan hash recycle proof content";
    let content_hash = edgequake_api::services::ContentHasher::hash_str(content);
    let hash_key =
        edgequake_api::services::ContentHasher::workspace_hash_key(&workspace_id, &content_hash);
    let ghost_doc_id = "ghost-doc-spec040";

    state
        .storage
        .kv_storage
        .upsert(&[(hash_key.clone(), serde_json::json!(ghost_doc_id))])
        .await
        .unwrap();

    let app = create_test_app(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("content-type", "application/json")
                .header("X-Tenant-ID", "default")
                .header("X-Workspace-ID", "default")
                .body(Body::from(
                    serde_json::json!({
                        "content": content,
                        "title": "orphan-recycle.md",
                        "async_processing": true,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "orphan hash must not block upload"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json.get("duplicate_of").is_none(),
        "expected fresh upload, got duplicate: {}",
        json
    );
    assert_ne!(
        json.get("document_id").and_then(|v| v.as_str()),
        Some(ghost_doc_id),
        "new document id must not reuse ghost mapping"
    );

    let _ = tenant_ctx;
}
