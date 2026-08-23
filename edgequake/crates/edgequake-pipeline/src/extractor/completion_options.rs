//! Shared LLM completion options for extraction extractors (SPEC-017 DRY-008).
//!
//! SPEC-109: reasoning effort is resolved + clamped via `edgequake_llm::reasoning_capabilities`
//! (capability SSOT). Callers may pass a desired effort; when unset, structured extract uses
//! the lowest supported effort for the model (`none` or `minimal`, never an illegal value).
//!
//! SPEC-113 / extract think-off: for Ollama / LM Studio, Auto (`reasoning_effort` unset) maps
//! to wire `think: true` on thinking-capable models. Structured KG extract must therefore
//! floor local providers to `"none"` so the client sends `think: false`.
//!
//! Cloud `none` is a different verb: it **disables** reasoning on OpenAI/OpenRouter. Catalog
//! listing `none` is not live truth — extract lifts to the lowest *enabled* effort so
//! mandatory-reasoning endpoints do not 400.

use edgequake_llm::traits::CompletionOptions;
use edgequake_llm::{clamp_reasoning_effort, lowest_for_structured_output};

use edgequake_llm::resolve_effective_temperature;

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
    let resolved = match desired.map(str::trim).filter(|s| !s.is_empty()) {
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
    };
    coerce_extract_disable_to_enabled(provider, model, resolved)
}

/// `none` means **disable reasoning** on OpenAI / OpenRouter / Azure chat wire.
///
/// Extract's intent is cheapest *legal* structured output. Local Ollama `none` is
/// `think:false` (SPEC-113). Cloud `none` is an explicit disable; endpoints that
/// mandate reasoning return HTTP 400
/// ("Reasoning is mandatory for this endpoint and cannot be disabled").
/// Catalog listing `none` is not ground truth — lift to the lowest enabled rung.
fn coerce_extract_disable_to_enabled(
    provider: &str,
    model: &str,
    effort: Option<String>,
) -> Option<String> {
    let Some(raw) = effort.as_deref() else {
        return effort;
    };
    if !raw.eq_ignore_ascii_case("none") {
        return effort;
    }
    if is_local_extraction_provider(provider) {
        return effort;
    }
    let p = provider.to_ascii_lowercase();
    if p.contains("mistral") {
        return effort;
    }
    match lowest_enabled_reasoning_effort(provider, model) {
        Some(lifted) => {
            tracing::info!(
                provider,
                model,
                from = "none",
                to = %lifted,
                "extract none would disable reasoning; using lowest enabled effort"
            );
            Some(lifted)
        }
        None => effort,
    }
}

/// Lowest effort the model accepts that is **not** disable (`none`).
pub fn lowest_enabled_reasoning_effort(provider: &str, model: &str) -> Option<String> {
    for candidate in ["minimal", "low", "medium", "high"] {
        if clamp_reasoning_effort(provider, model, Some(candidate)).as_deref() == Some(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Upstream rejected an explicit reasoning-off encoding.
pub fn reasoning_disable_rejected(err: &str) -> bool {
    let e = err.to_ascii_lowercase();
    e.contains("reasoning is mandatory")
        || (e.contains("cannot be disabled") && e.contains("reasoning"))
}

/// Next enabled effort after a disable-rejected 400 (`none`/`minimal` → `low` …).
pub fn lift_extract_effort_after_disable_reject(
    provider: &str,
    model: &str,
    current: Option<&str>,
) -> Option<String> {
    let ladder = ["minimal", "low", "medium", "high"];
    let cur = current.unwrap_or("none").trim().to_ascii_lowercase();
    let start = if cur == "none" {
        0
    } else {
        ladder
            .iter()
            .position(|s| *s == cur)
            .map(|i| i + 1)
            .unwrap_or(0)
    };
    for candidate in ladder.iter().skip(start) {
        if clamp_reasoning_effort(provider, model, Some(candidate)).as_deref() == Some(*candidate) {
            return Some((*candidate).to_string());
        }
    }
    None
}

/// If the LLM error is a disable-rejected 400, return the next legal extract effort.
pub fn maybe_lift_extract_effort_from_llm_error(
    provider: &str,
    model: &str,
    current_wire_effort: Option<&str>,
    err: &str,
) -> Option<String> {
    if !reasoning_disable_rejected(err) {
        return None;
    }
    lift_extract_effort_after_disable_reject(provider, model, current_wire_effort)
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
    // Extract must always send a legal enabled effort when the model supports
    // reasoning — omit-env (SPEC-131) is for non-reasoning models only. Mandatory-
    // reasoning endpoints 400 when the field is absent or explicitly `none`.
    let reasoning_effort = resolve_extraction_reasoning_effort(provider, model, desired);
    CompletionOptions {
        max_tokens: Some(max_tokens),
        temperature: resolve_effective_temperature(model, 0.0),
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
    fn gpt5_nano_desired_none_clamps_to_minimal() {
        let opts =
            extraction_completion_options_with_effort("gpt-5-nano", 1024, Some("none"), "openai");
        assert_eq!(opts.reasoning_effort.as_deref(), Some("minimal"));
    }

    #[test]
    fn gpt54_nano_floor_lifts_off_disable() {
        let opts = extraction_completion_options("gpt-5.4-nano", 1024);
        assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));
    }

    #[test]
    fn gpt54_nano_desired_none_lifts_to_low() {
        let opts =
            extraction_completion_options_with_effort("gpt-5.4-nano", 1024, Some("none"), "openai");
        assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));
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
    fn gpt55_extract_none_lifts_to_low() {
        // gpt-5.5 catalog lists `none` but live endpoints reject disable.
        let opts =
            extraction_completion_options_with_effort("gpt-5.5", 1024, Some("none"), "openai");
        assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));
    }

    #[test]
    fn gpt54_nano_extract_never_sends_disable() {
        let opts =
            extraction_completion_options_with_effort("gpt-5.4-nano", 1024, Some("none"), "openai");
        assert_ne!(opts.reasoning_effort.as_deref(), Some("none"));
        assert_eq!(opts.reasoning_effort.as_deref(), Some("low"));
    }

    #[test]
    fn openrouter_extract_none_lifts_off_disable() {
        let opts = extraction_completion_options_with_effort(
            "anthropic/claude-sonnet-4",
            1024,
            Some("none"),
            "openrouter",
        );
        assert_ne!(opts.reasoning_effort.as_deref(), Some("none"));
        assert!(opts.reasoning_effort.is_some());
    }

    #[test]
    fn reasoning_disable_rejected_matches_openrouter_wording() {
        assert!(reasoning_disable_rejected(
            "Invalid request: Reasoning is mandatory for this endpoint and cannot be disabled."
        ));
        assert!(!reasoning_disable_rejected("rate limit exceeded"));
    }

    #[test]
    fn lift_after_disable_reject_skips_unsupported_minimal() {
        // gpt-5.4-nano has none/low/… — no `minimal`. Next enabled is `low`.
        assert_eq!(
            lift_extract_effort_after_disable_reject("openai", "gpt-5.4-nano", Some("none"))
                .as_deref(),
            Some("low")
        );
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
