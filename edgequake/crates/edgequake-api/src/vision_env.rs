//! Shared vision provider / model resolution from environment (DRY SSOT).
//!
//! Used by PDF upload, image ingest, and startup `vision_llm_provider` wiring.

/// Read an env var, treating empty strings as unset.
pub fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

/// Server-default vision provider (matches `PdfUploadOptions` resolution chain).
pub fn resolved_vision_provider_from_env() -> String {
    non_empty_env("EDGEQUAKE_VISION_PROVIDER")
        .or_else(|| non_empty_env("EDGEQUAKE_VISION_LLM_PROVIDER"))
        .or_else(|| non_empty_env("EDGEQUAKE_DEFAULT_LLM_PROVIDER"))
        .or_else(|| non_empty_env("EDGEQUAKE_LLM_PROVIDER"))
        .unwrap_or_else(|| "ollama".to_string())
}

/// Return a vision model compatible with the resolved provider.
pub fn default_vision_model_for_provider(provider: &str) -> String {
    use crate::safety_limits::is_model_provider_mismatch;

    let candidates = [
        non_empty_env("EDGEQUAKE_VISION_MODEL"),
        non_empty_env("EDGEQUAKE_VISION_LLM_MODEL"),
        non_empty_env("EDGEQUAKE_DEFAULT_LLM_MODEL"),
        non_empty_env("EDGEQUAKE_LLM_MODEL"),
    ];

    for candidate in candidates.into_iter().flatten() {
        if !is_model_provider_mismatch(provider, &candidate) {
            return candidate;
        }
        tracing::warn!(
            provider,
            model = %candidate,
            "Skipping incompatible vision model from env — model '{}' cannot run on provider '{}'",
            candidate,
            provider,
        );
    }

    match provider {
        "openai" => "gpt-4.1-nano".to_string(),
        "anthropic" => "claude-sonnet-4-20250514".to_string(),
        // SPEC-047 locked Acc chain defaults vision to Small. Stronger W1 ablation
        // (025): set EDGEQUAKE_VISION_MODEL=mistral-medium-3-5 or pin workspace/upload.
        "mistral" => "mistral-small-latest".to_string(),
        _ => "gemma4:latest".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_openai_vision_model_is_compatible() {
        let model = default_vision_model_for_provider("openai");
        assert!(!model.contains("gemma"));
    }

    #[test]
    fn default_mistral_vision_stays_small_for_acc_chain() {
        // Clear env so we hit the compile-time default, not a leftover from other tests.
        let prev = std::env::var("EDGEQUAKE_VISION_MODEL").ok();
        std::env::remove_var("EDGEQUAKE_VISION_MODEL");
        std::env::remove_var("EDGEQUAKE_VISION_LLM_MODEL");
        std::env::remove_var("EDGEQUAKE_DEFAULT_LLM_MODEL");
        std::env::remove_var("EDGEQUAKE_LLM_MODEL");
        let model = default_vision_model_for_provider("mistral");
        assert_eq!(model, "mistral-small-latest");
        if let Some(v) = prev {
            std::env::set_var("EDGEQUAKE_VISION_MODEL", v);
        }
    }

    #[test]
    fn models_toml_lists_mistral_medium_3_5_with_vision() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models.toml");
        let text = std::fs::read_to_string(&root).expect("models.toml readable");
        assert!(
            text.contains("name = \"mistral-medium-3-5\""),
            "025 requires official Medium 3.5 id in catalog"
        );
        // After the medium-3-5 block, supports_vision must be true (coarse but fail-closed).
        let idx = text
            .find("name = \"mistral-medium-3-5\"")
            .expect("medium-3-5 entry");
        let window = &text[idx..idx.saturating_add(600).min(text.len())];
        assert!(
            window.contains("supports_vision = true"),
            "mistral-medium-3-5 must declare supports_vision=true"
        );
    }
}
