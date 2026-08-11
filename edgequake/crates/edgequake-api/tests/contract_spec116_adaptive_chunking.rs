//! SPEC-116 — Workspace chunking policy API contract.
//!
//! Run:
//!   cargo test -p edgequake-api --test contract_spec116_adaptive_chunking

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{create_test_app, extract_json};
use edgequake_api::openapi::ApiDoc;
use serde_json::json;
use tower::ServiceExt;
use utoipa::OpenApi;
use uuid::Uuid;

async fn create_tenant(app: &axum::Router) -> String {
    let slug = format!("spec116-{}", Uuid::new_v4());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": format!("SPEC-116 {slug}"),
                        "slug": slug,
                        "plan": "pro"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    extract_json(response).await["id"]
        .as_str()
        .expect("tenant id")
        .to_string()
}

#[tokio::test]
async fn spec116_api_create_update_get_chunking() {
    let app = create_test_app();
    let tenant_id = create_tenant(&app).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Chunk WS",
                        "slug": format!("chunk-{}", Uuid::new_v4()),
                        "chunking_mode": "fixed",
                        "chunk_token_size": 1200,
                        "chunk_overlap_token_size": 100
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let created = extract_json(create).await;
    assert_eq!(created["chunking_mode"].as_str(), Some("fixed"));
    assert_eq!(created["chunk_token_size"].as_u64(), Some(1200));
    assert_eq!(created["chunk_overlap_token_size"].as_u64(), Some(100));
    let workspace_id = created["id"].as_str().unwrap().to_string();

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", &tenant_id)
                .body(Body::from(
                    json!({ "chunking_mode": "adaptive" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    let updated = extract_json(update).await;
    assert_eq!(updated["chunking_mode"].as_str(), Some("adaptive"));
    assert!(
        updated["chunk_token_size"].is_null() || updated.get("chunk_token_size").is_none(),
        "adaptive clears size keys: {updated}"
    );

    let clear = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", &tenant_id)
                .body(Body::from(
                    json!({ "chunking_mode": "inherit" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(clear.status(), StatusCode::OK);
    let cleared = extract_json(clear).await;
    assert!(
        cleared["chunking_mode"].is_null() || cleared.get("chunking_mode").is_none(),
        "inherit clears chunking_mode: {cleared}"
    );
}

#[tokio::test]
async fn spec116_api_rejects_overlap_gte_size() {
    let app = create_test_app();
    let tenant_id = create_tenant(&app).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Bad Chunk",
                        "slug": format!("bad-chunk-{}", Uuid::new_v4()),
                        "chunking_mode": "fixed",
                        "chunk_token_size": 100,
                        "chunk_overlap_token_size": 100
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::BAD_REQUEST);
    let msg = extract_json(create).await.to_string();
    assert!(
        msg.contains("chunk_overlap") || msg.contains("must be <"),
        "expected overlap validation, got {msg}"
    );
}

#[tokio::test]
async fn spec116_api_fixed_defaults_1200_100() {
    let app = create_test_app();
    let tenant_id = create_tenant(&app).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Default Fixed",
                        "slug": format!("def-fixed-{}", Uuid::new_v4()),
                        "chunking_mode": "fixed"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let created = extract_json(create).await;
    assert_eq!(created["chunking_mode"].as_str(), Some("fixed"));
    assert_eq!(created["chunk_token_size"].as_u64(), Some(1200));
    assert_eq!(created["chunk_overlap_token_size"].as_u64(), Some(100));
}

#[test]
fn spec116_openapi_has_chunking_fields() {
    let doc = ApiDoc::openapi();
    let json = serde_json::to_value(&doc).expect("serialize openapi");
    let blob = json.to_string();
    assert!(
        blob.contains("chunking_mode"),
        "OpenAPI missing chunking_mode"
    );
    assert!(
        blob.contains("chunk_token_size"),
        "OpenAPI missing chunk_token_size"
    );
    assert!(
        blob.contains("chunk_overlap_token_size"),
        "OpenAPI missing chunk_overlap_token_size"
    );
}
