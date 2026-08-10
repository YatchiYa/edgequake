//! SPEC-114b — relation_edges persist + normalize.
//!
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! cargo test -p edgequake-api --features postgres --test e2e_spec114_relation_edges
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
async fn spec114_relation_edges_persist() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let suffix = uuid::Uuid::new_v4();

    let (_, tenant_body) = post_json(
        &app,
        "/api/v1/tenants",
        &json!({ "name": format!("SPEC114b {suffix}") }),
    )
    .await;
    let tenant_id = tenant_body["id"].as_str().unwrap();

    let (_, ws_body) = post_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces"),
        &json!({
            "name": "Edges WS",
            "entity_types": ["PERSON", "ORGANIZATION", "LOCATION"],
            "relation_types": ["WORKS_AT", "LOCATED_IN"],
            "relation_edges": [
                {
                    "source": " person ",
                    "relation": "works-at",
                    "target": "ORGANIZATION"
                },
                {
                    "source": "PERSON",
                    "relation": "WORKS_AT",
                    "target": "ORGANIZATION"
                },
                {
                    "source": "ALIEN",
                    "relation": "WORKS_AT",
                    "target": "ORGANIZATION"
                }
            ],
            "kg_schema_preset": "custom"
        }),
    )
    .await;
    let workspace_id = ws_body["id"].as_str().unwrap();
    let edges = ws_body["relation_edges"]
        .as_array()
        .expect("relation_edges");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["source"].as_str(), Some("PERSON"));
    assert_eq!(edges[0]["relation"].as_str(), Some("WORKS_AT"));
    assert_eq!(edges[0]["target"].as_str(), Some("ORGANIZATION"));

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
                        "relation_edges": []
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
        updated.get("relation_edges").is_none()
            || updated["relation_edges"].as_array().map(|a| a.is_empty()) == Some(true)
    );
}

#[tokio::test]
#[serial]
async fn spec114_relation_edges_cap_hundred() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let (_, tenant_body) = post_json(
        &app,
        "/api/v1/tenants",
        &json!({ "name": format!("SPEC114bcap {suffix}") }),
    )
    .await;
    let tenant_id = tenant_body["id"].as_str().unwrap();

    let edges: Vec<_> = (0..120)
        .map(|i| {
            json!({
                "source": "A",
                "relation": "R",
                "target": format!("T{i}")
            })
        })
        .collect();

    let (_, ws_body) = post_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces"),
        &json!({
            "name": "Cap Edges",
            "relation_types": ["R"],
            "relation_edges": edges
        }),
    )
    .await;
    let stored = ws_body["relation_edges"].as_array().expect("edges");
    assert_eq!(stored.len(), 100);
}
