//! Shared LLM completion options for extraction extractors (SPEC-017 DRY-008).
//!
//! SPEC-047: Mistral Large rejects `reasoning_effort` (API code 3051). Only send it
//! for models that document support (OpenAI reasoning family + Mistral Small / Medium 3.5).

use edgequake_llm::traits::CompletionOptions;

use super::temperature::effective_temperature_for_model;
use super::types::ExtractionResult;

/// Whether this model accepts a `reasoning_effort` request field.
///
/// - OpenAI gpt-5 / o-series: yes (use `"none"` for structured extraction)
/// - Mistral Small + Medium 3.5: yes ([docs](https://docs.mistral.ai/capabilities/reasoning))
/// - Mistral Large / Magistra / Codestral / others: **no** — sending it returns HTTP 400
pub fn model_accepts_reasoning_effort(model: &str) -> bool {
    let m = model.to_ascii_lowercase();
    if m.contains("gpt-5")
        || m.starts_with("o1")
        || m.starts_with("o3")
        || m.starts_with("o4")
        || m.contains("o1-")
        || m.contains("o3-")
        || m.contains("o4-")
    {
        return true;
    }
    // Mistral adjustable-reasoning models only
    if m.contains("mistral-small") {
        return true;
    }
    if m.contains("mistral-medium-3") || m.contains("mistral-medium-250") {
        return true;
    }
    // Explicit medium-3-5 alias used by health/provider resolution
    if m == "mistral-medium-3-5" || m.contains("medium-3-5") {
        return true;
    }
    false
}

/// Build extraction [`CompletionOptions`] with reasoning disabled when supported.
///
/// WHY reasoning_effort="none" (when accepted): Reasoning models exhaust completion
/// budget on CoT, producing empty structured output.
/// WHY omit for mistral-large-*: API returns 400 `reasoning_effort is not enabled`.
pub fn extraction_completion_options(model: &str, max_tokens: usize) -> CompletionOptions {
    CompletionOptions {
        max_tokens: Some(max_tokens),
        temperature: effective_temperature_for_model(model, 0.0),
        reasoning_effort: if model_accepts_reasoning_effort(model) {
            Some("none".to_string())
        } else {
            None
        },
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
        let opts = extraction_completion_options("mistral-large-latest", 1024);
        assert!(opts.reasoning_effort.is_none());
    }

    #[test]
    fn small_sets_reasoning_none() {
        let opts = extraction_completion_options("mistral-small-latest", 1024);
        assert_eq!(opts.reasoning_effort.as_deref(), Some("none"));
    }

    #[test]
    fn gpt5_sets_reasoning_none() {
        let opts = extraction_completion_options("gpt-5-nano", 1024);
        assert_eq!(opts.reasoning_effort.as_deref(), Some("none"));
    }
}
