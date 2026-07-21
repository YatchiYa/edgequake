//! LLM role resolution for workspace-scoped provider selection (SPEC-026 Phase 2 P-08 /
//! SPEC-046 EQ-046-13).
//!
//! Mirrors LightRAG `llm_roles.py` fallback: role config → workspace default.
//!
//! # Recommended default matrix (First Principles)
//!
//! | Role | Capability need | Typical model class |
//! |------|-----------------|---------------------|
//! | **Extract** | Structured NER/RE, schema adherence | Stronger / slower |
//! | **Query** | Long noisy context → faithful answer | Long-context strong |
//! | **Summary** | Merge entity/relation descriptions | Mid / cheap |
//! | **Vlm** | Image/table/equation understanding | Vision model |
//! | **Keyword** | Dual-level hl/ll keywords (optional) | Small / fast |
//!
//! Configure via workspace metadata:
//! ```json
//! { "llm_roles": {
//!     "extract": { "provider": "openai", "model": "gpt-5-nano" },
//!     "query":   { "provider": "openai", "model": "gpt-5-nano" },
//!     "summary": { "provider": "ollama", "model": "gemma3:latest" },
//!     "vlm":     { "provider": "ollama", "model": "llava" },
//!     "keyword": { "provider": "ollama", "model": "gemma3:latest" }
//! }}
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::Workspace;

/// LLM functional roles used during ingest and query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmRole {
    Extract,
    Query,
    Summary,
    /// Vision / multimodal analysis (LightRAG `vlm` role).
    Vlm,
    /// Dual-level keyword extraction at query time (SPEC-046 / LightRAG keyword role).
    Keyword,
}

impl LlmRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Extract => "extract",
            Self::Query => "query",
            Self::Summary => "summary",
            Self::Vlm => "vlm",
            Self::Keyword => "keyword",
        }
    }

    /// All roles for docs / iteration.
    pub fn all() -> &'static [LlmRole] {
        &[
            Self::Extract,
            Self::Query,
            Self::Summary,
            Self::Vlm,
            Self::Keyword,
        ]
    }
}

/// Per-role provider override stored in workspace metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleLlmConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
}

/// Resolved provider + model for a role (always populated after fallback).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoleLlm {
    pub provider: String,
    pub model: String,
}

/// Read role overrides from workspace metadata key `llm_roles`.
pub fn role_config_from_workspace(ws: &Workspace, role: LlmRole) -> Option<RoleLlmConfig> {
    let roles = ws.metadata.get("llm_roles")?;
    let role_obj = roles.get(role.as_str())?;
    serde_json::from_value(role_obj.clone()).ok()
}

/// Resolve provider/model with LightRAG-style fallback chain.
pub fn resolve_role_llm(ws: &Workspace, role: LlmRole) -> ResolvedRoleLlm {
    if let Some(cfg) = role_config_from_workspace(ws, role) {
        let provider = cfg
            .provider
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| default_provider_for_role(ws, role));
        let model = cfg
            .model
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| default_model_for_role(ws, role));
        return ResolvedRoleLlm { provider, model };
    }
    ResolvedRoleLlm {
        provider: default_provider_for_role(ws, role),
        model: default_model_for_role(ws, role),
    }
}

fn default_provider_for_role(ws: &Workspace, role: LlmRole) -> String {
    match role {
        LlmRole::Vlm => ws
            .vision_llm_provider
            .clone()
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| ws.llm_provider.clone()),
        // Keyword/Summary default to workspace LLM provider (cheap local OK)
        LlmRole::Keyword | LlmRole::Summary | LlmRole::Extract | LlmRole::Query => {
            ws.llm_provider.clone()
        }
    }
}

fn default_model_for_role(ws: &Workspace, role: LlmRole) -> String {
    match role {
        LlmRole::Vlm => ws
            .vision_llm_model
            .clone()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| ws.llm_model.clone()),
        LlmRole::Keyword | LlmRole::Summary | LlmRole::Extract | LlmRole::Query => {
            ws.llm_model.clone()
        }
    }
}

/// Human-readable capability hint for ops / docs (SPEC-046 EQ-046-13).
pub fn role_capability_hint(role: LlmRole) -> &'static str {
    match role {
        LlmRole::Extract => "strong structured extraction (NER/RE)",
        LlmRole::Query => "long-context faithful generation",
        LlmRole::Summary => "cheap merge/summarization of entity descriptions",
        LlmRole::Vlm => "vision / table / equation understanding",
        LlmRole::Keyword => "fast dual-level keyword extraction",
    }
}

/// Process-env KEYWORD role (LightRAG `KEYWORD_LLM_*` law).
///
/// Env (either MODEL or PROVIDER non-empty enables the override):
/// - `EDGEQUAKE_KEYWORD_LLM_MODEL` — required for a usable pin (empty = unset)
/// - `EDGEQUAKE_KEYWORD_LLM_PROVIDER` — optional; falls back to `EDGEQUAKE_LLM_PROVIDER`
///   then `"mistral"` when only the model is set
///
/// Precedence at query time: **env → workspace `llm_roles.keyword` → Query LLM**.
pub fn env_keyword_role_llm() -> Option<ResolvedRoleLlm> {
    let model = non_empty_env("EDGEQUAKE_KEYWORD_LLM_MODEL");
    let provider = non_empty_env("EDGEQUAKE_KEYWORD_LLM_PROVIDER");
    match (provider, model) {
        (None, None) => None,
        (Some(provider), Some(model)) => Some(ResolvedRoleLlm { provider, model }),
        (None, Some(model)) => {
            let provider = non_empty_env("EDGEQUAKE_LLM_PROVIDER")
                .unwrap_or_else(|| "mistral".to_string());
            Some(ResolvedRoleLlm { provider, model })
        }
        (Some(provider), None) => {
            // Provider-only: use that provider's conventional default model name.
            let model = match provider.to_ascii_lowercase().as_str() {
                "mistral" => "ministral-3b-latest".to_string(),
                "openai" => "gpt-5.4-nano".to_string(),
                "ollama" => "gemma3:latest".to_string(),
                other => other.to_string(),
            };
            Some(ResolvedRoleLlm { provider, model })
        }
    }
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Parse full llm_roles map from metadata (for tests / API).
pub fn parse_llm_roles_map(
    metadata: &HashMap<String, serde_json::Value>,
) -> HashMap<String, RoleLlmConfig> {
    let Some(raw) = metadata.get("llm_roles") else {
        return HashMap::new();
    };
    raw.as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| {
                    serde_json::from_value(v.clone())
                        .ok()
                        .map(|cfg| (k.clone(), cfg))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use uuid::Uuid;

    fn sample_workspace(metadata: HashMap<String, serde_json::Value>) -> Workspace {
        Workspace {
            workspace_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "test".into(),
            slug: "test".into(),
            description: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata,
            llm_model: "gemma3:latest".into(),
            llm_provider: "ollama".into(),
            embedding_model: "embeddinggemma:latest".into(),
            embedding_provider: "ollama".into(),
            embedding_dimension: 768,
            vision_llm_model: None,
            vision_llm_provider: None,
            pdf_parser_backend: None,
        }
    }

    #[test]
    fn resolve_extract_role_uses_configured_provider() {
        let mut meta = HashMap::new();
        meta.insert(
            "llm_roles".into(),
            serde_json::json!({
                "extract": { "provider": "mock", "model": "mock-extract" }
            }),
        );
        let ws = sample_workspace(meta);
        let resolved = resolve_role_llm(&ws, LlmRole::Extract);
        assert_eq!(resolved.provider, "mock");
        assert_eq!(resolved.model, "mock-extract");
    }

    #[test]
    fn resolve_query_role_falls_back_to_workspace_default() {
        let ws = sample_workspace(HashMap::new());
        let resolved = resolve_role_llm(&ws, LlmRole::Query);
        assert_eq!(resolved.provider, "ollama");
        assert_eq!(resolved.model, "gemma3:latest");
    }

    #[test]
    fn resolve_vlm_falls_back_to_vision_fields() {
        let mut ws = sample_workspace(HashMap::new());
        ws.vision_llm_provider = Some("openai".into());
        ws.vision_llm_model = Some("gpt-4.1-mini".into());
        let resolved = resolve_role_llm(&ws, LlmRole::Vlm);
        assert_eq!(resolved.provider, "openai");
        assert_eq!(resolved.model, "gpt-4.1-mini");
    }

    #[test]
    fn resolve_vlm_role_prefers_llm_roles_vlm() {
        let mut meta = HashMap::new();
        meta.insert(
            "llm_roles".into(),
            serde_json::json!({
                "vlm": { "provider": "mock", "model": "mock-vlm" }
            }),
        );
        let ws = sample_workspace(meta);
        let resolved = resolve_role_llm(&ws, LlmRole::Vlm);
        assert_eq!(resolved.provider, "mock");
        assert_eq!(resolved.model, "mock-vlm");
    }

    #[test]
    fn keyword_and_summary_roles_resolve_and_have_hints() {
        let mut meta = HashMap::new();
        meta.insert(
            "llm_roles".into(),
            serde_json::json!({
                "keyword": { "provider": "ollama", "model": "gemma3:latest" },
                "summary": { "provider": "mock", "model": "mock-summary" }
            }),
        );
        let ws = sample_workspace(meta);
        let kw = resolve_role_llm(&ws, LlmRole::Keyword);
        assert_eq!(kw.model, "gemma3:latest");
        let sum = resolve_role_llm(&ws, LlmRole::Summary);
        assert_eq!(sum.provider, "mock");
        assert!(!role_capability_hint(LlmRole::Keyword).is_empty());
        assert_eq!(LlmRole::all().len(), 5);
    }

    #[test]
    #[serial]
    fn env_keyword_role_unset_when_empty() {
        // Isolate from ambient Acc/dev env (serialize — env is process-global).
        std::env::remove_var("EDGEQUAKE_KEYWORD_LLM_PROVIDER");
        std::env::remove_var("EDGEQUAKE_KEYWORD_LLM_MODEL");
        assert!(env_keyword_role_llm().is_none());
        std::env::set_var("EDGEQUAKE_KEYWORD_LLM_MODEL", "ministral-3b-latest");
        let role = env_keyword_role_llm().expect("model-only pin");
        assert_eq!(role.model, "ministral-3b-latest");
        assert!(!role.provider.is_empty());
        std::env::remove_var("EDGEQUAKE_KEYWORD_LLM_MODEL");
        std::env::set_var("EDGEQUAKE_KEYWORD_LLM_PROVIDER", "mistral");
        std::env::set_var("EDGEQUAKE_KEYWORD_LLM_MODEL", "ministral-3b-latest");
        let role = env_keyword_role_llm().expect("full pin");
        assert_eq!(role.provider, "mistral");
        assert_eq!(role.model, "ministral-3b-latest");
        std::env::remove_var("EDGEQUAKE_KEYWORD_LLM_PROVIDER");
        std::env::remove_var("EDGEQUAKE_KEYWORD_LLM_MODEL");
    }
}
