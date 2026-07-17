//! E2E: tenant/workspace create persists pdf_parser_backend=vision.
//!
//! Covers the create path end-to-end through the HTTP API, including the case
//! where server env prefers edgeparse (must not override a new workspace).

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{create_test_app, extract_json};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_tenant(app: &axum::Router, plan: &str) -> String {
    let slug = format!("vision-e2e-{}", Uuid::new_v4());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": format!("Vision E2E {slug}"),
                        "slug": slug,
                        "plan": plan
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = extract_json(response).await;
    body["id"].as_str().expect("tenant id").to_string()
}

async fn get_workspace(app: &axum::Router, workspace_id: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    extract_json(response).await
}

#[tokio::test]
async fn e2e_create_tenant_auto_default_workspace_persists_vision() {
    let app = create_test_app();
    let tenant_id = create_tenant(&app, "free").await;

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let body = extract_json(list).await;
    let items = body["items"].as_array().expect("items");
    assert!(
        !items.is_empty(),
        "tenant create must auto-create Default Workspace"
    );

    let workspace_id = items[0]["id"].as_str().expect("workspace id");
    assert_eq!(
        items[0]["pdf_parser_backend"].as_str(),
        Some("vision"),
        "list response must show vision"
    );

    // Round-trip GET must also report persisted vision.
    let detail = get_workspace(&app, workspace_id).await;
    assert_eq!(detail["pdf_parser_backend"].as_str(), Some("vision"));
    assert_eq!(detail["slug"].as_str(), Some("default"));
}

#[tokio::test]
async fn e2e_create_workspace_omitted_backend_persists_vision_roundtrip() {
    let app = create_test_app();
    let tenant_id = create_tenant(&app, "pro").await;

    let create_ws = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "No Backend Field",
                        "slug": format!("ws-{}", Uuid::new_v4())
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_ws.status(), StatusCode::CREATED);
    let created = extract_json(create_ws).await;
    assert_eq!(created["pdf_parser_backend"].as_str(), Some("vision"));

    let workspace_id = created["id"].as_str().expect("workspace id");
    let detail = get_workspace(&app, workspace_id).await;
    assert_eq!(
        detail["pdf_parser_backend"].as_str(),
        Some("vision"),
        "GET after create must return persisted vision"
    );
}

#[tokio::test]
async fn e2e_create_workspace_explicit_edgeparse_still_allowed() {
    let app = create_test_app();
    let tenant_id = create_tenant(&app, "pro").await;

    let create_ws = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "EdgeParse WS",
                        "slug": format!("ep-{}", Uuid::new_v4()),
                        "pdf_parser_backend": "edgeparse"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_ws.status(), StatusCode::CREATED);
    let created = extract_json(create_ws).await;
    assert_eq!(created["pdf_parser_backend"].as_str(), Some("edgeparse"));

    let workspace_id = created["id"].as_str().expect("workspace id");
    let detail = get_workspace(&app, workspace_id).await;
    assert_eq!(detail["pdf_parser_backend"].as_str(), Some("edgeparse"));
}

/// Restores `EDGEQUAKE_PDF_PARSER_BACKEND` even if the test panics.
struct PdfParserBackendEnvGuard {
    previous: Option<String>,
}

impl PdfParserBackendEnvGuard {
    fn set_edgeparse() -> Self {
        let previous = std::env::var("EDGEQUAKE_PDF_PARSER_BACKEND").ok();
        std::env::set_var("EDGEQUAKE_PDF_PARSER_BACKEND", "edgeparse");
        Self { previous }
    }
}

impl Drop for PdfParserBackendEnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => std::env::set_var("EDGEQUAKE_PDF_PARSER_BACKEND", v),
            None => std::env::remove_var("EDGEQUAKE_PDF_PARSER_BACKEND"),
        }
    }
}

/// Env edgeparse must not override omitted create — workspace stays vision.
#[tokio::test]
#[serial]
async fn e2e_create_workspace_ignores_env_edgeparse_default() {
    let _env_guard = PdfParserBackendEnvGuard::set_edgeparse();

    let app = create_test_app();
    let tenant_id = create_tenant(&app, "pro").await;

    let create_ws = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Env Edgeparse Ignored",
                        "slug": format!("env-{}", Uuid::new_v4())
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_ws.status(), StatusCode::CREATED);
    let created = extract_json(create_ws).await;
    assert_eq!(
        created["pdf_parser_backend"].as_str(),
        Some("vision"),
        "new workspace must persist vision even when EDGEQUAKE_PDF_PARSER_BACKEND=edgeparse"
    );

    // Auto Default Workspace from tenant create must also be vision.
    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = extract_json(list).await;
    let default_ws = body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|w| w["slug"] == "default")
        .expect("default workspace");
    assert_eq!(
        default_ws["pdf_parser_backend"].as_str(),
        Some("vision"),
        "tenant auto-default workspace must persist vision under env edgeparse"
    );
}
