//! SPEC-113 + extract think-off: outbound Ollama extract must send `think: false`
//! for thinking-capable models (qwen3.6) when effort floors to `none`.
//!
//! First principles: Ollama Auto enables thinking when `think` is unset/`true`.
//! Structured KG extract must disable thinking by default.

use edgequake_llm::OllamaProvider;
use edgequake_pipeline::{EntityExtractor, LLMExtractor, TextChunk};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

struct CapturingChat {
    bodies: Arc<Mutex<Vec<Value>>>,
}

impl wiremock::Respond for CapturingChat {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        if let Ok(v) = serde_json::from_slice::<Value>(&request.body) {
            if let Ok(mut g) = self.bodies.lock() {
                g.push(v);
            }
        }
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "qwen3.6:35b-a3b",
            "message": {
                "role": "assistant",
                "content": "{\"entities\":[{\"name\":\"ALICE\",\"type\":\"PERSON\",\"description\":\"A person\"}],\"relationships\":[]}"
            },
            "done": true,
            "prompt_eval_count": 10,
            "eval_count": 20
        }))
    }
}

#[tokio::test]
async fn e2e_extract_ollama_qwen_sends_think_false() {
    let server = MockServer::start().await;
    let bodies = Arc::new(Mutex::new(Vec::new()));

    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "capabilities": ["completion", "tools", "thinking"],
            "details": { "family": "qwen35moe" }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(CapturingChat {
            bodies: Arc::clone(&bodies),
        })
        .mount(&server)
        .await;

    let provider = OllamaProvider::builder()
        .host(server.uri())
        .model("qwen3.6:35b-a3b")
        .build()
        .expect("ollama provider");

    // Mirror workspace factory: resolve extract effort then bind on extractor.
    let effort = edgequake_api::services::resolve_extract_reasoning_effort(
        None,
        "ollama",
        "qwen3.6:35b-a3b",
        None,
        None,
    );
    assert_eq!(
        effort.as_deref(),
        Some("none"),
        "extract role must floor ollama/qwen to none"
    );

    let extractor = LLMExtractor::new(Arc::new(provider)).with_reasoning_effort(effort);
    let chunk = TextChunk::new("c1", "Alice works at Acme.", 0, 0, 20);

    let result = extractor
        .extract(&chunk)
        .await
        .expect("extract must succeed against mock");
    assert!(
        !result.entities.is_empty(),
        "expected parsed entities from mock JSON"
    );

    let captured = bodies.lock().unwrap();
    assert_eq!(captured.len(), 1, "one /api/chat call");
    let think = captured[0].get("think");
    assert_eq!(
        think,
        Some(&Value::Bool(false)),
        "extract must send think:false, got body={}",
        captured[0]
    );
}

#[tokio::test]
async fn e2e_extract_ollama_unset_effort_still_floors_via_provider_name() {
    let server = MockServer::start().await;
    let bodies = Arc::new(Mutex::new(Vec::new()));

    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "capabilities": ["completion", "thinking"],
            "details": { "family": "qwen3" }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(CapturingChat {
            bodies: Arc::clone(&bodies),
        })
        .mount(&server)
        .await;

    let provider = OllamaProvider::builder()
        .host(server.uri())
        .model("qwen3:8b")
        .build()
        .unwrap();

    // No explicit effort — LLMExtractor uses provider.name() + local floor.
    let extractor = LLMExtractor::new(Arc::new(provider));
    let chunk = TextChunk::new("c2", "Bob lives in Paris.", 0, 0, 19);
    extractor.extract(&chunk).await.expect("extract");

    let captured = bodies.lock().unwrap();
    assert_eq!(
        captured[0].get("think"),
        Some(&Value::Bool(false)),
        "provider-aware floor must send think:false without explicit effort: {}",
        captured[0]
    );
}
