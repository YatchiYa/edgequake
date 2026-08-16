//! Shared LLM completion options for extraction extractors (SPEC-017 DRY-008).
//!
//! SPEC-109: reasoning effort is resolved + clamped via `edgequake_llm::reasoning_capabilities`
//! (capability SSOT). Callers may pass a desired effort; when unset, structured extract uses
//! the lowest supported effort for the model (`none` or `minimal`, never an illegal value).
//!
//! SPEC-113 / extract think-off: for Ollama / LM Studio, Auto (`reasoning_effort` unset) maps
//! to wire `think: true` on thinking-capable models. Structured KG extract must therefore
//! floor local providers to `"none"` so the client sends `think: false`.

use edgequake_llm::traits::CompletionOptions;
use edgequake_llm::{clamp_reasoning_effort, lowest_for_structured_output};

use super::temperature::effective_temperature_for_model;
use super::types::ExtractionResult;
use crate::pipeline::is_local_extraction_provider;

/// Whether this model accepts a `reasoning_effort` request field.
///
/// Thin wrapper over the edgequake-llm capability registry (SPEC-109 LAW-R4).
#[allow(dead_code)] // public API for callers / future UI gates
pub fn model_accepts_reasoning_effort(provider: &str, model: &str) -> bool {
    edgequake_llm::reasoning_capabilities_for(provider, model).is_some()
}

/// Structured-extract floor when the capability registry has no ladder (Ollama static = None).
///
/// Returns `Some("none")` for local providers so extract disables thinking by default.
pub fn structured_extract_effort_floor(provider: &str) -> Option<String> {
    if is_local_extraction_provider(provider) {
        Some("none".to_string())
    } else {
        None
    }
}

/// Resolve extract `reasoning_effort` for CompletionOptions (provider-aware).
pub fn resolve_extraction_reasoning_effort(
    provider: &str,
    model: &str,
    desired: Option<&str>,
) -> Option<String> {
    match desired.map(str::trim).filter(|s| !s.is_empty()) {
        Some(d) => {
            // Registry clamp when present; for Ollama (no static caps) pass through
            // so map_think can emit think:false for "none".
            clamp_reasoning_effort(provider, model, Some(d)).or_else(|| {
                if is_local_extraction_provider(provider) {
                    Some(d.to_ascii_lowercase())
                } else {
                    // Non-reasoning / reject — omit wire field
                    None
                }
            })
        }
        None => lowest_for_structured_output(provider, model)
            .or_else(|| structured_extract_effort_floor(provider)),
    }
}

/// Build extraction [`CompletionOptions`] with clamped reasoning effort.
///
/// Provider defaults to `"openai"` for backward-compatible model-id matching.
/// Prefer [`extraction_completion_options_with_effort`] with the real provider
/// (especially `"ollama"`) so local think-off flooring applies.
pub fn extraction_completion_options(model: &str, max_tokens: usize) -> CompletionOptions {
    extraction_completion_options_with_effort(model, max_tokens, None, "openai")
}

/// SPEC-109 / SPEC-113: build extract options with explicit provider + desired effort.
pub fn extraction_completion_options_with_effort(
    model: &str,
    max_tokens: usize,
    desired: Option<&str>,
    provider: &str,
) -> CompletionOptions {
    let reasoning_effort = resolve_extraction_reasoning_effort(provider, model, desired);
    CompletionOptions {
        max_tokens: Some(max_tokens),
        temperature: effective_temperature_for_model(model, 0.0),
        reasoning_effort,
        ..Default::default()
    }
    .with_provider_prompt_cache("extract", provider, model)
}

/// Adaptive chunk size recommendation based on document size (bytes).
pub fn recommended_chunk_size_for_bytes(chunk_size_bytes: usize) -> usize {
    crate::adaptive_chunking::calculate_adaptive_chunk_size(chunk_size_bytes)
}

/// Copy token usage from an LLM response into an extraction result.
pub fn assign_token_usage(
    result: &mut ExtractionResult,
    input_tokens: usize,
    output_tokens: usize,
) {
    result.input_tokens = input_tokens;
    result.output_tokens = output_tokens;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_omits_reasoning_effort() {
        let opts = extraction_completion_options_with_effort(
            "mistral-large-latest",
            1024,
            Some("high"),
            "mistral",
        );
        assert!(opts.reasoning_effort.is_none());
    }

    #[test]
    fn small_sets_reasoning_floor_none() {
        let opts = extraction_completion_options_with_effort(
            "mistral-small-latest",
            1024,
            None,
            "mistral",
        );
        assert_eq!(opts.reasoning_effort.as_deref(), Some("none"));
    }

    #[test]
    fn gpt5_nano_floor_is_minimal() {
        // gpt-5-nano (non-5.4) does not accept `none`
        let opts = extraction_completion_options("gpt-5-nano", 1024);
        assert_eq!(opts.reasoning_effort.as_deref(), Some("minimal"));
    }

    #[test]
    fn gpt54_nano_floor_is_none() {
        let opts = extraction_completion_options("gpt-5.4-nano", 1024);
        assert_eq!(opts.reasoning_effort.as_deref(), Some("none"));
    }

    #[test]
    fn desired_low_preserved_when_supported() {
        let opts =
            extraction_completion_options_with_effort("gpt-5-mini", 1024, Some("low"), "openai");
        assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));
    }

    #[test]
    fn ollama_qwen_unset_floors_to_none() {
        let opts =
            extraction_completion_options_with_effort("qwen3.6:35b-a3b", 1024, None, "ollama");
        assert_eq!(
            opts.reasoning_effort.as_deref(),
            Some("none"),
            "local extract must floor to none so Ollama sends think:false"
        );
    }

    #[test]
    fn lmstudio_qwen_unset_floors_to_none() {
        let opts = extraction_completion_options_with_effort("qwen3-14b", 1024, None, "lmstudio");
        assert_eq!(
            opts.reasoning_effort.as_deref(),
            Some("none"),
            "LM Studio is a local extract provider and must floor to none"
        );
    }

    #[test]
    fn ollama_qwen_explicit_none_preserved() {
        let opts = extraction_completion_options_with_effort(
            "qwen3.6:35b-a3b",
            1024,
            Some("none"),
            "ollama",
        );
        assert_eq!(opts.reasoning_effort.as_deref(), Some("none"));
    }

    #[test]
    fn ollama_qwen_explicit_high_passthrough() {
        let opts = extraction_completion_options_with_effort(
            "qwen3.6:35b-a3b",
            1024,
            Some("high"),
            "ollama",
        );
        assert_eq!(opts.reasoning_effort.as_deref(), Some("high"));
    }

    #[test]
    fn openai_compat_qwen_without_local_floor_stays_open() {
        // Hardcoded openai provider + qwen model has no registry ladder → no floor.
        // Callers must pass the real provider ("ollama").
        let opts = extraction_completion_options("qwen3.6:35b-a3b", 1024);
        assert!(opts.reasoning_effort.is_none());
    }

    #[test]
    fn extract_options_set_provider_prompt_cache_key_by_default() {
        let opts = extraction_completion_options_with_effort(
            "mistral-small-latest",
            1024,
            None,
            "mistral",
        );
        if edgequake_llm::provider_prompt_cache_enabled() {
            assert_eq!(
                opts.prompt_cache_key.as_deref(),
                Some("eq:extract:mistral:mistral-small-latest")
            );
        } else {
            assert!(opts.prompt_cache_key.is_none());
        }
    }
}
