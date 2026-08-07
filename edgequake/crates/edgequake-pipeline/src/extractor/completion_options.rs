//! Shared LLM completion options for extraction extractors (SPEC-017 DRY-008).
//!
//! SPEC-109: reasoning effort is resolved + clamped via `edgequake_llm::reasoning_capabilities`
//! (capability SSOT). Callers may pass a desired effort; when unset, structured extract uses
//! the lowest supported effort for the model (`none` or `minimal`, never an illegal value).

use edgequake_llm::traits::CompletionOptions;
use edgequake_llm::{clamp_reasoning_effort, lowest_for_structured_output};

use super::temperature::effective_temperature_for_model;
use super::types::ExtractionResult;

/// Whether this model accepts a `reasoning_effort` request field.
///
/// Thin wrapper over the edgequake-llm capability registry (SPEC-109 LAW-R4).
#[allow(dead_code)] // public API for callers / future UI gates
pub fn model_accepts_reasoning_effort(provider: &str, model: &str) -> bool {
    edgequake_llm::reasoning_capabilities_for(provider, model).is_some()
}

/// Build extraction [`CompletionOptions`] with clamped reasoning effort.
///
/// - `desired`: optional override from workspace/env/request hierarchy
/// - When `desired` is `None`, uses [`lowest_for_structured_output`]
/// - Provider defaults to `"openai"` for model-id matching when unknown
pub fn extraction_completion_options(model: &str, max_tokens: usize) -> CompletionOptions {
    extraction_completion_options_with_effort(model, max_tokens, None, "openai")
}

/// SPEC-109: build extract options with explicit provider + desired effort.
pub fn extraction_completion_options_with_effort(
    model: &str,
    max_tokens: usize,
    desired: Option<&str>,
    provider: &str,
) -> CompletionOptions {
    let reasoning_effort = match desired {
        Some(d) => clamp_reasoning_effort(provider, model, Some(d)),
        None => lowest_for_structured_output(provider, model),
    };
    CompletionOptions {
        max_tokens: Some(max_tokens),
        temperature: effective_temperature_for_model(model, 0.0),
        reasoning_effort,
        ..Default::default()
    }
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
}
