//! GraphRAG-Bench-style difficulty levels (SPEC-046 P2.2).
//!
//! Four levels from ICLR 2026 GraphRAG-Bench:
//! L1 Fact → L2 Reasoning → L3 Summary → L4 Faithfulness/Creative.
//! Used for adaptive-routing contracts and synthetic e2e fixtures.

use serde::{Deserialize, Serialize};

use crate::keywords::QueryIntent;
use crate::modes::QueryMode;

/// Task difficulty aligned with GraphRAG-Bench.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphRagLevel {
    /// Isolated fact lookup — naive RAG should win.
    FactRetrieval,
    /// Multi-hop / relational reasoning — graph helps.
    ComplexReasoning,
    /// Thematic synthesis across fragments.
    ContextSummary,
    /// Generation grounded in retrieved evidence.
    Faithfulness,
}

impl GraphRagLevel {
    /// Evidence-aligned preferred mode for this level (SPEC-046).
    pub fn preferred_mode(self) -> QueryMode {
        match self {
            Self::FactRetrieval => QueryMode::Naive,
            Self::ComplexReasoning => QueryMode::Hybrid,
            Self::ContextSummary => QueryMode::Global,
            Self::Faithfulness => QueryMode::Mix,
        }
    }

    /// Map from query intent (best-effort).
    ///
    /// Comparative uses Mix (multi-entity fusion), closer to Faithfulness/L4
    /// than pure L2 Hybrid relational walks.
    pub fn from_intent(intent: QueryIntent) -> Self {
        match intent {
            QueryIntent::Factual => Self::FactRetrieval,
            QueryIntent::Relational => Self::ComplexReasoning,
            QueryIntent::Exploratory => Self::ContextSummary,
            QueryIntent::Comparative | QueryIntent::Procedural => Self::Faithfulness,
        }
    }
}

/// One synthetic eval case for CI (no external corpus required).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRagBenchCase {
    pub id: &'static str,
    pub level: GraphRagLevel,
    pub query: &'static str,
    pub expected_mode: QueryMode,
    pub expected_intent: QueryIntent,
}

/// Built-in subset for nightly / unit gates.
pub fn spec046_synthetic_bench() -> Vec<GraphRagBenchCase> {
    vec![
        GraphRagBenchCase {
            id: "l1_what_is",
            level: GraphRagLevel::FactRetrieval,
            query: "What is Apache AGE?",
            expected_mode: QueryMode::Naive,
            expected_intent: QueryIntent::Factual,
        },
        GraphRagBenchCase {
            id: "l1_who_is",
            level: GraphRagLevel::FactRetrieval,
            query: "Who is Sarah Chen?",
            expected_mode: QueryMode::Naive,
            expected_intent: QueryIntent::Factual,
        },
        GraphRagBenchCase {
            id: "l2_relationship",
            level: GraphRagLevel::ComplexReasoning,
            query: "What is the relationship between Alice and Bob?",
            expected_mode: QueryMode::Hybrid,
            expected_intent: QueryIntent::Relational,
        },
        GraphRagBenchCase {
            id: "l2_how_does",
            level: GraphRagLevel::ComplexReasoning,
            query: "How does EdgeQuake relate to LightRAG?",
            expected_mode: QueryMode::Hybrid,
            expected_intent: QueryIntent::Relational,
        },
        GraphRagBenchCase {
            id: "l3_themes",
            level: GraphRagLevel::ContextSummary,
            query: "What are the main themes in the corpus?",
            expected_mode: QueryMode::Global,
            expected_intent: QueryIntent::Exploratory,
        },
        GraphRagBenchCase {
            id: "l3_overview",
            level: GraphRagLevel::ContextSummary,
            query: "Tell me about the knowledge graph architecture",
            expected_mode: QueryMode::Global,
            expected_intent: QueryIntent::Exploratory,
        },
        GraphRagBenchCase {
            id: "l4_compare",
            level: GraphRagLevel::Faithfulness,
            query: "Compare Mix vs Hybrid query modes",
            expected_mode: QueryMode::Mix,
            expected_intent: QueryIntent::Comparative,
        },
        GraphRagBenchCase {
            id: "l4_howto",
            level: GraphRagLevel::Faithfulness,
            query: "How to configure EDGEQUAKE_GRAPH_WALK?",
            expected_mode: QueryMode::Mix,
            expected_intent: QueryIntent::Procedural,
        },
    ]
}

/// Validate that heuristic intent + recommended_mode match the bench case.
pub fn assert_case_routing(case: &GraphRagBenchCase) {
    let intent = QueryIntent::classify_heuristic(case.query);
    assert_eq!(
        intent, case.expected_intent,
        "case {}: intent mismatch for {:?}",
        case.id, case.query
    );
    assert_eq!(
        intent.recommended_mode(),
        case.expected_mode,
        "case {}: mode mismatch",
        case.id
    );
    assert_eq!(
        case.level.preferred_mode(),
        case.expected_mode,
        "case {}: level preferred_mode drift",
        case.id
    );
}

/// Per-case routing outcome for CI JSON reports (SPEC-046 P2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRagBenchCaseResult {
    pub id: String,
    pub level: GraphRagLevel,
    pub query: String,
    pub expected_mode: QueryMode,
    pub actual_mode: QueryMode,
    pub expected_intent: QueryIntent,
    pub actual_intent: QueryIntent,
    pub passed: bool,
}

/// Aggregate report for the synthetic GraphRAG-Bench subset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphRagBenchReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f32,
    pub cases: Vec<GraphRagBenchCaseResult>,
}

/// Run the built-in synthetic subset and return a machine-readable report.
pub fn run_spec046_bench_report() -> GraphRagBenchReport {
    let cases = spec046_synthetic_bench();
    let mut results = Vec::with_capacity(cases.len());
    for case in &cases {
        let intent = QueryIntent::classify_heuristic(case.query);
        let mode = intent.recommended_mode();
        let passed = intent == case.expected_intent
            && mode == case.expected_mode
            && case.level.preferred_mode() == case.expected_mode;
        results.push(GraphRagBenchCaseResult {
            id: case.id.to_string(),
            level: case.level,
            query: case.query.to_string(),
            expected_mode: case.expected_mode,
            actual_mode: mode,
            expected_intent: case.expected_intent,
            actual_intent: intent,
            passed,
        });
    }
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    GraphRagBenchReport {
        total,
        passed,
        failed: total.saturating_sub(passed),
        pass_rate: if total == 0 {
            1.0
        } else {
            passed as f32 / total as f32
        },
        cases: results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_synthetic_cases_route_correctly() {
        for case in spec046_synthetic_bench() {
            assert_case_routing(&case);
        }
    }

    #[test]
    fn level_from_intent_roundtrip_modes() {
        assert_eq!(
            GraphRagLevel::from_intent(QueryIntent::Factual).preferred_mode(),
            QueryMode::Naive
        );
        assert_eq!(
            GraphRagLevel::from_intent(QueryIntent::Exploratory).preferred_mode(),
            QueryMode::Global
        );
    }

    #[test]
    fn bench_report_is_full_pass() {
        let report = run_spec046_bench_report();
        assert_eq!(report.failed, 0);
        assert_eq!(report.pass_rate, 1.0);
        assert_eq!(report.total, spec046_synthetic_bench().len());
    }
}
