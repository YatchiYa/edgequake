//! SPEC-126 e2e (workspace): Native OpenAI sends GPT-5.6 breakpoints; Compatible does not.
//!
//! Runs in CI via `cargo nextest run -p edgequake-api --test e2e_spec126_prompt_cache`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use edgequake_llm::{ChatMessage, CompletionOptions, LLMProvider, OpenAIProvider};
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

fn chat_ok() -> Value {
    serde_json::json!({
        "id": "chatcmpl-spec126",
        "object": "chat.completion",
        "created": 0,
        "model": "test",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "ok" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 20,
            "completion_tokens": 1,
            "total_tokens": 21,
            "prompt_tokens_details": { "cached_tokens": 0, "cache_write_tokens": 18 }
        }
    })
}

struct CapturingJson {
    bodies: Arc<std::sync::Mutex<Vec<Value>>>,
    hits: AtomicUsize,
    responder: Box<dyn Fn(&Value) -> ResponseTemplate + Send + Sync>,
}

impl wiremock::Respond for CapturingJson {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        self.hits.fetch_add(1, Ordering::SeqCst);
        let body = serde_json::from_slice::<Value>(&request.body).unwrap_or(Value::Null);
        if let Ok(mut g) = self.bodies.lock() {
            g.push(body.clone());
        }
        (self.responder)(&body)
    }
}

fn extract_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage::system("Stable extract instructions."),
        ChatMessage::user("unique chunk"),
    ]
}

#[tokio::test]
async fn e2e_native_openai_sends_explicit_cache_fields() {
    let server = MockServer::start().await;
    let bodies = Arc::new(std::sync::Mutex::new(Vec::new()));
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(CapturingJson {
            bodies: Arc::clone(&bodies),
            hits: AtomicUsize::new(0),
            responder: Box::new(|_| ResponseTemplate::new(200).set_body_json(chat_ok())),
        })
        .mount(&server)
        .await;

    let provider = OpenAIProvider::new("sk-test")
        .with_base_url(format!("{}/v1", server.uri()))
        .with_model("eq-spec126-ws-native");
    let opts = CompletionOptions::default().with_role_cache("extract", &provider);
    provider
        .chat(&extract_messages(), Some(&opts))
        .await
        .expect("native chat");

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["prompt_cache_options"]["mode"], "explicit");
    assert_eq!(
        bodies[0]["messages"][0]["content"][0]["prompt_cache_breakpoint"]["mode"],
        "explicit"
    );
}

#[tokio::test]
async fn e2e_compatible_openai_omits_explicit_cache_fields() {
    let server = MockServer::start().await;
    let bodies = Arc::new(std::sync::Mutex::new(Vec::new()));
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(CapturingJson {
            bodies: Arc::clone(&bodies),
            hits: AtomicUsize::new(0),
            responder: Box::new(|_| ResponseTemplate::new(200).set_body_json(chat_ok())),
        })
        .mount(&server)
        .await;

    let provider = OpenAIProvider::compatible("sk-test", format!("{}/v1", server.uri()))
        .with_model("mistral-small-latest");
    let opts = CompletionOptions::default().with_role_cache("extract", &provider);
    provider
        .chat(&extract_messages(), Some(&opts))
        .await
        .expect("compat chat");

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1);
    assert!(bodies[0].get("prompt_cache_options").is_none());
    assert!(bodies[0].get("prompt_cache_key").is_some());
}

#[tokio::test]
async fn e2e_native_retries_after_structured_param_400() {
    let server = MockServer::start().await;
    let bodies = Arc::new(std::sync::Mutex::new(Vec::new()));
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(CapturingJson {
            bodies: Arc::clone(&bodies),
            hits: AtomicUsize::new(0),
            responder: Box::new(|body| {
                if body.get("prompt_cache_options").is_some() {
                    ResponseTemplate::new(400).set_body_json(serde_json::json!({
                        "error": {
                            "message": "Unknown parameter",
                            "type": "invalid_request_error",
                            "param": "prompt_cache_options",
                            "code": "unknown_parameter"
                        }
                    }))
                } else {
                    ResponseTemplate::new(200).set_body_json(chat_ok())
                }
            }),
        })
        .mount(&server)
        .await;

    let provider = OpenAIProvider::new("sk-test")
        .with_base_url(format!("{}/v1", server.uri()))
        .with_model("eq-spec126-ws-unaware");
    let opts = CompletionOptions::default().with_role_cache("extract", &provider);
    provider
        .chat(&extract_messages(), Some(&opts))
        .await
        .expect("retry");

    let bodies = bodies.lock().unwrap();
    assert!(bodies.len() >= 2);
    assert!(bodies[0].get("prompt_cache_options").is_some());
    assert!(bodies[1].get("prompt_cache_options").is_none());
}

#[test]
fn contract_extract_and_glean_use_shared_prompt_cache_chat() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-pipeline/src/extractor");
    let glean = std::fs::read_to_string(root.join("gleaning.rs")).unwrap();
    let extract = std::fs::read_to_string(root.join("llm.rs")).unwrap();
    let options = std::fs::read_to_string(root.join("completion_options.rs")).unwrap();
    assert!(
        glean.contains("extraction_completion_options")
            && glean.contains(".chat(")
            && glean.contains("with_provider_prompt_cache"),
        "gleaning must chat with shared extract options (SPEC-126)"
    );
    assert!(
        extract.contains(".chat(") && extract.contains("extraction_completion_options"),
        "extract llm.rs must chat with shared extract options (SPEC-126)"
    );
    assert!(
        options.contains("with_provider_prompt_cache(\"extract\""),
        "extraction CompletionOptions must set eq:extract prompt_cache_key (SPEC-126)"
    );
}
