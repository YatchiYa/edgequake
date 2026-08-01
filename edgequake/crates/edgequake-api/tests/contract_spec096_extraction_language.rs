//! SPEC-096 — Workspace `extraction_language` API contract.
//!
//! Run:
//!   cargo test -p edgequake-api --test contract_spec096_extraction_language

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
    let slug = format!("spec096-{}", Uuid::new_v4());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": format!("SPEC-096 {slug}"),
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
async fn spec096_api_create_update_get_language() {
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
                        "name": "Lang WS",
                        "slug": format!("lang-{}", Uuid::new_v4()),
                        "extraction_language": "chinese"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let created = extract_json(create).await;
    assert_eq!(created["extraction_language"].as_str(), Some("Chinese"));
    let workspace_id = created["id"].as_str().unwrap().to_string();

    let get = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .header("X-Tenant-ID", &tenant_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);
    assert_eq!(
        extract_json(get).await["extraction_language"].as_str(),
        Some("Chinese")
    );

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", &tenant_id)
                .body(Body::from(
                    json!({ "extraction_language": "Japanese" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    assert_eq!(
        extract_json(update).await["extraction_language"].as_str(),
        Some("Japanese")
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
                    json!({ "extraction_language": "none" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(clear.status(), StatusCode::OK);
    let cleared = extract_json(clear).await;
    assert!(
        cleared["extraction_language"].is_null() || cleared.get("extraction_language").is_none(),
        "clearing override should omit/null extraction_language, got {cleared}"
    );
}

#[tokio::test]
async fn spec096_api_rejects_unsupported_language() {
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
                        "name": "Bad Lang",
                        "slug": format!("bad-{}", Uuid::new_v4()),
                        "extraction_language": "Klingon"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::BAD_REQUEST);
    let body = extract_json(create).await;
    let msg = body.to_string();
    assert!(
        msg.contains("Unsupported extraction_language") || msg.contains("Klingon"),
        "expected allowlist error, got {msg}"
    );
}

#[test]
fn spec096_openapi_has_extraction_language() {
    let doc = ApiDoc::openapi();
    let json = serde_json::to_value(&doc).expect("serialize openapi");
    let blob = json.to_string();
    assert!(
        blob.contains("extraction_language"),
        "OpenAPI schema must document extraction_language on workspace DTOs"
    );
}
