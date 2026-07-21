//! Query intent classification for adaptive retrieval strategy (SPEC-046).
//!
//! Evidence-aligned routing (GraphRAG-Bench ICLR 2026 / SPEC-046 P0.1):
//! - Factual (L1): Naive — graphs add tax on simple facts
//! - Relational (L2): Hybrid — local entities + global relations
//! - Exploratory (L3): Global — thematic / relationship-centric
//! - Comparative: Mix — multi-entity needs fused arms
//! - Procedural: Mix — chunks + graph fusion for procedures
//!
//! Explicit `QueryRequest.mode` always wins; adaptive mode only applies when
//! mode is omitted and `QueryEngineConfig.use_adaptive_mode` is true.

use serde::{Deserialize, Serialize};

/// Query intent classification for adaptive retrieval.
///
/// Beyond LightRAG: classify intent to select the optimal retrieval strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum QueryIntent {
    /// "What is X?" — Level-1 fact lookup.
    /// Preferred mode: Naive (avoid graph tax; GraphRAG-Bench).
    Factual,

    /// "How does X relate to Y?" — multi-hop / relational.
    /// Preferred mode: Hybrid (local + global; EQ Hybrid includes naive).
    Relational,

    /// "Tell me about X" / overview — thematic synthesis.
    /// Preferred mode: Global (relationship-centric / community expand).
    #[default]
    Exploratory,

    /// "Compare X and Y" — multi-entity parallel.
    /// Preferred mode: Mix (weighted/RRF fusion of all arms).
    Comparative,

    /// "How to do X?" — step-by-step instructions.
    /// Preferred mode: Mix (chunks + graph fusion).
    Procedural,
}

impl QueryIntent {
    /// Recommended query mode for this intent (SPEC-046 P0.1).
    ///
    /// Aligns with GraphRAG-Bench: skip graphs on L1 facts; use graph arms on
    /// L2/L3; reserve Mix for comparative/procedural workloads.
    pub fn recommended_mode(&self) -> crate::modes::QueryMode {
        match self {
            QueryIntent::Factual => crate::modes::QueryMode::Naive,
            QueryIntent::Relational => crate::modes::QueryMode::Hybrid,
            QueryIntent::Exploratory => crate::modes::QueryMode::Global,
            QueryIntent::Comparative => crate::modes::QueryMode::Mix,
            QueryIntent::Procedural => crate::modes::QueryMode::Mix,
        }
    }

    /// Parse from string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "factual" => QueryIntent::Factual,
            "relational" => QueryIntent::Relational,
            "exploratory" => QueryIntent::Exploratory,
            "comparative" => QueryIntent::Comparative,
            "procedural" => QueryIntent::Procedural,
            _ => QueryIntent::Exploratory,
        }
    }

    /// Heuristic classification based on query text patterns.
    ///
    /// Fast fallback when LLM classification is unavailable.
    /// Order matters: procedural before relational (`how to` vs `how does`).
    pub fn classify_heuristic(query: &str) -> Self {
        let lower = query.to_lowercase();
        let trimmed = lower.trim();

        if trimmed.is_empty() {
            return QueryIntent::Exploratory;
        }

        // Procedural indicators (before relational: "how to" vs "how does")
        if trimmed.starts_with("how to ")
            || trimmed.starts_with("how do i ")
            || trimmed.starts_with("how do you ")
            || trimmed.contains("step by step")
            || trimmed.contains("instructions")
            || (trimmed.contains("guide") && !trimmed.starts_with("what "))
        {
            return QueryIntent::Procedural;
        }

        // Comparative indicators
        if trimmed.contains(" vs ")
            || trimmed.contains(" versus ")
            || trimmed.contains("compare ")
            || trimmed.contains("difference between")
            || trimmed.contains("differences between")
            || trimmed.contains("similarities between")
        {
            return QueryIntent::Comparative;
        }

        // Yes/no / closed Factoid (GraphRAG Fact Retrieval) — before relational
        // so "Is X associated with Y?" is not swallowed by "associated with".
        // 028 A2: keep multi-aspect "is … and how …" as exploratory later.
        if (trimmed.starts_with("is ")
            || trimmed.starts_with("are ")
            || trimmed.starts_with("does ")
            || trimmed.starts_with("do ")
            || trimmed.starts_with("can ")
            || trimmed.starts_with("was ")
            || trimmed.starts_with("were "))
            && !trimmed.contains(" and how ")
            && !trimmed.contains("stages")
            && !trimmed.contains("types of")
        {
            return QueryIntent::Factual;
        }

        // GraphRAG Contextual Summarize: "How are the stages… distinguishing features?"
        // must not be swallowed by relational "how are …" below.
        if (trimmed.starts_with("how are ") || trimmed.starts_with("how is "))
            && (trimmed.contains("stages")
                || trimmed.contains("distinguishing features")
                || trimmed.contains("classified")
                || trimmed.contains("types of")
                || trimmed.contains("main types"))
        {
            return QueryIntent::Exploratory;
        }

        // Relational indicators
        // Include bare "how do …" (GraphRAG Summarize/Complex) after procedural
        // "how do i/you" already returned above.
        if trimmed.contains(" relate ")
            || trimmed.contains("relationship between")
            || trimmed.contains("connection between")
            || trimmed.contains("linked to")
            || trimmed.contains("associated with")
            || trimmed.starts_with("how does ")
            || trimmed.starts_with("how do ")
            || trimmed.starts_with("how are ")
            || trimmed.starts_with("how is ")
        {
            return QueryIntent::Relational;
        }

        // Exploratory / thematic (L3) — before factual so
        // "What are the main themes?" is not misclassified as L1.
        // 021 F1: GraphRAG Contextual Summarize cues → chunk floor via truncation.
        if trimmed.starts_with("tell me about")
            || trimmed.starts_with("explain ")
            || trimmed.starts_with("describe ")
            || trimmed.contains("overview")
            || trimmed.contains("summary of")
            || trimmed.contains("summarize ")
            || trimmed.contains("summarise ")
            || trimmed.contains("main themes")
            || trimmed.contains("key themes")
            || trimmed.contains("key points")
            || trimmed.contains("distinguishing features")
            || trimmed.contains("broader")
        {
            return QueryIntent::Exploratory;
        }

        // Factual indicators (L1) — include count/quantity (MMLongBench / GraphRAG L1).
        // 028 A2: bare "What <noun>…?" / "Which …?" closed lookups after exploratory
        // multi-aspect cues above ("main themes", stages, …).
        if trimmed.starts_with("what is ")
            || trimmed.starts_with("what are ")
            || trimmed.starts_with("what ")
            || trimmed.starts_with("who is ")
            || trimmed.starts_with("who are ")
            || trimmed.starts_with("when ")
            || trimmed.starts_with("where ")
            || trimmed.starts_with("define ")
            || trimmed.starts_with("what's ")
            || trimmed.starts_with("whats ")
            || trimmed.starts_with("how many ")
            || trimmed.starts_with("how much ")
            || trimmed.starts_with("which ")
            || trimmed.starts_with("according to ")
        {
            return QueryIntent::Factual;
        }

        QueryIntent::Exploratory
    }
}

impl std::fmt::Display for QueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryIntent::Factual => write!(f, "factual"),
            QueryIntent::Relational => write!(f, "relational"),
            QueryIntent::Exploratory => write!(f, "exploratory"),
            QueryIntent::Comparative => write!(f, "comparative"),
            QueryIntent::Procedural => write!(f, "procedural"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factual_detection() {
        assert_eq!(
            QueryIntent::classify_heuristic("What is machine learning?"),
            QueryIntent::Factual
        );
        assert_eq!(
            QueryIntent::classify_heuristic("Who is Sarah Chen?"),
            QueryIntent::Factual
        );
        assert_eq!(
            QueryIntent::classify_heuristic("What's Rust?"),
            QueryIntent::Factual
        );
        assert_eq!(
            QueryIntent::classify_heuristic("How many samples in MMMU?"),
            QueryIntent::Factual
        );
        assert_eq!(
            QueryIntent::classify_heuristic("According to this paper, which image type?"),
            QueryIntent::Factual
        );
        // 028 A2: yes/no Factoid before "associated with" → relational
        assert_eq!(
            QueryIntent::classify_heuristic(
                "Is autoimmune disease associated with increased BCC risk?"
            ),
            QueryIntent::Factual
        );
    }

    #[test]
    fn test_relational_detection() {
        assert_eq!(
            QueryIntent::classify_heuristic("How does Sarah relate to the project?"),
            QueryIntent::Relational
        );
        assert_eq!(
            QueryIntent::classify_heuristic("What is the relationship between A and B?"),
            QueryIntent::Relational
        );
        // GraphRAG Contextual Summarize / Complex often use bare "How do …".
        assert_eq!(
            QueryIntent::classify_heuristic(
                "How do biomarkers influence treatment selection in colon cancer?"
            ),
            QueryIntent::Relational
        );
    }

    #[test]
    fn test_summarize_cues_exploratory() {
        assert_eq!(
            QueryIntent::classify_heuristic(
                "How are the stages of esophageal cancer defined and what are their distinguishing features?"
            ),
            QueryIntent::Exploratory
        );
    }

    #[test]
    fn test_comparative_detection() {
        assert_eq!(
            QueryIntent::classify_heuristic("Compare Python vs Rust"),
            QueryIntent::Comparative
        );
        assert_eq!(
            QueryIntent::classify_heuristic("What's the difference between X and Y?"),
            QueryIntent::Comparative
        );
    }

    #[test]
    fn test_procedural_detection() {
        assert_eq!(
            QueryIntent::classify_heuristic("How to install PostgreSQL?"),
            QueryIntent::Procedural
        );
        assert_eq!(
            QueryIntent::classify_heuristic("Step by step guide to setup"),
            QueryIntent::Procedural
        );
    }

    #[test]
    fn test_exploratory_detection() {
        assert_eq!(
            QueryIntent::classify_heuristic("Tell me about the project"),
            QueryIntent::Exploratory
        );
        assert_eq!(
            QueryIntent::classify_heuristic("Explain quantum computing"),
            QueryIntent::Exploratory
        );
        assert_eq!(
            QueryIntent::classify_heuristic("What are the main themes?"),
            QueryIntent::Exploratory
        );
    }

    #[test]
    fn test_empty_and_default() {
        assert_eq!(
            QueryIntent::classify_heuristic(""),
            QueryIntent::Exploratory
        );
        assert_eq!(
            QueryIntent::classify_heuristic("   "),
            QueryIntent::Exploratory
        );
        assert_eq!(
            QueryIntent::classify_heuristic("Random query without clear intent"),
            QueryIntent::Exploratory
        );
    }

    #[test]
    fn test_recommended_modes_spec046() {
        assert_eq!(
            QueryIntent::Factual.recommended_mode(),
            crate::modes::QueryMode::Naive
        );
        assert_eq!(
            QueryIntent::Relational.recommended_mode(),
            crate::modes::QueryMode::Hybrid
        );
        assert_eq!(
            QueryIntent::Exploratory.recommended_mode(),
            crate::modes::QueryMode::Global
        );
        assert_eq!(
            QueryIntent::Comparative.recommended_mode(),
            crate::modes::QueryMode::Mix
        );
        assert_eq!(
            QueryIntent::Procedural.recommended_mode(),
            crate::modes::QueryMode::Mix
        );
    }

    #[test]
    fn contract_what_is_x_routes_to_naive() {
        let intent = QueryIntent::classify_heuristic("What is X?");
        assert_eq!(intent, QueryIntent::Factual);
        assert_eq!(intent.recommended_mode(), crate::modes::QueryMode::Naive);
    }
}
