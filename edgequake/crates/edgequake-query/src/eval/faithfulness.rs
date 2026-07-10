//! Online faithfulness sampler (SPEC-046 OPS-P2.20).
//!
//! Heuristic only (no LLM judge) — token overlap between answer and retrieved
//! chunk content. Deterministic and non-flaky for CI.
//!
//! Sampling rate is parsed via [`parse_faithfulness_sample_rate`] (pure) so
//! tests never mutate process env.

use crate::context::QueryContext;
use crate::eval::metrics::keyword_recall_in_text;

/// Parse sample rate in `[0.0, 1.0]` from a raw string (default 0 = off).
pub fn parse_faithfulness_sample_rate(raw: &str) -> f64 {
    raw.trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

/// Read `EDGEQUAKE_FAITHFULNESS_SAMPLE_RATE` (default off).
pub fn faithfulness_sample_rate_from_env() -> f64 {
    parse_faithfulness_sample_rate(
        &std::env::var("EDGEQUAKE_FAITHFULNESS_SAMPLE_RATE").unwrap_or_default(),
    )
}

/// Deterministic sample decision from request id / query hash (no RNG flakiness).
pub fn should_sample_faithfulness(sample_rate: f64, sample_key: &str) -> bool {
    if sample_rate <= 0.0 {
        return false;
    }
    if sample_rate >= 1.0 {
        return true;
    }
    let bucket = stable_bucket(sample_key);
    (bucket as f64) < sample_rate * 10_000.0
}

fn stable_bucket(key: &str) -> u32 {
    // FNV-1a 32-bit — stable across platforms, no crypto dep.
    let mut hash: u32 = 0x811c_9dc5;
    for b in key.as_bytes() {
        hash ^= u32::from(*b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash % 10_000
}

/// Extract significant tokens (≥4 chars, alphanumeric) from text.
pub fn significant_tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 4)
        .map(|t| t.to_lowercase())
        .collect()
}

/// Heuristic faithfulness: fraction of answer tokens found in retrieved chunks.
///
/// Returns `1.0` when answer is empty or has no significant tokens (nothing to
/// ground). Returns `0.0` when context is empty but answer has tokens.
pub fn score_faithfulness_heuristic(answer: &str, context: &QueryContext) -> f32 {
    let tokens = significant_tokens(answer);
    if tokens.is_empty() {
        return 1.0;
    }
    let mut corpus = String::new();
    for chunk in &context.chunks {
        corpus.push_str(&chunk.content);
        corpus.push(' ');
    }
    for entity in &context.entities {
        corpus.push_str(&entity.name);
        corpus.push(' ');
        corpus.push_str(&entity.description);
        corpus.push(' ');
    }
    if corpus.trim().is_empty() {
        return 0.0;
    }
    keyword_recall_in_text(&corpus, &tokens)
}

/// Maybe score faithfulness for a completed query (pure decision + score).
pub fn maybe_score_faithfulness(
    sample_rate: f64,
    sample_key: &str,
    answer: &str,
    context: &QueryContext,
) -> Option<f32> {
    if !should_sample_faithfulness(sample_rate, sample_key) {
        return None;
    }
    Some(score_faithfulness_heuristic(answer, context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{QueryContext, RetrievedChunk};

    #[test]
    fn parse_rate_edge_cases() {
        assert_eq!(parse_faithfulness_sample_rate(""), 0.0);
        assert_eq!(parse_faithfulness_sample_rate("0.5"), 0.5);
        assert_eq!(parse_faithfulness_sample_rate("2"), 1.0);
        assert_eq!(parse_faithfulness_sample_rate("-1"), 0.0);
        assert_eq!(parse_faithfulness_sample_rate("nope"), 0.0);
    }

    #[test]
    fn sample_decision_is_deterministic() {
        let a = should_sample_faithfulness(0.5, "req-abc");
        let b = should_sample_faithfulness(0.5, "req-abc");
        assert_eq!(a, b);
        assert!(!should_sample_faithfulness(0.0, "req-abc"));
        assert!(should_sample_faithfulness(1.0, "req-abc"));
    }

    #[test]
    fn heuristic_grounds_answer_in_chunk() {
        let mut ctx = QueryContext::new();
        ctx.add_chunk(RetrievedChunk::new(
            "c1",
            "Sarah Chen leads EdgeQuake research",
            1.0,
        ));
        let score = score_faithfulness_heuristic("Sarah Chen leads EdgeQuake", &ctx);
        assert!(score > 0.5, "expected grounded score, got {score}");
    }

    #[test]
    fn heuristic_zero_when_context_empty() {
        let ctx = QueryContext::new();
        let score = score_faithfulness_heuristic("Sarah Chen invented widgets", &ctx);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn maybe_score_respects_rate_off() {
        let ctx = QueryContext::new();
        assert!(maybe_score_faithfulness(0.0, "k", "answer tokens here", &ctx).is_none());
    }
}
