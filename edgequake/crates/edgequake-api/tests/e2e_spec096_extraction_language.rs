//! SPEC-096 — Postgres metadata round-trip + ingest language wiring.
//!
//! Run:
//!   DATABASE_URL=postgresql://… \
//!     cargo test -p edgequake-api --features postgres --test e2e_spec096_extraction_language

#![cfg(feature = "postgres")]

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{extract_json, spec013_postgres};
use edgequake_pipeline::prompts::EntityExtractionSchema;
use edgequake_pipeline::{json_extraction_prompt, resolve_extraction_language_from_env};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn spec096_workspace_metadata_roundtrip() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP spec096_workspace_metadata_roundtrip: no DATABASE_URL");
        return;
    };

    let slug = format!("spec096-{}", Uuid::new_v4());
    let tenant = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": format!("SPEC-096 PG {slug}"),
                        "slug": slug,
                        "plan": "pro"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tenant.status(), StatusCode::CREATED);
    let tenant_id = extract_json(tenant).await["id"].as_str().unwrap().to_string();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Chinese KG",
                        "slug": format!("zh-{}", Uuid::new_v4()),
                        "extraction_language": "Chinese",
                        "llm_provider": "mock",
                        "llm_model": "mock-model",
                        "embedding_provider": "mock",
                        "embedding_model": "mock-embed",
                        "embedding_dimension": 768
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
}

#[tokio::test]
#[serial]
async fn spec096_e2e_ingest_prompt_language() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP spec096_e2e_ingest_prompt_language: no DATABASE_URL");
        return;
    };

    let slug = format!("spec096-ingest-{}", Uuid::new_v4());
    let tenant = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": format!("SPEC-096 Ingest {slug}"),
                        "slug": slug,
                        "plan": "pro"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let tenant_id = extract_json(tenant).await["id"].as_str().unwrap().to_string();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Ingest Lang",
                        "slug": format!("ingest-{}", Uuid::new_v4()),
                        "extraction_language": "Chinese",
                        "llm_provider": "mock",
                        "llm_model": "mock-model",
                        "embedding_provider": "mock",
                        "embedding_model": "mock-embed",
                        "embedding_dimension": 768
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let workspace = extract_json(create).await;
    assert_eq!(workspace["extraction_language"].as_str(), Some("Chinese"));

    // Resolved language must appear in production JSON prompt SSOT.
    let resolved = resolve_extraction_language_from_env(Some("Chinese"));
    assert_eq!(resolved, "Chinese");
    let prompt = json_extraction_prompt(
        "北京是中国的首都。",
        &EntityExtractionSchema::server_default(),
        &resolved,
    );
    assert!(prompt.contains("Chinese"));
    assert!(prompt.contains("Output Language"));
    assert!(prompt.contains("\"entities\""));
}
