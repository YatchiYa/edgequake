//! SPEC-114 G-114-15 — Entity + relation extraction under workspace schema (mock ingest).
//!
//! ```bash
//! cargo test -p edgequake-api --test e2e_spec114_extraction_schema -- --nocapture --test-threads=1
//! ```

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{create_test_app_with_extraction_only, extract_json, upload_and_wait_with_tenant};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;
use tower::ServiceExt;

/// Canonical default workspace id (`Uuid::from_u128(3)`).
const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000003";
const DEFAULT_TENANT_ID: &str = "00000000-0000-0000-0000-000000000002";
const DEFAULT_USER_ID: &str = "00000000-0000-0000-0000-000000000001";

const HAPPY_JSON: &str = r#"{
  "entities": [
    {"name": "Alice", "type": "PERSON", "description": "Engineer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Company"}
  ],
  "relationships": [
    {"source": "Alice", "target": "Acme", "type": "WORKS_AT", "description": "Alice works at Acme"}
  ]
}"#;

const UNKNOWN_RELATION_JSON: &str = r#"{
  "entities": [
    {"name": "Alice", "type": "PERSON", "description": "Engineer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Company"}
  ],
  "relationships": [
    {"source": "Alice", "target": "Acme", "type": "EMPLOYS", "description": "bad label"}
  ]
}"#;

const REVERSED_EDGE_JSON: &str = r#"{
  "entities": [
    {"name": "Alice", "type": "PERSON", "description": "Engineer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Company"}
  ],
  "relationships": [
    {"source": "Acme", "target": "Alice", "type": "WORKS_AT", "description": "reversed endpoints"}
  ]
}"#;

const UNKNOWN_ENTITY_JSON: &str = r#"{
  "entities": [
    {"name": "Alice", "type": "PERSON", "description": "Engineer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Company"},
    {"name": "Mystery", "type": "WIDGET", "description": "unknown entity type"}
  ],
  "relationships": [
    {"source": "Alice", "target": "Acme", "type": "WORKS_AT", "description": "Alice works at Acme"}
  ]
}"#;

const DOC: &str = "Alice works at Acme Corp.";

fn default_scope() -> (&'static str, &'static str, &'static str) {
    (DEFAULT_TENANT_ID, DEFAULT_USER_ID, DEFAULT_WORKSPACE_ID)
}

async fn put_workspace_schema(app: &axum::Router, body: Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{DEFAULT_WORKSPACE_ID}"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", DEFAULT_TENANT_ID)
                .header("X-Workspace-ID", DEFAULT_WORKSPACE_ID)
                .header("X-User-ID", DEFAULT_USER_ID)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "workspace schema update failed: {:?}",
        extract_json(response).await
    );
}

async fn get_graph(app: &axum::Router) -> Value {
    let (tenant, user, workspace) = default_scope();
    let (status, body) = common::get_with_tenant(
        app,
        "/api/v1/graph?max_nodes=200&depth=3",
        tenant,
        user,
        workspace,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "graph GET failed: {body}");
    body
}

async fn ingest(app: &axum::Router, title: &str) -> (String, String, String) {
    upload_and_wait_with_tenant(
        app,
        title,
        DOC,
        Duration::from_secs(45),
        Some(default_scope()),
    )
    .await
}

fn edge_types(graph: &Value) -> HashSet<String> {
    graph["edges"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            e.get("relationship_type")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    e.get("properties")
                        .and_then(|p| p.get("relationship_type").or_else(|| p.get("relation_type")))
                        .and_then(|v| v.as_str())
                })
                .map(|s| s.to_uppercase())
        })
        .collect()
}

fn node_types(graph: &Value) -> HashSet<String> {
    graph["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| {
            n.get("node_type")
                .or_else(|| n.get("entity_type"))
                .or_else(|| n.get("type"))
                .and_then(|v| v.as_str())
                .or_else(|| {
                    n.get("properties")
                        .and_then(|p| p.get("entity_type").or_else(|| p.get("type")))
                        .and_then(|v| v.as_str())
                })
                .map(|s| s.to_uppercase())
        })
        .collect()
}

fn dual_allowlist_payload(relation_strict: bool, with_edges: bool) -> Value {
    let mut body = json!({
        "entity_types": ["PERSON", "ORGANIZATION", "OTHER"],
        "entity_types_strict": true,
        "relation_types": ["WORKS_AT", "RELATED_TO"],
        "relation_types_strict": relation_strict,
        "kg_schema_preset": "custom"
    });
    if with_edges {
        body["relation_edges"] = json!([
            {
                "source": "PERSON",
                "relation": "WORKS_AT",
                "target": "ORGANIZATION"
            }
        ]);
    } else {
        body["relation_edges"] = json!([]);
    }
    body
}

#[tokio::test]
async fn spec114_extract_happy_path_works_at() {
    let workers = create_test_app_with_extraction_only(HAPPY_JSON).await;
    let app = workers.app();
    put_workspace_schema(app, dual_allowlist_payload(true, true)).await;

    let (_doc_id, _track, status) = ingest(app, "spec114-happy").await;
    assert!(
        matches!(
            status.as_str(),
            "completed" | "processed" | "indexed" | "partial_failure"
        ),
        "unexpected status {status}"
    );

    let graph = get_graph(app).await;
    let types = edge_types(&graph);
    assert!(
        types.contains("WORKS_AT"),
        "expected WORKS_AT edge, got {types:?} graph={graph}"
    );
    let entities = node_types(&graph);
    assert!(
        entities.contains("PERSON") || entities.contains("ORGANIZATION"),
        "expected PERSON/ORGANIZATION nodes, got {entities:?}"
    );
}

#[tokio::test]
async fn spec114_extract_strict_relation_remaps_unknown() {
    let workers = create_test_app_with_extraction_only(UNKNOWN_RELATION_JSON).await;
    let app = workers.app();
    put_workspace_schema(app, dual_allowlist_payload(true, false)).await;

    ingest(app, "spec114-strict-rel").await;
    let types = edge_types(&get_graph(app).await);
    assert!(
        !types.contains("EMPLOYS"),
        "EMPLOYS must be remapped under strict relation allow-list, got {types:?}"
    );
    assert!(
        types.contains("RELATED_TO") || types.contains("WORKS_AT"),
        "expected RELATED_TO or WORKS_AT fallback, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_extract_permissive_relation_passthrough() {
    let workers = create_test_app_with_extraction_only(UNKNOWN_RELATION_JSON).await;
    let app = workers.app();
    put_workspace_schema(app, dual_allowlist_payload(false, false)).await;

    ingest(app, "spec114-perm-rel").await;
    let types = edge_types(&get_graph(app).await);
    assert!(
        types.contains("EMPLOYS"),
        "strict=false must keep EMPLOYS, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_extract_typed_edge_violation_remaps() {
    let workers = create_test_app_with_extraction_only(REVERSED_EDGE_JSON).await;
    let app = workers.app();
    put_workspace_schema(app, dual_allowlist_payload(true, true)).await;

    ingest(app, "spec114-edge-viol").await;
    let types = edge_types(&get_graph(app).await);
    // Reversed PERSON←WORKS_AT←ORG has no matching typed edge; strict remaps
    // to RELATED_TO (preferred fallback) or WORKS_AT (first allow-list).
    assert!(
        !types.is_empty(),
        "expected at least one relationship after edge enforce"
    );
    assert!(
        types.contains("RELATED_TO") || types.contains("WORKS_AT"),
        "expected remapped relation label, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_extract_empty_relations_free_form() {
    let workers = create_test_app_with_extraction_only(UNKNOWN_RELATION_JSON).await;
    let app = workers.app();
    put_workspace_schema(
        app,
        json!({
            "entity_types": ["PERSON", "ORGANIZATION", "OTHER"],
            "entity_types_strict": true,
            "relation_types": [],
            "relation_edges": [],
            "kg_schema_preset": "blank"
        }),
    )
    .await;

    ingest(app, "spec114-free-form").await;
    let types = edge_types(&get_graph(app).await);
    assert!(
        types.contains("EMPLOYS"),
        "empty relation_types must preserve free-form EMPLOYS, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_extract_unknown_entity_other_keeps_relation() {
    let workers = create_test_app_with_extraction_only(UNKNOWN_ENTITY_JSON).await;
    let app = workers.app();
    put_workspace_schema(app, dual_allowlist_payload(true, true)).await;

    ingest(app, "spec114-entity-other").await;
    let graph = get_graph(app).await;
    let entities = node_types(&graph);
    assert!(
        !entities.contains("WIDGET"),
        "WIDGET must remap under entity strict, got {entities:?}"
    );
    assert!(
        entities.contains("OTHER") || entities.contains("PERSON"),
        "expected OTHER and/or PERSON, got {entities:?}"
    );
    let types = edge_types(&graph);
    assert!(
        types.contains("WORKS_AT") || types.contains("RELATED_TO"),
        "relation allow-list must still apply, got {types:?}"
    );
}
