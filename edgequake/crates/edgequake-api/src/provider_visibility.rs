//! Provider visibility + runtime ban for Mock (SPEC-043 / app safety).
//!
//! Mock providers must never appear in WebUI pickers and must never be used by
//! the running application for LLM, embedding, or vision work. Tests may opt in
//! via [`ALLOW_MOCK_PROVIDER_ENV`] or the existing test-provider override.

use std::collections::HashSet;

use edgequake_llm::model_config::ProviderConfig;

/// Provider IDs that must never appear in user-facing API responses or WebUI.
pub const UI_HIDDEN_PROVIDER_IDS: &[&str] = &["mock", "mock-imagegen"];

/// Opt-in escape hatch for integration tests that intentionally exercise Mock
/// through `create_safe_*` factories. Never set this in application runtime.
pub const ALLOW_MOCK_PROVIDER_ENV: &str = "EDGEQUAKE_ALLOW_MOCK_PROVIDER";

/// Returns true when the provider is the internal mock/test integration.
pub fn is_mock_provider(provider_id: &str) -> bool {
    matches!(
        provider_id.trim().to_lowercase().as_str(),
        "mock" | "mock-imagegen"
    )
}

/// Whether Mock may be constructed via application provider factories.
///
/// Allowed only when explicitly opted in for tests (`EDGEQUAKE_ALLOW_MOCK_PROVIDER=1`
/// or `EDGEQUAKE_ALLOW_TEST_PROVIDER_OVERRIDE=1`).
pub fn mock_provider_allowed() -> bool {
    matches!(
        std::env::var(ALLOW_MOCK_PROVIDER_ENV).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    ) || matches!(
        std::env::var("EDGEQUAKE_ALLOW_TEST_PROVIDER_OVERRIDE").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

/// Reject Mock for application runtime provider construction.
pub fn ensure_non_mock_provider(provider_id: &str, role: &str) -> Result<(), String> {
    if is_mock_provider(provider_id) && !mock_provider_allowed() {
        Err(format!(
            "Mock {role} provider is forbidden in the EdgeQuake application. \
             Configure a real provider (ollama, openai, mistral, …) on the workspace/tenant \
             or set EDGEQUAKE_LLM_PROVIDER / EDGEQUAKE_EMBEDDING_PROVIDER. \
             (refusing provider='{provider_id}')"
        ))
    } else {
        Ok(())
    }
}

/// Heal a persisted/API Mock provider id to a real provider.
///
/// Prefer detecting from the paired model name (e.g. `embeddinggemma:latest` → ollama),
/// otherwise fall back to the server non-mock default. Empty / `"none"` pass through.
pub fn heal_mock_provider_id(provider_id: &str, model: Option<&str>) -> String {
    if provider_id.is_empty() || provider_id.eq_ignore_ascii_case("none") {
        return provider_id.to_string();
    }
    if !is_mock_provider(provider_id) || mock_provider_allowed() {
        return provider_id.to_string();
    }

    let healed = model
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .map(edgequake_core::Workspace::detect_provider_from_model)
        .filter(|p| !is_mock_provider(p))
        .unwrap_or_else(edgequake_core::Workspace::non_mock_fallback_provider);

    tracing::warn!(
        requested = provider_id,
        model = model.unwrap_or(""),
        healed = %healed,
        "Mock provider in workspace/tenant config — healing to a real provider"
    );
    healed
}

/// Heal optional provider fields from create/update requests.
pub fn heal_optional_mock_provider(
    provider: Option<String>,
    model: Option<&str>,
) -> Option<String> {
    provider.map(|p| heal_mock_provider_id(&p, model))
}

/// Whether a provider ID may be shown in pickers, settings, and catalog APIs.
pub fn is_ui_visible_provider_id(provider_id: &str) -> bool {
    !is_mock_provider(provider_id)
}

/// Whether a configured provider may be shown in the UI.
pub fn is_ui_visible_provider(provider: &ProviderConfig) -> bool {
    provider.enabled && is_ui_visible_provider_id(&provider.name)
}

/// Filter enabled providers for UI, respecting allowlist and mock exclusion.
pub fn filter_ui_providers<'a>(
    providers: &'a [ProviderConfig],
    allowed: &Option<HashSet<String>>,
) -> Vec<&'a ProviderConfig> {
    crate::model_catalog::filter_providers(providers, allowed)
        .into_iter()
        .filter(|p| is_ui_visible_provider_id(&p.name))
        .collect()
}

/// Chat-capable provider IDs from edgequake-llm that EdgeQuake should expose via models.toml.
pub fn expected_chat_provider_ids() -> Vec<&'static str> {
    edgequake_llm::provider_catalog::ProviderCatalog::list_llm_providers()
        .into_iter()
        .filter(|id| is_ui_visible_provider_id(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_is_hidden_from_ui() {
        assert!(is_mock_provider("mock"));
        assert!(is_mock_provider("MOCK"));
        assert!(is_ui_visible_provider_id("openai"));
        assert!(!is_ui_visible_provider_id("mock"));
    }

    #[test]
    fn mock_is_rejected_unless_explicitly_allowed() {
        std::env::remove_var(ALLOW_MOCK_PROVIDER_ENV);
        std::env::remove_var("EDGEQUAKE_ALLOW_TEST_PROVIDER_OVERRIDE");
        assert!(ensure_non_mock_provider("mock", "LLM").is_err());
        assert!(ensure_non_mock_provider("ollama", "LLM").is_ok());
    }

    #[test]
    fn heal_mock_provider_detects_ollama_from_colon_model() {
        std::env::remove_var(ALLOW_MOCK_PROVIDER_ENV);
        std::env::remove_var("EDGEQUAKE_ALLOW_TEST_PROVIDER_OVERRIDE");
        assert_eq!(
            heal_mock_provider_id("mock", Some("embeddinggemma:latest")),
            "ollama"
        );
        assert_eq!(
            heal_mock_provider_id("mistral", Some("magistral-small-latest")),
            "mistral"
        );
    }

    #[test]
    fn filter_ui_providers_excludes_mock() {
        use edgequake_llm::ProviderConfig;
        let providers = vec![
            ProviderConfig {
                name: "openai".into(),
                enabled: true,
                ..ProviderConfig::default()
            },
            ProviderConfig {
                name: "mock".into(),
                enabled: true,
                ..ProviderConfig::default()
            },
        ];
        let filtered = filter_ui_providers(&providers, &None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "openai");
    }
}
