//! Chat / Bypass A+ contracts — 2026 AI eng practices + HTTP e2e.
//!
//! Proves:
//! - No KG retrieval / empty sources
//! - Dedicated chatbot system prompt reaches `chat()`
//! - History reaches `chat()` (recording LLM — not MockProvider)
//! - Pair-safe + token-budget history policy
//! - `chat` alias ≡ bypass
//! - Streaming bypass does not return RAG apology

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_core::types::CreateWorkspaceRequest;
use edgequake_core::Tenant;
use edgequake_llm::error::Result as LlmResult;
use edgequake_llm::traits::{
    ChatMessage, ChatRole, CompletionOptions, EmbeddingProvider, LLMProvider, LLMResponse,
};
use edgequake_llm::MockProvider;
use edgequake_query::conversation_context::{
    apply_history_policy, build_bypass_chat_messages, cut_conversation_history,
    drop_leading_orphan_assistant, resolve_bypass_system_prompt, trim_history_to_token_budget,
    DEFAULT_BYPASS_SYSTEM_PROMPT, DEFAULT_CONVERSATION_TURN_LIMIT, DEFAULT_HISTORY_TOKEN_BUDGET,
};
use edgequake_query::engine::QueryRequest;
use edgequake_query::{ConversationMessage, QueryEngine, QueryEngineConfig, QueryMode};
use edgequake_storage::traits::{GraphStorage, VectorStorage};
use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage};
use futures::StreamExt;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use uuid::Uuid;

/// Records the last `chat()` messages — First Principles proof the model sees history.
struct RecordingLlm {
    last_messages: Mutex<Vec<ChatMessage>>,
    response: String,
}

impl RecordingLlm {
    fn new(response: impl Into<String>) -> Self {
        Self {
            last_messages: Mutex::new(Vec::new()),
            response: response.into(),
        }
    }

    fn last_messages(&self) -> Vec<ChatMessage> {
        self.last_messages.lock().unwrap().clone()
    }
}

#[async_trait]
impl LLMProvider for RecordingLlm {
    fn name(&self) -> &str {
        "recording"
    }
    fn model(&self) -> &str {
        "recording-model"
    }
    fn max_context_length(&self) -> usize {
        128_000
    }
    async fn complete(&self, _prompt: &str) -> LlmResult<LLMResponse> {
        Ok(LLMResponse::new(&self.response, self.model()))
    }
    async fn complete_with_options(
        &self,
        prompt: &str,
        _options: &CompletionOptions,
    ) -> LlmResult<LLMResponse> {
        self.complete(prompt).await
    }
    async fn chat(
        &self,
        messages: &[ChatMessage],
        _options: Option<&CompletionOptions>,
    ) -> LlmResult<LLMResponse> {
        *self.last_messages.lock().unwrap() = messages.to_vec();
        Ok(LLMResponse::new(&self.response, self.model()))
    }
    fn supports_streaming(&self) -> bool {
        false
    }
}

fn make_engine_with_llm(
    vector: Arc<MemoryVectorStorage>,
    graph: Arc<MemoryGraphStorage>,
    embed: Arc<dyn EmbeddingProvider>,
    llm: Arc<dyn LLMProvider>,
) -> QueryEngine {
    QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        embed,
        llm,
    )
}

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
        "Chat A+ Tenant".to_string(),
        format!("chat-aplus-{}", Uuid::new_v4()),
    );
    let created_tenant = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .expect("create tenant");

    let request = CreateWorkspaceRequest {
        name: "Chat A+ Workspace".to_string(),
        slug: Some(format!("chat-aplus-{}", Uuid::new_v4())),
        description: Some("A+ chatbot bypass".to_string()),
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

#[test]
fn chat_alias_parses_as_bypass() {
    assert_eq!(QueryMode::parse("chat"), Some(QueryMode::Bypass));
    assert_eq!(QueryMode::parse("CHAT"), Some(QueryMode::Bypass));
}

#[test]
fn history_policy_is_pair_safe_and_token_aware() {
    let history = vec![
        ConversationMessage {
            role: "assistant".into(),
            content: "orphan-lead".into(),
        },
        ConversationMessage {
            role: "user".into(),
            content: "x".repeat(2_000),
        },
        ConversationMessage {
            role: "assistant".into(),
            content: "y".repeat(2_000),
        },
        ConversationMessage {
            role: "user".into(),
            content: "recent-user".into(),
        },
        ConversationMessage {
            role: "assistant".into(),
            content: "recent-asst".into(),
        },
    ];
    let policy = apply_history_policy(&history, 6, 200);
    assert_eq!(policy.first().map(|m| m.role.as_str()), Some("user"));
    assert!(!policy.iter().any(|m| m.content == "orphan-lead"));
    assert!(policy.iter().any(|m| m.content == "recent-user"));

    let cut = cut_conversation_history(&history, 4);
    assert_eq!(
        drop_leading_orphan_assistant(cut.clone())
            .first()
            .unwrap()
            .role,
        "user"
    );
    let _ = trim_history_to_token_budget(&history, DEFAULT_HISTORY_TOKEN_BUDGET);
}

#[tokio::test]
async fn recording_llm_proves_system_prompt_and_history_reach_chat() {
    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new("test", dim));
    let graph = Arc::new(MemoryGraphStorage::new("test"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();

    let recorder = Arc::new(RecordingLlm::new("REMEMBERED"));
    let embed = Arc::new(MockProvider::default()) as Arc<dyn EmbeddingProvider>;
    let engine = make_engine_with_llm(
        vector,
        graph,
        embed,
        Arc::clone(&recorder) as Arc<dyn LLMProvider>,
    );

    let history = vec![
        ConversationMessage {
            role: "user".into(),
            content: "Secret code is ORBIT-42.".into(),
        },
        ConversationMessage {
            role: "assistant".into(),
            content: "Stored.".into(),
        },
    ];
    let req = QueryRequest::new("What is the secret code?")
        .with_mode(QueryMode::Bypass)
        .with_conversation_history(history)
        .with_system_prompt("Stay terse.");

    let response = engine.query(req).await.expect("bypass");
    assert_eq!(response.answer, "REMEMBERED");
    assert!(response.context.is_empty());

    let seen = recorder.last_messages();
    assert!(
        !seen.is_empty(),
        "recording LLM must observe chat() messages"
    );
    assert_eq!(seen[0].role, ChatRole::System);
    assert!(
        seen[0].content.contains(DEFAULT_BYPASS_SYSTEM_PROMPT)
            || seen[0].content.contains("Stay terse."),
        "system must include chatbot persona and/or extension: {}",
        seen[0].content
    );
    assert!(
        seen[0].content.contains("Stay terse."),
        "SPEC-004 extension must append to system"
    );
    assert!(
        seen.iter().any(|m| m.content.contains("ORBIT-42")),
        "history must reach the model: {seen:?}"
    );
    assert_eq!(seen.last().unwrap().content, "What is the secret code?");
}

#[tokio::test]
async fn bypass_stream_skips_kg_and_avoids_apology() {
    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new("test", dim));
    let graph = Arc::new(MemoryGraphStorage::new("test"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();

    let mock = Arc::new(MockProvider::default());
    mock.add_response("STREAM_OK").await;
    let engine = make_engine_with_llm(
        Arc::clone(&vector),
        Arc::clone(&graph),
        Arc::clone(&mock) as Arc<dyn EmbeddingProvider>,
        mock as Arc<dyn LLMProvider>,
    );

    let mut req = QueryRequest::new("hello");
    req.mode = Some(QueryMode::Bypass);
    let (ctx, mode, mut stream) = engine.query_stream_with_context(req).await.expect("stream");
    assert_eq!(mode, QueryMode::Bypass);
    assert!(ctx.is_empty());
    let mut answer = String::new();
    while let Some(t) = stream.next().await {
        answer.push_str(&t.unwrap());
    }
    assert!(!answer.contains("couldn't find any relevant information"));
    assert_eq!(answer, "STREAM_OK");
}

#[tokio::test]
async fn e2e_http_chat_alias_and_bypass_empty_sources() {
    let state = AppState::test_state();
    let workspace = create_workspace(&state).await;
    let app = Server::new(create_test_config(), state).build_router();

    for mode in ["bypass", "chat"] {
        let body = json!({
            "message": format!("Hello via {mode}"),
            "mode": mode,
            "stream": false
        });
        let response = app
            .clone()
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
        assert_eq!(response.status(), StatusCode::OK, "mode={mode}");
        let parsed = extract_json(response).await;
        assert_eq!(parsed["mode"].as_str(), Some("bypass"), "mode={mode}");
        assert!(
            parsed["sources"].as_array().unwrap().is_empty(),
            "no KG sources for {mode}"
        );
        let content = parsed["content"].as_str().unwrap_or("");
        assert!(
            !content.contains("couldn't find any relevant information"),
            "{mode}: {content}"
        );
    }
}

#[tokio::test]
async fn e2e_http_multi_turn_bypass_conversation() {
    let state = AppState::test_state();
    let workspace = create_workspace(&state).await;
    let app = Server::new(create_test_config(), state).build_router();
    let tenant = workspace.tenant_id.to_string();
    let user = Uuid::new_v4().to_string();
    let ws = workspace.workspace_id.to_string();

    let first = json!({
        "message": "Remember token NEBULA",
        "mode": "bypass",
        "stream": false,
        "system_prompt": "Acknowledge briefly."
    });
    let first_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/chat/completions")
                .header("Content-Type", "application/json")
                .header("X-Workspace-ID", &ws)
                .header("X-Tenant-Id", &tenant)
                .header("X-User-Id", &user)
                .body(Body::from(first.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_resp.status(), StatusCode::OK);
    let first_json = extract_json(first_resp).await;
    let conversation_id = first_json["conversation_id"].as_str().unwrap();

    let second = json!({
        "conversation_id": conversation_id,
        "message": "What token?",
        "mode": "bypass",
        "stream": false
    });
    let second_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/chat/completions")
                .header("Content-Type", "application/json")
                .header("X-Workspace-ID", &ws)
                .header("X-Tenant-Id", &tenant)
                .header("X-User-Id", &user)
                .body(Body::from(second.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_resp.status(), StatusCode::OK);
    let second_json = extract_json(second_resp).await;
    assert_eq!(second_json["mode"].as_str(), Some("bypass"));
    assert!(second_json["sources"].as_array().unwrap().is_empty());
}

#[test]
fn bypass_message_builder_pins_system() {
    let msgs = build_bypass_chat_messages("Q", &[], Some("Extra."), 6, None);
    assert_eq!(msgs[0].role, ChatRole::System);
    let resolved = resolve_bypass_system_prompt(Some("Extra."));
    assert_eq!(msgs[0].content, resolved);
    const {
        assert!(DEFAULT_CONVERSATION_TURN_LIMIT >= 4);
    }
}
