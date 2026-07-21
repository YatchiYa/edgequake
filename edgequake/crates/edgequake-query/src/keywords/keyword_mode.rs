//! Product keyword extraction mode (059/060 Horizon C).
//!
//! LightRAG's KEYWORD role uses a small non-thinking model for query latency.
//! Under Acc Mistral pins EQ paid ~1.8s keyword LLM p50 ([059](../../../../../specs/001-benchmark/001-edgquake-improvements/059-c1b-latency-ceiling-keyword-embed.md)).
//!
//! `EDGEQUAKE_KEYWORD_MODE=heuristic` skips the keyword LLM and uses the same
//! rule-based + intent heuristic path already used for MockProvider — product
//! latency peer only; Acc Fact peer keeps default `llm`.

use super::extractor::{rule_based_keyword_extraction, ExtractedKeywords};
use super::intent::QueryIntent;

/// How query keywords are produced before Mix retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordMode {
    /// LLM JSON extract (Acc default / LightRAG KEYWORD role with a model).
    Llm,
    /// Deterministic stopword/casing rules + heuristic intent (no LLM RTT).
    Heuristic,
}

/// Keyword LLM result cache (064). Default **on**; set `EDGEQUAKE_KEYWORD_CACHE=0` to disable.
pub fn keyword_cache_enabled() -> bool {
    !matches!(
        std::env::var("EDGEQUAKE_KEYWORD_CACHE")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("0") | Some("false") | Some("off") | Some("no")
    )
}

/// `EDGEQUAKE_KEYWORD_MODE=llm|heuristic` (default `llm`).
pub fn keyword_mode_from_env() -> KeywordMode {
    match std::env::var("EDGEQUAKE_KEYWORD_MODE")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "heuristic" | "rules" | "rule" | "fast" => KeywordMode::Heuristic,
        _ => KeywordMode::Llm,
    }
}

/// Rule-based keywords + heuristic intent (MockProvider parity).
pub fn heuristic_extracted_keywords(query: &str) -> ExtractedKeywords {
    let keywords = rule_based_keyword_extraction(query);
    ExtractedKeywords::new(
        keywords.high_level,
        keywords.low_level,
        QueryIntent::classify_heuristic(query),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn keyword_mode_env_and_heuristic_extract() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("EDGEQUAKE_KEYWORD_MODE");
        assert_eq!(keyword_mode_from_env(), KeywordMode::Llm);
        std::env::set_var("EDGEQUAKE_KEYWORD_MODE", "heuristic");
        assert_eq!(keyword_mode_from_env(), KeywordMode::Heuristic);
        std::env::remove_var("EDGEQUAKE_KEYWORD_MODE");

        let k = heuristic_extracted_keywords("What is BRCA1 mutation risk?");
        assert!(!k.is_empty());
        assert!(matches!(k.query_intent, QueryIntent::Factual));
    }
}
