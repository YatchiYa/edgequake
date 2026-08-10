//! SPEC-114 — relation_types / relation_types_strict / kg_schema_preset persist.
//!
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! cargo test -p edgequake-api --features postgres --test e2e_spec114_relation_types
//! ```

#![cfg(feature = "postgres")]

mod common;

use common::spec013_postgres;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{extract_json, post_json};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn spec114_relation_types_persist() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let suffix = uuid::Uuid::new_v4();

    let (_, tenant_body) = post_json(
        &app,
        "/api/v1/tenants",
        &json!({ "name": format!("SPEC114 {suffix}") }),
    )
    .await;
    let tenant_id = tenant_body["id"].as_str().unwrap();

    let (_, ws_body) = post_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces"),
        &json!({
            "name": "Schema WS",
            "relation_types": ["works-at", "PART_OF", "works-at"],
            "relation_types_strict": false,
            "kg_schema_preset": "Manufacturing"
        }),
    )
    .await;
    let workspace_id = ws_body["id"].as_str().unwrap();
    let types = ws_body["relation_types"].as_array().expect("relation_types");
    assert_eq!(types.len(), 2);
    assert_eq!(types[0].as_str(), Some("WORKS_AT"));
    assert_eq!(types[1].as_str(), Some("PART_OF"));
    assert_eq!(ws_body["relation_types_strict"].as_bool(), Some(false));
    assert_eq!(ws_body["kg_schema_preset"].as_str(), Some("manufacturing"));

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", tenant_id)
                .body(Body::from(
                    json!({
                        "relation_types": [],
                        "relation_types_strict": true,
                        "kg_schema_preset": "none"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    let updated = extract_json(update).await;
    assert!(
        updated.get("relation_types").is_none()
            || updated["relation_types"].as_array().map(|a| a.is_empty()) == Some(true)
    );
    assert_eq!(updated["relation_types_strict"].as_bool(), Some(true));
    assert!(
        updated.get("kg_schema_preset").is_none()
            || updated["kg_schema_preset"].is_null()
    );

    let bad = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{workspace_id}"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", tenant_id)
                .body(Body::from(
                    json!({ "kg_schema_preset": "aliens" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn spec114_relation_types_cap_fifty() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let (_, tenant_body) = post_json(
        &app,
        "/api/v1/tenants",
        &json!({ "name": format!("SPEC114cap {suffix}") }),
    )
    .await;
    let tenant_id = tenant_body["id"].as_str().unwrap();

    let many: Vec<String> = (0..60).map(|i| format!("rel_{i}")).collect();
    let (_, ws_body) = post_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces"),
        &json!({
            "name": "Cap WS",
            "relation_types": many
        }),
    )
    .await;
    let types = ws_body["relation_types"].as_array().expect("relation_types");
    assert_eq!(types.len(), 50);
}
