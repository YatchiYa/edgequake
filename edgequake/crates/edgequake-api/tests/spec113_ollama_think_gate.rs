//! SPEC-113 — Ollama thinking capability gate (EdgeQuake integration).
//!
//! T-113-19/20: models search `supports_thinking` follows tags capabilities;
//! Auto chat via `edgequake-llm` omits `think` for VL-class fixtures.

use edgequake_llm::{ChatMessage, LLMProvider, OllamaProvider};
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
            "model": "qwen3-vl:8b",
            "message": { "role": "assistant", "content": "ok" },
            "done": true,
            "prompt_eval_count": 1,
            "eval_count": 1
        }))
    }
}

#[tokio::test]
async fn t113_19_tags_capabilities_map_to_supports_thinking() {
    use edgequake_llm::discovery::providers::ollama::OllamaDiscovery;
    use edgequake_llm::discovery::ModelDiscoveryProvider;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [
                {
                    "name": "qwen3-vl:8b",
                    "model": "qwen3-vl:8b",
                    "capabilities": ["completion", "vision"]
                },
                {
                    "name": "qwen3:8b",
                    "model": "qwen3:8b",
                    "capabilities": ["completion", "tools", "thinking"]
                }
            ]
        })))
        .mount(&server)
        .await;

    // Discovery reads OLLAMA_HOST — point at mock.
    std::env::set_var("OLLAMA_HOST", server.uri());
    let discovery = OllamaDiscovery::with_host(server.uri());
    let models = discovery.discover_models().await.expect("discover");
    std::env::remove_var("OLLAMA_HOST");

    let vl = models.iter().find(|m| m.id == "qwen3-vl:8b").expect("vl");
    let text = models.iter().find(|m| m.id == "qwen3:8b").expect("text");
    assert!(!vl.capabilities.supports_thinking);
    assert!(text.capabilities.supports_thinking);

    // Catalog DTO honesty (same mapping used by models list).
    let vl_resp = edgequake_api::model_catalog::discovered_to_response(vl);
    assert!(!vl_resp.capabilities.supports_thinking);
    assert!(vl_resp.capabilities.reasoning_effort.is_none());
    let text_resp = edgequake_api::model_catalog::discovered_to_response(text);
    assert!(text_resp.capabilities.supports_thinking);
    assert!(text_resp.capabilities.reasoning_effort.is_some());
}

#[tokio::test]
async fn t113_20_auto_chat_vl_omits_think() {
    let server = MockServer::start().await;
    let bodies = Arc::new(Mutex::new(Vec::new()));

    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "capabilities": ["completion", "vision"],
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
        .model("qwen3-vl:8b")
        .build()
        .unwrap();

    let resp = provider
        .chat(&[ChatMessage::user("ping")], None)
        .await
        .expect("VL Auto chat must succeed without think injection");
    assert_eq!(resp.content, "ok");

    let captured = bodies.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert!(
        captured[0].get("think").is_none(),
        "outbound chat must omit think for VL caps: {}",
        captured[0]
    );
}
