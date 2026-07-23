//! D-53 / D-34 — TokenEstimator SSOT for pipeline gates and chunk sizing.
//!
//! Prefer tiktoken `cl100k_base` (embedding / GPT-4 family). Falls back to
//! chars/4 only if the tokenizer fails to initialize.

/// Shared token estimator used by chunker, merger description gates, summarizer.
pub trait TokenEstimator: Send + Sync {
    fn count(&self, text: &str) -> usize;
}

/// Default production estimator (tiktoken cl100k when available).
#[derive(Debug, Default)]
pub struct DefaultTokenEstimator;

impl TokenEstimator for DefaultTokenEstimator {
    fn count(&self, text: &str) -> usize {
        count_tokens(text)
    }
}

/// Count tokens with SSOT estimator (tiktoken cl100k → chars/4 fallback).
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // tiktoken-rs 0.6 returns Arc<parking_lot::Mutex<CoreBPE>>; lock never poisons.
    let bpe = tiktoken_rs::cl100k_base_singleton();
    let guard = bpe.lock();
    let tokens = guard.encode_with_special_tokens(text);
    tokens.len().max(1)
}

/// Legacy chars/4 heuristic (tests / fallback only).
pub fn heuristic_token_count(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.len().div_ceil(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_token_estimator_ssot_non_empty() {
        let n = count_tokens("hello world from edgequake");
        assert!(n >= 3, "expected real tokenizer count, got {n}");
    }

    #[test]
    fn unit_token_estimator_empty() {
        assert_eq!(count_tokens(""), 0);
    }
}
