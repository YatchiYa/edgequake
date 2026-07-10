//! RAG evaluation harness (SPEC-025 8.1 / SPEC-046 P2.2).

pub mod acc_harness;
pub mod faithfulness;
pub mod faithfulness_judge;
pub mod golden_set;
pub mod graphrag_corpus;
pub mod graphrag_levels;
pub mod metrics;

pub use acc_harness::{
    run_spec046_acc_report, write_spec046_acc_report_json, AccCheckResult, AccReport,
};
pub use faithfulness::{
    faithfulness_sample_rate_from_env, maybe_score_faithfulness, parse_faithfulness_sample_rate,
    score_faithfulness_heuristic, should_sample_faithfulness, significant_tokens,
};
pub use faithfulness_judge::{
    build_faithfulness_judge_prompt, faithfulness_judge_enabled_from_env,
    maybe_score_faithfulness_llm, parse_faithfulness_judge_enabled, parse_judge_score,
    score_faithfulness_llm,
};
pub use golden_set::{load_spec025_golden_set, GoldenQaCase, GoldenSetStats};
pub use graphrag_corpus::{
    mini_corpus_query_context, run_spec046_corpus_acc_report, spec046_mini_corpus,
    spec046_mini_corpus_kg, CorpusAccReport, CorpusCase, CorpusCaseResult, CorpusKgSlice,
};
pub use graphrag_levels::{
    assert_case_routing, run_spec046_bench_report, spec046_synthetic_bench, GraphRagBenchCase,
    GraphRagBenchCaseResult, GraphRagBenchReport, GraphRagLevel,
};
pub use metrics::{context_entity_recall, keyword_recall_in_text};
