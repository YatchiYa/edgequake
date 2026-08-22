//! SPEC-109 contract: reasoning effort resolution + API DTO shape.
//!
//! Gates: E2E-109-02 (clamp), E2E-109-03 (extract floor), E2E-109-05 (mistral-large),
//! E2E-109-06 (effective config reasoning_roles).

use edgequake_api::config_resolution::build_effective_config;
use edgequake_api::handlers::query_types::QueryRequest;
use edgequake_api::server_config_store::ServerConfigSnapshot;
use edgequake_api::services::resolve_extract_reasoning_effort;
use edgequake_core::{
    install_server_config, resolve_role_reasoning_effort, ConfigPriorityMode, LlmRole,
    ServerLlmDefaults, Workspace,
};
use edgequake_pipeline::{
    extraction_completion_options, extraction_completion_options_with_effort,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn e2e109_02_gpt5_mini_none_clamps_to_minimal() {
    let ws = Workspace::new(Uuid::nil(), "t", "t");
    let r = resolve_role_reasoning_effort(
        LlmRole::Extract,
        "openai",
        "gpt-5-mini",
        &ws,
        Some("none"),
        None,
        None,
        None,
    );
    assert_eq!(r.effective.as_deref(), Some("minimal"));
    assert!(r.clamped);
}

#[test]
fn e2e109_03_extract_default_floor() {
    // gpt-5.4-nano catalog lists `none`; live OpenAI rejects disable — floor is `low`.
    let opts = extraction_completion_options("gpt-5.4-nano", 1024);
    assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));

    let overridden =
        extraction_completion_options_with_effort("gpt-5-mini", 1024, Some("low"), "openai");
    assert_eq!(overridden.reasoning_effort.as_deref(), Some("low"));
}

#[test]
fn e2e109_08_api_resolve_extract_none_env_lifts_for_gpt54_nano() {
    let mut ws = Workspace::new(Uuid::nil(), "t", "t");
    ws.llm_provider = "openai".into();
    ws.llm_model = "gpt-5.4-nano".into();
    let effort = resolve_extract_reasoning_effort(
        Some(&ws),
        "openai",
        "gpt-5.4-nano",
        None,
        Some("none"),
    );
    assert_eq!(effort.as_deref(), Some("low"));
}

#[test]
fn e2e109_07_gpt55_extract_none_lifts_off_disable() {
    // Catalog lists `none` for gpt-5.5; live endpoints reject disable.
    let opts = extraction_completion_options_with_effort("gpt-5.5", 1024, Some("none"), "openai");
    assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));
}

#[test]
fn e2e109_05_mistral_large_omits() {
    let opts = extraction_completion_options_with_effort(
        "mistral-large-latest",
        1024,
        Some("high"),
        "mistral",
    );
    assert!(opts.reasoning_effort.is_none());
}

#[test]
fn e2e109_04_query_request_deserializes_reasoning_effort() {
    let body = json!({
        "query": "hello",
        "reasoning_effort": "low"
    });
    let req: QueryRequest = serde_json::from_value(body).expect("deserialize");
    assert_eq!(req.reasoning_effort.as_deref(), Some("low"));
}

#[test]
fn e2e109_06_effective_config_includes_reasoning_roles() {
    let defaults = ServerLlmDefaults {
        llm_provider: Some("openai".into()),
        llm_model: Some("gpt-5-mini".into()),
        reasoning_effort: None,
        reasoning_by_role: HashMap::new(),
        ..Default::default()
    };
    install_server_config(defaults.clone(), ConfigPriorityMode::ServerFirst);
    let snapshot = ServerConfigSnapshot {
        llm_defaults: defaults,
        priority_mode: ConfigPriorityMode::ServerFirst,
        app_attribution: Default::default(),
        postgres_available: false,
    };
    let effective = build_effective_config(&snapshot);
    let extract = effective
        .reasoning_roles
        .get("extract")
        .expect("extract role present");
    assert_eq!(extract.effective.as_deref(), Some("minimal"));
    assert_eq!(extract.source, "compiled_default");

    let query = effective
        .reasoning_roles
        .get("query")
        .expect("query role present");
    assert!(query.effective.is_none(), "query Auto omits by default");
}
