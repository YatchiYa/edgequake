//! ACC (Acceptance) CI harness for SPEC-046 OPS-P3.22.
//!
//! Combines:
//! 1. GraphRAG-Bench **routing** contracts (`run_spec046_bench_report`)
//! 2. Retrieval **physics** contracts (PPR parse, path prune, truncation)
//! 3. Faithfulness **heuristic** floor on a synthetic grounded answer
//!
//! All checks are deterministic (no env mutation, no network). Live LLM-judge
//! ACC is opt-in via a separate ignored e2e test.

use serde::{Deserialize, Serialize};

use crate::context::{QueryContext, RetrievedChunk, RetrievedEntity, RetrievedRelationship};
use crate::eval::faithfulness::score_faithfulness_heuristic;
use crate::eval::graphrag_levels::{run_spec046_bench_report, GraphRagBenchReport};
use crate::graph_ppr::{parse_graph_walk_mode, GraphWalkMode};
use crate::path_prune::{prune_relationships, PathPruneConfig};
use crate::tokenizer::MockTokenizer;
use crate::truncation::{balance_context, TruncationConfig};

/// One ACC check result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccCheckResult {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

/// Aggregate ACC report for CI JSON artifacts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f32,
    pub routing: GraphRagBenchReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus: Option<crate::eval::graphrag_corpus::CorpusAccReport>,
    pub checks: Vec<AccCheckResult>,
}

impl AccReport {
    /// True when every check passed and routing bench is full-pass.
    pub fn is_full_pass(&self) -> bool {
        self.failed == 0
            && self.routing.failed == 0
            && self
                .corpus
                .as_ref()
                .map(|c| c.is_full_pass())
                .unwrap_or(true)
    }
}

/// Write ACC report JSON to `path` (CI artifact helper — SPEC-046 EQ-046-16).
pub fn write_spec046_acc_report_json(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    let report = run_spec046_acc_report();
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Run the deterministic ACC suite (no LLM, no Postgres).
pub fn run_spec046_acc_report() -> AccReport {
    let routing = run_spec046_bench_report();
    let corpus = crate::eval::graphrag_corpus::run_spec046_corpus_acc_report();
    let mut checks = Vec::new();

    // --- Routing gate ---
    checks.push(AccCheckResult {
        id: "routing_bench".into(),
        passed: routing.failed == 0,
        detail: format!(
            "pass_rate={:.2} failed={}",
            routing.pass_rate, routing.failed
        ),
    });

    // --- PPR parse / default policy (OPS-P3 bridge) ---
    let default_walk = parse_graph_walk_mode("");
    checks.push(AccCheckResult {
        id: "graph_walk_default_ppr".into(),
        passed: default_walk == GraphWalkMode::Ppr,
        detail: format!("parse(\"\") => {default_walk:?}"),
    });
    let bfs_escape = parse_graph_walk_mode("bfs");
    checks.push(AccCheckResult {
        id: "graph_walk_bfs_escape".into(),
        passed: bfs_escape == GraphWalkMode::Bfs,
        detail: format!("parse(\"bfs\") => {bfs_escape:?}"),
    });

    // --- Path prune reduces relation tax ---
    let rels: Vec<_> = (0..20)
        .map(|i| {
            RetrievedRelationship::new("A", format!("T{i}"), "REL")
                .with_score(i as f32 * 0.05)
                .with_description(if i > 10 { "rich evidence path" } else { "" })
        })
        .collect();
    let cfg = PathPruneConfig {
        drop_fraction: 0.4,
        min_keep: 3,
        min_input: 5,
    };
    let kept = prune_relationships(rels, &cfg);
    checks.push(AccCheckResult {
        id: "path_prune_cap".into(),
        passed: kept.len() == 12,
        detail: format!("kept={}", kept.len()),
    });

    // --- Truncation gives chunks dynamic remainder ---
    let tokenizer = MockTokenizer::with_rate(1.0);
    let trunc = TruncationConfig {
        max_entity_tokens: 50,
        max_relation_tokens: 50,
        max_total_tokens: 100,
        buffer_tokens: 10,
    };
    let entities = vec![RetrievedEntity::new("E", "T", "x")];
    let rels = vec![RetrievedRelationship::new("A", "B", "R")];
    let chunks = vec![
        RetrievedChunk::new("c1", "AAAAAAAAAA", 1.0),
        RetrievedChunk::new("c2", "BBBBBBBBBB", 0.9),
        RetrievedChunk::new("c3", "CCCCCCCCCC", 0.8),
        RetrievedChunk::new("c4", "DDDDDDDDDD", 0.7),
    ];
    let (e, r, c) = balance_context(entities, rels, chunks, &trunc, &tokenizer);
    checks.push(AccCheckResult {
        id: "truncation_budget".into(),
        passed: !e.is_empty() && !r.is_empty() && !c.is_empty(),
        detail: format!("entities={} rels={} chunks={}", e.len(), r.len(), c.len()),
    });

    // --- Faithfulness heuristic floor on grounded answer ---
    let mut grounded = QueryContext::new();
    grounded.add_chunk(RetrievedChunk::new(
        "g1",
        "Apache AGE extends PostgreSQL with graph queries",
        1.0,
    ));
    let faith = score_faithfulness_heuristic(
        "Apache AGE extends PostgreSQL with graph queries",
        &grounded,
    );
    checks.push(AccCheckResult {
        id: "faithfulness_heuristic_floor".into(),
        passed: faith >= 0.8,
        detail: format!("score={faith}"),
    });

    // --- Bipartite dual-node smoke ---
    {
        use crate::graph_ppr::{rank_chunks_bipartite_ppr, PprConfig};
        use edgequake_storage::traits::GraphEdge;
        let edges = vec![GraphEdge {
            source: "SEED".into(),
            target: "N1".into(),
            properties: Default::default(),
        }];
        let links = vec![
            ("SEED".into(), "chunk-hot".into()),
            ("N1".into(), "chunk-cold".into()),
        ];
        let ranked =
            rank_chunks_bipartite_ppr(&edges, &links, &["SEED".into()], &PprConfig::default(), 2);
        checks.push(AccCheckResult {
            id: "bipartite_dual_node".into(),
            passed: ranked.first().map(String::as_str) == Some("chunk-hot"),
            detail: format!("ranked={ranked:?}"),
        });
    }

    // --- Corpus retrieval ACC ---
    checks.push(AccCheckResult {
        id: "corpus_retrieval_acc".into(),
        passed: corpus.is_full_pass(),
        detail: format!("pass_rate={:.2} failed={}", corpus.pass_rate, corpus.failed),
    });

    let passed = checks.iter().filter(|c| c.passed).count();
    let total = checks.len();
    AccReport {
        total,
        passed,
        failed: total.saturating_sub(passed),
        pass_rate: if total == 0 {
            1.0
        } else {
            passed as f32 / total as f32
        },
        routing,
        corpus: Some(corpus),
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acc_report_is_full_pass() {
        let report = run_spec046_acc_report();
        assert!(
            report.is_full_pass(),
            "ACC failures: {:?}",
            report
                .checks
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
        assert_eq!(report.failed, 0);
        assert_eq!(report.routing.failed, 0);
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("routing_bench"));
        assert!(json.contains("graph_walk_default_ppr"));
    }
}
