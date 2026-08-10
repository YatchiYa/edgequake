//! SPEC-114 shared live extract matrix (Mistral / Ollama).
//!
//! Soft-asserts allow-list compliance after real-model ingest. Remap math is
//! covered by unit + mock gates; these scenarios prove prompt + post-enforce
//! graph labels under a live provider.

#![cfg(feature = "postgres")]
#![allow(dead_code)]

use super::spec013_postgres;
use super::{extract_json, get_with_tenant, post_json_with_tenant, TEST_USER_ID};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;
use tower::ServiceExt;

/// Which live LLM provider drives the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveProviderKind {
    Mistral,
    Ollama,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveScenario {
    HappyDualAllowlist,
    FreeFormRelations,
    StrictClosedWorld,
    PermissiveRelations,
    TypedEdgePresent,
    EntityOtherPath,
}

impl LiveScenario {
    pub fn all() -> &'static [LiveScenario] {
        &[
            LiveScenario::HappyDualAllowlist,
            LiveScenario::FreeFormRelations,
            LiveScenario::StrictClosedWorld,
            LiveScenario::PermissiveRelations,
            LiveScenario::TypedEdgePresent,
            LiveScenario::EntityOtherPath,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            LiveScenario::HappyDualAllowlist => "happy",
            LiveScenario::FreeFormRelations => "free-form",
            LiveScenario::StrictClosedWorld => "strict-closed",
            LiveScenario::PermissiveRelations => "permissive",
            LiveScenario::TypedEdgePresent => "typed-edge",
            LiveScenario::EntityOtherPath => "entity-other",
        }
    }

    fn doc_text(self) -> &'static str {
        match self {
            LiveScenario::StrictClosedWorld => {
                "Alice is employed by Acme Corp and resides in Paris. \
                 She collaborates with Bob who mentors junior engineers."
            }
            _ => "Alice works at Acme in Paris.",
        }
    }

    fn entity_allowlist(self) -> HashSet<&'static str> {
        match self {
            LiveScenario::EntityOtherPath => ["PERSON", "ORGANIZATION", "OTHER"]
                .into_iter()
                .collect(),
            _ => ["PERSON", "ORGANIZATION", "OTHER"].into_iter().collect(),
        }
    }

    fn relation_allowlist(self) -> HashSet<&'static str> {
        ["WORKS_AT", "LOCATED_IN", "RELATED_TO"].into_iter().collect()
    }

    fn workspace_json(self, provider: LiveProviderKind, name: &str) -> Value {
        let entity = &["PERSON", "ORGANIZATION", "OTHER"];
        let edges = &[("PERSON", "WORKS_AT", "ORGANIZATION")];
        match (provider, self) {
            (LiveProviderKind::Mistral, LiveScenario::FreeFormRelations) => {
                spec013_postgres::mistral_kg_schema_workspace_json(
                    name, entity, &[], &[], true, true,
                )
            }
            (LiveProviderKind::Ollama, LiveScenario::FreeFormRelations) => {
                spec013_postgres::ollama_kg_schema_workspace_json(
                    name, entity, &[], &[], true, true,
                )
            }
            (LiveProviderKind::Mistral, LiveScenario::PermissiveRelations) => {
                spec013_postgres::mistral_kg_schema_workspace_json(
                    name,
                    entity,
                    &["WORKS_AT", "LOCATED_IN"],
                    edges,
                    true,
                    false,
                )
            }
            (LiveProviderKind::Ollama, LiveScenario::PermissiveRelations) => {
                spec013_postgres::ollama_kg_schema_workspace_json(
                    name,
                    entity,
                    &["WORKS_AT", "LOCATED_IN"],
                    edges,
                    true,
                    false,
                )
            }
            (LiveProviderKind::Mistral, _) => {
                spec013_postgres::mistral_kg_schema_workspace_json(
                    name,
                    entity,
                    &["WORKS_AT", "LOCATED_IN"],
                    edges,
                    true,
                    true,
                )
            }
            (LiveProviderKind::Ollama, _) => {
                spec013_postgres::ollama_kg_schema_workspace_json(
                    name,
                    entity,
                    &["WORKS_AT", "LOCATED_IN"],
                    edges,
                    true,
                    true,
                )
            }
        }
    }
}

pub fn edge_types(graph: &Value) -> HashSet<String> {
    graph["edges"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            e.get("relationship_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_uppercase())
        })
        .collect()
}

pub fn node_types(graph: &Value) -> HashSet<String> {
    graph["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| {
            n.get("node_type")
                .or_else(|| n.get("entity_type"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_uppercase())
        })
        .collect()
}

fn wait_timeout(provider: LiveProviderKind) -> Duration {
    match provider {
        LiveProviderKind::Mistral => Duration::from_secs(180),
        LiveProviderKind::Ollama => Duration::from_secs(300),
    }
}

fn provider_label(provider: LiveProviderKind) -> &'static str {
    match provider {
        LiveProviderKind::Mistral => "Mistral",
        LiveProviderKind::Ollama => "Ollama",
    }
}

/// Run one live scenario: ingest + soft graph asserts.
pub async fn run_live_scenario(
    app: &axum::Router,
    provider: LiveProviderKind,
    scenario: LiveScenario,
) {
    let suffix = uuid::Uuid::new_v4();
    let tenant_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": format!(
                            "SPEC114 {} {} {suffix}",
                            provider_label(provider),
                            scenario.label()
                        )
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tenant_res.status(), StatusCode::CREATED);
    let tenant = extract_json(tenant_res).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let ws_name = format!(
        "SPEC114 {} {} {suffix}",
        provider_label(provider),
        scenario.label()
    );
    let ws_body = scenario.workspace_json(provider, &ws_name);
    let ws_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .body(Body::from(ws_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let ws_status = ws_res.status();
    let ws = extract_json(ws_res).await;
    assert_eq!(ws_status, StatusCode::CREATED, "workspace create: {ws}");
    match provider {
        LiveProviderKind::Mistral => spec013_postgres::assert_workspace_uses_mistral(&ws),
        LiveProviderKind::Ollama => spec013_postgres::assert_workspace_uses_ollama(&ws),
    }
    let workspace_id = ws["id"].as_str().unwrap();

    let (status, body) = post_json_with_tenant(
        app,
        "/api/v1/documents",
        &json!({
            "content": scenario.doc_text(),
            "title": format!("spec114-{}-{}-{suffix}", provider_label(provider).to_lowercase(), scenario.label()),
            "async_processing": true
        }),
        tenant_id,
        TEST_USER_ID,
        workspace_id,
    )
    .await;
    assert!(
        status == StatusCode::CREATED
            || status == StatusCode::ACCEPTED
            || status == StatusCode::OK,
        "upload: {status} {body:?}"
    );
    let doc_id = body["document_id"]
        .as_str()
        .or_else(|| body["id"].as_str())
        .expect("document_id");

    let deadline = tokio::time::Instant::now() + wait_timeout(provider);
    let mut terminal = String::new();
    loop {
        if tokio::time::Instant::now() > deadline {
            panic!(
                "[{} {}] document did not complete within {:?} (last={terminal})",
                provider_label(provider),
                scenario.label(),
                wait_timeout(provider)
            );
        }
        let (st, doc) = get_with_tenant(
            app,
            &format!("/api/v1/documents/{doc_id}"),
            tenant_id,
            TEST_USER_ID,
            workspace_id,
        )
        .await;
        if st.is_success() {
            let status_str = doc["status"].as_str().unwrap_or("").to_string();
            terminal = status_str.clone();
            if matches!(
                status_str.to_ascii_lowercase().as_str(),
                "completed" | "processed" | "indexed"
            ) {
                break;
            }
            if status_str.eq_ignore_ascii_case("failed") {
                panic!(
                    "[{} {}] document failed: {doc}",
                    provider_label(provider),
                    scenario.label()
                );
            }
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    let (gst, graph) = get_with_tenant(
        app,
        "/api/v1/graph?max_nodes=200&depth=3",
        tenant_id,
        TEST_USER_ID,
        workspace_id,
    )
    .await;
    assert_eq!(gst, StatusCode::OK, "graph GET failed: {graph}");

    soft_assert_graph(provider, scenario, &graph);
}

fn soft_assert_graph(provider: LiveProviderKind, scenario: LiveScenario, graph: &Value) {
    let entities = node_types(graph);
    let rels = edge_types(graph);
    let entity_allow = scenario.entity_allowlist();
    let rel_allow = scenario.relation_allowlist();

    eprintln!(
        "[{} {}] entities={entities:?} relations={rels:?}",
        provider_label(provider),
        scenario.label()
    );

    assert!(
        entities
            .iter()
            .any(|t| entity_allow.contains(t.as_str())),
        "[{} {}] expected ≥1 allow-listed entity, got {entities:?} graph={graph}",
        provider_label(provider),
        scenario.label()
    );

    // Entity strict: no out-of-list entity labels.
    for t in &entities {
        assert!(
            entity_allow.contains(t.as_str()),
            "[{} {}] entity `{t}` outside allow-list {entity_allow:?} under strict entity schema",
            provider_label(provider),
            scenario.label()
        );
    }

    match scenario {
        LiveScenario::FreeFormRelations => {
            assert!(
                !rels.is_empty(),
                "[{} {}] free-form must yield ≥1 relationship, graph={graph}",
                provider_label(provider),
                scenario.label()
            );
        }
        LiveScenario::PermissiveRelations => {
            assert!(
                !rels.is_empty(),
                "[{} {}] permissive must yield ≥1 relationship, graph={graph}",
                provider_label(provider),
                scenario.label()
            );
        }
        LiveScenario::HappyDualAllowlist
        | LiveScenario::StrictClosedWorld
        | LiveScenario::TypedEdgePresent
        | LiveScenario::EntityOtherPath => {
            assert!(
                !rels.is_empty(),
                "[{} {}] expected ≥1 relationship, graph={graph}",
                provider_label(provider),
                scenario.label()
            );
            assert!(
                rels.iter().any(|t| rel_allow.contains(t.as_str())),
                "[{} {}] expected ≥1 relation ∈ allow-list, got {rels:?}",
                provider_label(provider),
                scenario.label()
            );
            // Strict closed world: every relation ⊆ allow-list.
            if matches!(
                scenario,
                LiveScenario::HappyDualAllowlist
                    | LiveScenario::StrictClosedWorld
                    | LiveScenario::TypedEdgePresent
                    | LiveScenario::EntityOtherPath
            ) {
                for t in &rels {
                    assert!(
                        rel_allow.contains(t.as_str()),
                        "[{} {}] relation `{t}` outside allow-list {rel_allow:?} (strict)",
                        provider_label(provider),
                        scenario.label()
                    );
                }
            }
        }
    }
}

/// Run the full soft EC matrix for one provider (serial scenarios).
pub async fn run_full_live_matrix(app: &axum::Router, provider: LiveProviderKind) {
    for scenario in LiveScenario::all() {
        eprintln!(
            "=== SPEC-114 live {} / {} ===",
            provider_label(provider),
            scenario.label()
        );
        run_live_scenario(app, provider, *scenario).await;
    }
}
