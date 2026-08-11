//! Effective LLM lineage for chat/query responses (SPEC-032 honesty).
//!
//! When resolution falls through to the server default (`Ok(None)`), callers must
//! still record the provider/model that **actually** answered — not leave
//! `llm_provider` / `llm_model` null for the UI metadata bar.

use edgequake_llm::traits::LLMProvider;

/// Coalesce resolved lineage with the effective provider that will run the query.
///
/// - Prefer non-empty `used_*` from workspace/request resolution.
/// - Otherwise use `effective.name()` / `effective.model()` (server default or override).
pub(crate) fn coalesce_effective_llm_lineage(
    used_provider: Option<String>,
    used_model: Option<String>,
    effective: &dyn LLMProvider,
) -> (String, String) {
    let provider = used_provider
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| effective.name().to_string());
    let model = used_model
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| effective.model().to_string());
    (provider, model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_llm::MockProvider;

    #[test]
    fn prefers_resolved_lineage_when_present() {
        let mock = MockProvider::new();
        let (p, m) = coalesce_effective_llm_lineage(
            Some("ollama".into()),
            Some("gemma3:latest".into()),
            &mock,
        );
        assert_eq!(p, "ollama");
        assert_eq!(m, "gemma3:latest");
    }

    #[test]
    fn fills_server_default_when_resolution_empty() {
        let mock = MockProvider::new();
        let (p, m) = coalesce_effective_llm_lineage(None, None, &mock);
        assert_eq!(p, "mock");
        assert_eq!(m, "mock-model");
    }

    #[test]
    fn treats_blank_strings_as_missing() {
        let mock = MockProvider::new();
        let (p, m) = coalesce_effective_llm_lineage(Some("  ".into()), Some("".into()), &mock);
        assert_eq!(p, "mock");
        assert_eq!(m, "mock-model");
    }
}
