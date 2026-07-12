//! SPEC-047 P7a — LightRAG-aligned description merge policy (SSOT).
//!
//! # Law (LightRAG `operate.py::_handle_entity_relation_summary`)
//!
//! 1. One fragment → return as-is (no LLM).
//! 2. `len(fragments) < FORCE_LLM_SUMMARY_ON_MERGE` and under token budget
//!    → join with [`GRAPH_FIELD_SEP`] (no LLM).
//! 3. Otherwise → LLM summarize once.
//!
//! EdgeQuake keeps SPEC-032 W-06 Jaccard as a **secondary** soft-resume gate:
//! near-identical `(existing, incoming)` skips LLM and keeps the richer text.
//!
//! # SOLID
//!
//! - **S**: This module only decides *whether* to LLM and how to join.
//! - **O**: Thresholds via [`DescriptionMergePolicy`] / env.
//! - **D**: Callers inject LLM only when [`DescriptionMergeDecision::NeedsLlm`].

use std::collections::HashSet;

use super::entity::description_similarity;

/// LightRAG `GRAPH_FIELD_SEP` (local code-is-law: `<SEP>`).
///
/// Used to join description fragments when LLM is skipped. Must stay stable
/// after data is written so soft-resume can split existing descriptions.
pub const GRAPH_FIELD_SEP: &str = "<SEP>";

/// LightRAG `DEFAULT_FORCE_LLM_SUMMARY_ON_MERGE`.
pub const DEFAULT_FORCE_LLM_SUMMARY_ON_MERGE: usize = 8;

/// LightRAG `DEFAULT_SUMMARY_MAX_TOKENS` — approx token budget for join-without-LLM.
pub const DEFAULT_SUMMARY_MAX_TOKENS: usize = 1200;

/// Env: `EDGEQUAKE_FORCE_LLM_SUMMARY_ON_MERGE` (also accepts `FORCE_LLM_SUMMARY_ON_MERGE`).
pub fn force_llm_summary_on_merge_from_env() -> usize {
    std::env::var("EDGEQUAKE_FORCE_LLM_SUMMARY_ON_MERGE")
        .or_else(|_| std::env::var("FORCE_LLM_SUMMARY_ON_MERGE"))
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or(DEFAULT_FORCE_LLM_SUMMARY_ON_MERGE)
}

/// Env: `EDGEQUAKE_SUMMARY_MAX_TOKENS` (also accepts `SUMMARY_MAX_TOKENS`).
pub fn summary_max_tokens_from_env() -> usize {
    std::env::var("EDGEQUAKE_SUMMARY_MAX_TOKENS")
        .or_else(|_| std::env::var("SUMMARY_MAX_TOKENS"))
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 64)
        .unwrap_or(DEFAULT_SUMMARY_MAX_TOKENS)
}

/// Tunables for description merge decisions.
#[derive(Debug, Clone)]
pub struct DescriptionMergePolicy {
    /// When true, LLM may run if fragment/token gates say so.
    pub use_llm: bool,
    /// Fragment count at which LLM is forced (LightRAG default 8).
    pub force_llm_summary_on_merge: usize,
    /// Approx token budget under which join-without-LLM is allowed.
    pub summary_max_tokens: usize,
    /// Jaccard gate (SPEC-032 W-06): skip LLM when near-identical.
    pub similarity_threshold: f32,
    /// Hard cap on stored description length.
    pub max_description_length: usize,
}

impl DescriptionMergePolicy {
    /// Build from merger-facing knobs (DRY with [`super::MergerConfig`]).
    pub fn from_parts(
        use_llm: bool,
        force_llm_summary_on_merge: usize,
        summary_max_tokens: usize,
        similarity_threshold: f32,
        max_description_length: usize,
    ) -> Self {
        Self {
            use_llm,
            force_llm_summary_on_merge: force_llm_summary_on_merge.max(1),
            summary_max_tokens: summary_max_tokens.max(64),
            similarity_threshold,
            max_description_length,
        }
    }
}

/// Outcome of the pure merge policy (no I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionMergeDecision {
    /// Final description — do **not** call the LLM.
    Resolved(String),
    /// Call LLM once with these unique fragments.
    NeedsLlm { fragments: Vec<String> },
}

/// Rough token estimate (LightRAG uses a real tokenizer; we use chars/4).
pub fn approx_token_count(text: &str) -> usize {
    text.len().div_ceil(4)
}

/// Split a stored description into fragments on [`GRAPH_FIELD_SEP`].
pub fn split_description_fragments(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if !trimmed.contains(GRAPH_FIELD_SEP) {
        return vec![trimmed.to_string()];
    }
    trimmed
        .split(GRAPH_FIELD_SEP)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Deduped fragment list from existing graph text + incoming extract.
pub fn collect_unique_fragments(existing: &str, incoming: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for frag in split_description_fragments(existing)
        .into_iter()
        .chain(split_description_fragments(incoming))
    {
        let key = frag.to_lowercase();
        if seen.insert(key) {
            out.push(frag);
        }
    }
    out
}

/// Join fragments with [`GRAPH_FIELD_SEP`], truncated to `max_len`.
pub fn join_description_fragments(fragments: &[String], max_len: usize) -> String {
    if fragments.is_empty() {
        return String::new();
    }
    let joined = fragments.join(GRAPH_FIELD_SEP);
    truncate_at_boundary(&joined, max_len)
}

fn truncate_at_boundary(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    let mut end = max_length;
    for (i, c) in text.char_indices().take(max_length) {
        if c == '.' || c == '!' || c == '?' {
            end = i + 1;
        }
    }
    // Prefer not to cut mid-separator if possible.
    if let Some(sep_at) = text[..end].rfind(GRAPH_FIELD_SEP) {
        if sep_at > max_length / 4 {
            return text[..sep_at].to_string();
        }
    }
    text[..end].to_string()
}

fn keep_longer(a: &str, b: &str) -> String {
    if b.len() > a.len() {
        b.to_string()
    } else {
        a.to_string()
    }
}

/// Decide how to merge `existing` graph description with `incoming` extract.
///
/// Pure / side-effect free — callers own the LLM call.
pub fn decide_description_merge(
    existing: &str,
    incoming: &str,
    policy: &DescriptionMergePolicy,
) -> DescriptionMergeDecision {
    let existing = existing.trim();
    let incoming = incoming.trim();

    if existing.is_empty() && incoming.is_empty() {
        return DescriptionMergeDecision::Resolved(String::new());
    }
    if existing.is_empty() {
        return DescriptionMergeDecision::Resolved(truncate_at_boundary(
            incoming,
            policy.max_description_length,
        ));
    }
    if incoming.is_empty() {
        return DescriptionMergeDecision::Resolved(existing.to_string());
    }

    // SPEC-032 W-06: near-identical update → keep richer, no LLM.
    let similarity = description_similarity(existing, incoming);
    if similarity >= policy.similarity_threshold {
        return DescriptionMergeDecision::Resolved(keep_longer(existing, incoming));
    }

    let fragments = collect_unique_fragments(existing, incoming);
    if fragments.is_empty() {
        return DescriptionMergeDecision::Resolved(String::new());
    }
    if fragments.len() == 1 {
        return DescriptionMergeDecision::Resolved(truncate_at_boundary(
            &fragments[0],
            policy.max_description_length,
        ));
    }

    let total_tokens: usize = fragments.iter().map(|f| approx_token_count(f)).sum();
    let under_fragment_gate = fragments.len() < policy.force_llm_summary_on_merge;
    let under_token_gate = total_tokens < policy.summary_max_tokens;

    if !policy.use_llm || (under_fragment_gate && under_token_gate) {
        return DescriptionMergeDecision::Resolved(join_description_fragments(
            &fragments,
            policy.max_description_length,
        ));
    }

    DescriptionMergeDecision::NeedsLlm { fragments }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(force: usize, use_llm: bool) -> DescriptionMergePolicy {
        DescriptionMergePolicy::from_parts(use_llm, force, 1200, 0.85, 4096)
    }

    #[test]
    fn single_fragment_never_needs_llm() {
        let d = decide_description_merge("", "Alice is a researcher", &policy(8, true));
        assert_eq!(
            d,
            DescriptionMergeDecision::Resolved("Alice is a researcher".into())
        );
    }

    #[test]
    fn two_distinct_fragments_join_below_threshold() {
        let d = decide_description_merge(
            "Alice works at Acme",
            "Alice published a paper on RAG",
            &policy(8, true),
        );
        match d {
            DescriptionMergeDecision::Resolved(s) => {
                assert!(s.contains(GRAPH_FIELD_SEP));
                assert!(s.contains("Acme"));
                assert!(s.contains("RAG"));
            }
            DescriptionMergeDecision::NeedsLlm { .. } => panic!("expected join, not LLM"),
        }
    }

    #[test]
    fn eight_fragments_force_llm() {
        let existing = (0..7)
            .map(|i| format!("fact {i} about entity"))
            .collect::<Vec<_>>()
            .join(GRAPH_FIELD_SEP);
        let d = decide_description_merge(&existing, "fact 7 about entity", &policy(8, true));
        match d {
            DescriptionMergeDecision::NeedsLlm { fragments } => {
                assert_eq!(fragments.len(), 8);
            }
            DescriptionMergeDecision::Resolved(s) => panic!("expected LLM, got resolved: {s}"),
        }
    }

    #[test]
    fn seven_fragments_still_join() {
        let existing = (0..6)
            .map(|i| format!("fact {i} about entity"))
            .collect::<Vec<_>>()
            .join(GRAPH_FIELD_SEP);
        let d = decide_description_merge(&existing, "fact 6 about entity", &policy(8, true));
        assert!(matches!(d, DescriptionMergeDecision::Resolved(_)));
    }

    #[test]
    fn jaccard_gate_skips_llm_even_with_many_fragments() {
        // Existing already has 8 joined facts; incoming is near-copy of whole blob.
        let existing = (0..8)
            .map(|i| format!("fact {i} about entity xyz"))
            .collect::<Vec<_>>()
            .join(GRAPH_FIELD_SEP);
        let incoming = existing.clone();
        let d = decide_description_merge(&existing, &incoming, &policy(8, true));
        assert!(matches!(d, DescriptionMergeDecision::Resolved(_)));
    }

    #[test]
    fn use_llm_false_always_joins() {
        let existing = (0..10)
            .map(|i| format!("fact {i}"))
            .collect::<Vec<_>>()
            .join(GRAPH_FIELD_SEP);
        let d = decide_description_merge(&existing, "fact extra", &policy(8, false));
        assert!(matches!(d, DescriptionMergeDecision::Resolved(_)));
    }

    #[test]
    fn split_roundtrip() {
        let joined = join_description_fragments(&["a".into(), "b".into(), "c".into()], 4096);
        assert_eq!(split_description_fragments(&joined), vec!["a", "b", "c"]);
    }
}
