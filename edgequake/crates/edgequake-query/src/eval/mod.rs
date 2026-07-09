//! RAG evaluation harness (SPEC-025 8.1 / SPEC-046 P2.2).

pub mod golden_set;
pub mod graphrag_levels;
pub mod metrics;

pub use golden_set::{load_spec025_golden_set, GoldenQaCase, GoldenSetStats};
pub use graphrag_levels::{
    assert_case_routing, run_spec046_bench_report, spec046_synthetic_bench, GraphRagBenchCase,
    GraphRagBenchCaseResult, GraphRagBenchReport, GraphRagLevel,
};
pub use metrics::{context_entity_recall, keyword_recall_in_text};
