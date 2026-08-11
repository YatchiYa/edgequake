//! Chat message lineage round-trip — mode + llm_provider/llm_model persistence.
//!
//! Proves the metadata-bar contract:
//! 1. Non-streaming chat returns effective llm_provider/llm_model (not null on default).
//! 2. Persisted assistant message retains mode + lineage after GET conversation.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_core::types::CreateWorkspaceRequest;
use edgequake_core::Tenant;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn create_test_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: true,
    }
}

async fn extract_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("parse json")
}

async fn create_workspace(state: &AppState) -> edgequake_core::Workspace {
    let tenant = Tenant::new(
        "Lineage Tenant".to_string(),
        format!("lineage-{}", Uuid::new_v4()),
    );
    let created_tenant = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .expect("create tenant");

    let request = CreateWorkspaceRequest {
        name: "Lineage Workspace".to_string(),
        slug: Some(format!("lineage-{}", Uuid::new_v4())),
        description: Some("message lineage".to_string()),
        max_documents: None,
        llm_model: Some("mock-model".to_string()),
        llm_provider: Some("mock".to_string()),
        embedding_model: Some("mock-embedding".to_string()),
        embedding_provider: Some("mock".to_string()),
        embedding_dimension: Some(1536),
        vision_llm_provider: None,
        vision_llm_model: None,
        pdf_parser_backend: None,
        entity_types: None,
        ..Default::default()
    };

    state
        .workspace_service
        .create_workspace(created_tenant.tenant_id, request)
        .await
        .expect("create workspace")
}

/// Workspace without painted LLM so chat falls through toward server default.
async fn create_workspace_without_llm(state: &AppState) -> edgequake_core::Workspace {
    let tenant = Tenant::new(
        "Default Lineage Tenant".to_string(),
        format!("lineage-default-{}", Uuid::new_v4()),
    );
    let created_tenant = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .expect("create tenant");

    let request = CreateWorkspaceRequest {
        name: "Default Lineage Workspace".to_string(),
        slug: Some(format!("lineage-default-{}", Uuid::new_v4())),
        description: Some("server default lineage".to_string()),
        max_documents: None,
        llm_model: Some(String::new()),
        llm_provider: Some(String::new()),
        embedding_model: Some("mock-embedding".to_string()),
        embedding_provider: Some("mock".to_string()),
        embedding_dimension: Some(1536),
        vision_llm_provider: None,
        vision_llm_model: None,
        pdf_parser_backend: None,
        entity_types: None,
        ..Default::default()
    };

    state
        .workspace_service
        .create_workspace(created_tenant.tenant_id, request)
        .await
        .expect("create workspace")
}

#[tokio::test]
async fn e2e_chat_persists_mode_and_llm_lineage() {
    let state = AppState::test_state();
    let workspace = create_workspace(&state).await;
    let app = Server::new(create_test_config(), state).build_router();
    let tenant = workspace.tenant_id.to_string();
    let user = Uuid::new_v4().to_string();
    let ws = workspace.workspace_id.to_string();

    let body = json!({
        "message": "What is Chat mode lineage?",
        "mode": "bypass",
        "stream": false
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/chat/completions")
                .header("Content-Type", "application/json")
                .header("X-Workspace-ID", &ws)
                .header("X-Tenant-Id", &tenant)
                .header("X-User-Id", &user)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if response.status() != StatusCode::OK {
        let status = response.status();
        let err_body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read err body");
        panic!(
            "chat failed status={status} body={}",
            String::from_utf8_lossy(&err_body)
        );
    }
    let parsed = extract_json(response).await;

    assert_eq!(parsed["mode"].as_str(), Some("bypass"));
    let provider = parsed["llm_provider"].as_str().expect("llm_provider on response");
    let model = parsed["llm_model"].as_str().expect("llm_model on response");
    assert!(!provider.is_empty(), "provider must be non-empty");
    assert!(!model.is_empty(), "model must be non-empty");
    let conversation_id = parsed["conversation_id"].as_str().expect("conversation_id");

    let detail = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/conversations/{conversation_id}"))
                .header("X-Workspace-ID", &ws)
                .header("X-Tenant-Id", &tenant)
                .header("X-User-Id", &user)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail_json = extract_json(detail).await;
    let messages = detail_json["messages"]
        .as_array()
        .expect("messages array");
    let assistant = messages
        .iter()
        .find(|m| m["role"].as_str() == Some("assistant"))
        .expect("assistant message");

    assert_eq!(assistant["mode"].as_str(), Some("bypass"));
    assert_eq!(assistant["llm_provider"].as_str(), Some(provider));
    assert_eq!(assistant["llm_model"].as_str(), Some(model));
    // Mock may report 0 tokens; field must still be present after update_message.
    assert!(
        assistant.get("tokens_used").is_some() && !assistant["tokens_used"].is_null(),
        "tokens_used field should be present: {assistant}"
    );
}

#[tokio::test]
async fn e2e_chat_server_default_still_emits_lineage() {
    let state = AppState::test_state();
    let workspace = create_workspace_without_llm(&state).await;
    let app = Server::new(create_test_config(), state).build_router();

    let body = json!({
        "message": "Hello default lineage",
        "mode": "bypass",
        "stream": false
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/chat/completions")
                .header("Content-Type", "application/json")
                .header("X-Workspace-ID", workspace.workspace_id.to_string())
                .header("X-Tenant-Id", workspace.tenant_id.to_string())
                .header("X-User-Id", Uuid::new_v4().to_string())
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let parsed = extract_json(response).await;
    let provider = parsed["llm_provider"].as_str().expect("llm_provider");
    let model = parsed["llm_model"].as_str().expect("llm_model");
    assert!(!provider.is_empty());
    assert!(!model.is_empty());
}
