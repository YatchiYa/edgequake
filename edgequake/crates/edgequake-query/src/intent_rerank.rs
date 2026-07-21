//! Intent-gated reranker switch (027) + Fact BM25 protect (035) + C1a CE-skip.
//!
//! Acc ledger: CE under P2b holds Fact evidence_recall at ~0.85 while BM25
//! holds ~0.95. Cross-encoder helps ctx_rel on Complex/Summarize. Route
//! **Factual** queries to BM25 when `EDGEQUAKE_FACT_RERANKER=bm25`.
//!
//! 035: Prefer Acc-safe Fact path — BM25-reorder Mix **before** CE+protect
//! (`EDGEQUAKE_FACT_PROTECT_BM25=1`) so lexical Fact gold stays in the CE set
//! without skipping CE (027 Acc tax) or dual-list (034 Acc tax).
//!
//! 057/058 C1a **product latency**: `EDGEQUAKE_FACT_CE_SKIP=1` aliases Fact→BM25
//! (skip CE HTTP ~1s). Acc Fact peer keeps `FACT_PROTECT_BM25` + CE labeled.

use crate::keywords::QueryIntent;

fn env_flag_on(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// `EDGEQUAKE_FACT_RERANKER=bm25`, `EDGEQUAKE_INTENT_RERANK=1`, or
/// `EDGEQUAKE_FACT_CE_SKIP=1` (058 product latency alias) enables Fact→BM25.
pub fn fact_bm25_rerank_enabled() -> bool {
    if env_flag_on("EDGEQUAKE_INTENT_RERANK") || env_flag_on("EDGEQUAKE_FACT_CE_SKIP") {
        return true;
    }
    std::env::var("EDGEQUAKE_FACT_RERANKER")
        .map(|v| v.trim().eq_ignore_ascii_case("bm25"))
        .unwrap_or(false)
}

/// Whether this intent should use the BM25 fact path (skip CE).
pub fn use_bm25_for_intent(intent: QueryIntent) -> bool {
    fact_bm25_rerank_enabled() && matches!(intent, QueryIntent::Factual)
}

/// `EDGEQUAKE_FACT_PROTECT_BM25=1` — BM25-reorder Mix before CE for Factual only.
pub fn fact_protect_bm25_enabled() -> bool {
    env_flag_on("EDGEQUAKE_FACT_PROTECT_BM25")
}

/// BM25 first-stage for protect (CE still ranks). Mutually exclusive with Fact→BM25 skip-CE.
pub fn use_bm25_protect_for_intent(intent: QueryIntent) -> bool {
    fact_protect_bm25_enabled()
        && !fact_bm25_rerank_enabled()
        && matches!(intent, QueryIntent::Factual)
}

/// `EDGEQUAKE_COVERAGE_PROTECT_FIRST` — Exploratory protect slots (0/unset = use global).
///
/// Summarize/coverage intents need Mix set membership; CE precision demotes tails
/// outside `RERANK_PROTECT_FIRST=12`. When set (>0), Exploratory uses this protect
/// count (clamped to top_k) so CE may reorder but not shrink Mix[:top_k].
pub fn coverage_protect_first_from_env() -> Option<usize> {
    std::env::var("EDGEQUAKE_COVERAGE_PROTECT_FIRST")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .map(|n| n.clamp(1, 100))
}

/// Protect slots for this intent: Exploratory coverage override, else global protect.
pub fn protect_first_for_intent(intent: QueryIntent) -> usize {
    if matches!(intent, QueryIntent::Exploratory) {
        if let Some(n) = coverage_protect_first_from_env() {
            return n;
        }
    }
    crate::rerank_protect::protect_first_from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factual_routes_when_enabled() {
        std::env::remove_var("EDGEQUAKE_INTENT_RERANK");
        std::env::remove_var("EDGEQUAKE_FACT_CE_SKIP");
        std::env::set_var("EDGEQUAKE_FACT_RERANKER", "bm25");
        assert!(use_bm25_for_intent(QueryIntent::Factual));
        assert!(!use_bm25_for_intent(QueryIntent::Comparative));
        std::env::remove_var("EDGEQUAKE_FACT_RERANKER");
    }

    #[test]
    fn fact_ce_skip_alias_enables_bm25() {
        std::env::remove_var("EDGEQUAKE_FACT_RERANKER");
        std::env::remove_var("EDGEQUAKE_INTENT_RERANK");
        std::env::set_var("EDGEQUAKE_FACT_CE_SKIP", "1");
        assert!(use_bm25_for_intent(QueryIntent::Factual));
        assert!(!use_bm25_for_intent(QueryIntent::Exploratory));
        std::env::remove_var("EDGEQUAKE_FACT_CE_SKIP");
    }

    #[test]
    fn protect_bm25_only_when_factual_and_not_skip_ce() {
        std::env::remove_var("EDGEQUAKE_FACT_RERANKER");
        std::env::remove_var("EDGEQUAKE_INTENT_RERANK");
        std::env::set_var("EDGEQUAKE_FACT_PROTECT_BM25", "1");
        assert!(use_bm25_protect_for_intent(QueryIntent::Factual));
        assert!(!use_bm25_protect_for_intent(QueryIntent::Exploratory));
        std::env::set_var("EDGEQUAKE_FACT_RERANKER", "bm25");
        assert!(!use_bm25_protect_for_intent(QueryIntent::Factual));
        std::env::remove_var("EDGEQUAKE_FACT_RERANKER");
        std::env::remove_var("EDGEQUAKE_FACT_PROTECT_BM25");
    }

    #[test]
    fn coverage_protect_overrides_exploratory_only() {
        std::env::set_var("EDGEQUAKE_RERANK_PROTECT_FIRST", "12");
        std::env::set_var("EDGEQUAKE_COVERAGE_PROTECT_FIRST", "30");
        assert_eq!(protect_first_for_intent(QueryIntent::Exploratory), 30);
        assert_eq!(protect_first_for_intent(QueryIntent::Factual), 12);
        std::env::set_var("EDGEQUAKE_COVERAGE_PROTECT_FIRST", "0");
        assert_eq!(protect_first_for_intent(QueryIntent::Exploratory), 12);
        std::env::remove_var("EDGEQUAKE_COVERAGE_PROTECT_FIRST");
        std::env::remove_var("EDGEQUAKE_RERANK_PROTECT_FIRST");
    }
}
