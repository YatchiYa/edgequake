//! GraphRAG-Bench-style corpus fixture (SPEC-046 EQ-046-18).
//!
//! Checked-in mini corpus mirrors GraphRAG-Bench schema fields
//! (`id`, `question`, `question_type`, `evidence`, `gold_answer`) without
//! network/HF downloads — deterministic CI. Full HF corpus remains optional.
//!
//! Retrieval ACC: bipartite PPR from explicit `seed_entities` must surface
//! `required_chunk_ids` in top-k (no fragile free-text seed matching).

use serde::{Deserialize, Serialize};

use crate::context::{QueryContext, RetrievedEntity, RetrievedRelationship};
use crate::eval::graphrag_levels::GraphRagLevel;
use crate::eval::metrics::keyword_recall_in_text;
use crate::graph_ppr::{rank_chunks_bipartite_ppr, PprConfig};
use crate::keywords::QueryIntent;
use crate::modes::QueryMode;
use edgequake_storage::traits::GraphEdge;

/// One fixture case aligned with GraphRAG-Bench question record shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorpusCase {
    pub id: &'static str,
    /// GraphRAG-Bench `question_type` string (Fact Retrieval, …).
    pub question_type: &'static str,
    pub level: GraphRagLevel,
    pub question: &'static str,
    pub evidence: &'static str,
    pub gold_answer: &'static str,
    /// Entity names used as PPR teleport seeds (deterministic).
    pub seed_entities: &'static [&'static str],
    /// Chunk IDs that must appear in top-k bipartite retrieval for pass.
    pub required_chunk_ids: &'static [&'static str],
    pub expected_mode: QueryMode,
    pub expected_intent: QueryIntent,
}

/// In-memory KG slice for the mini corpus (entities, edges, mentions).
#[derive(Debug, Clone)]
pub struct CorpusKgSlice {
    pub entities: Vec<RetrievedEntity>,
    pub edges: Vec<GraphEdge>,
    pub relationships: Vec<RetrievedRelationship>,
}

/// Built-in mini corpus (Novel + Medical flavored) — no HF dependency.
///
/// Questions are worded so [`QueryIntent::classify_heuristic`] matches
/// `expected_intent` (non-flaky routing gate).
pub fn spec046_mini_corpus() -> Vec<CorpusCase> {
    vec![
        CorpusCase {
            id: "novel_l1_mont_st_michel",
            question_type: "Fact Retrieval",
            level: GraphRagLevel::FactRetrieval,
            question: "What is Mont St. Michel?",
            evidence: "Mont St. Michel stands in Normandy on the coast of France.",
            gold_answer: "Normandy coastal abbey",
            seed_entities: &["MONT_ST_MICHEL"],
            required_chunk_ids: &["chunk_normandy"],
            expected_mode: QueryMode::Naive,
            expected_intent: QueryIntent::Factual,
        },
        CorpusCase {
            id: "novel_l2_hinze_felicia",
            question_type: "Complex Reasoning",
            level: GraphRagLevel::ComplexReasoning,
            question: "How does Hinze relate to Felicia regarding England's rulers?",
            evidence:
                "Hinze and Felicia signed a pact that mirrored the distrust of England's rulers.",
            gold_answer: "The pact reflected distrust of England's rulers.",
            seed_entities: &["HINZE"],
            required_chunk_ids: &["chunk_hinze_pact"],
            expected_mode: QueryMode::Hybrid,
            expected_intent: QueryIntent::Relational,
        },
        CorpusCase {
            id: "novel_l3_curgenven",
            question_type: "Contextual Summarize",
            level: GraphRagLevel::ContextSummary,
            question: "Tell me about John Curgenven as a Cornish boatman",
            evidence:
                "John Curgenven is a Cornish boatman who guides visitors exploring the coast.",
            gold_answer: "He guides visitors along the Cornish coast.",
            seed_entities: &["JOHN_CURGENVEN"],
            required_chunk_ids: &["chunk_curgenven"],
            expected_mode: QueryMode::Global,
            expected_intent: QueryIntent::Exploratory,
        },
        CorpusCase {
            id: "medical_l1_erica_vagans",
            question_type: "Fact Retrieval",
            level: GraphRagLevel::FactRetrieval,
            question: "What is Erica vagans?",
            evidence: "Erica vagans is commonly known as Cornish heath.",
            gold_answer: "Cornish heath",
            seed_entities: &["ERICA_VAGANS"],
            required_chunk_ids: &["chunk_erica"],
            expected_mode: QueryMode::Naive,
            expected_intent: QueryIntent::Factual,
        },
        CorpusCase {
            id: "medical_l4_compare_treatments",
            question_type: "Creative Generation",
            level: GraphRagLevel::Faithfulness,
            question: "Compare ACE inhibitors versus beta blockers for hypertension",
            evidence:
                "ACE inhibitors dilate vessels; beta blockers reduce heart rate for hypertension.",
            gold_answer: "ACE inhibitors dilate vessels while beta blockers lower heart rate.",
            seed_entities: &["ACE_INHIBITORS", "BETA_BLOCKERS"],
            required_chunk_ids: &["chunk_ace", "chunk_beta"],
            expected_mode: QueryMode::Mix,
            expected_intent: QueryIntent::Comparative,
        },
    ]
}

/// Seeded KG for bipartite retrieval ACC (deterministic, in-memory).
pub fn spec046_mini_corpus_kg() -> CorpusKgSlice {
    let mut mont = RetrievedEntity::new("MONT_ST_MICHEL", "PLACE", "island abbey");
    mont.score = 1.0;
    mont.source_chunk_ids = vec!["chunk_normandy".into()];

    let mut normandy = RetrievedEntity::new("NORMANDY", "REGION", "French region");
    normandy.score = 0.8;
    normandy.source_chunk_ids = vec!["chunk_normandy".into()];

    let mut hinze = RetrievedEntity::new("HINZE", "PERSON", "pact signer");
    hinze.score = 1.0;
    hinze.source_chunk_ids = vec!["chunk_hinze_pact".into()];

    let mut felicia = RetrievedEntity::new("FELICIA", "PERSON", "pact counterparty");
    felicia.score = 0.7;
    felicia.source_chunk_ids = vec!["chunk_hinze_pact".into()];

    let mut england = RetrievedEntity::new("ENGLAND", "PLACE", "rulers distrust");
    england.score = 0.5;
    england.source_chunk_ids = vec!["chunk_hinze_pact".into()];

    let mut curgenven = RetrievedEntity::new("JOHN_CURGENVEN", "PERSON", "Cornish boatman");
    curgenven.score = 1.0;
    curgenven.source_chunk_ids = vec!["chunk_curgenven".into()];

    let mut erica = RetrievedEntity::new("ERICA_VAGANS", "SPECIES", "heath plant");
    erica.score = 1.0;
    erica.source_chunk_ids = vec!["chunk_erica".into()];

    let mut ace = RetrievedEntity::new("ACE_INHIBITORS", "DRUG", "dilate vessels");
    ace.score = 1.0;
    ace.source_chunk_ids = vec!["chunk_ace".into()];

    let mut beta = RetrievedEntity::new("BETA_BLOCKERS", "DRUG", "reduce heart rate");
    beta.score = 0.9;
    beta.source_chunk_ids = vec!["chunk_beta".into()];

    let mut distraction = RetrievedEntity::new("UNRELATED", "CONCEPT", "noise");
    distraction.score = 0.01;
    distraction.source_chunk_ids = vec!["chunk_noise".into()];

    let edges = vec![
        GraphEdge {
            source: "MONT_ST_MICHEL".into(),
            target: "NORMANDY".into(),
            properties: Default::default(),
        },
        GraphEdge {
            source: "HINZE".into(),
            target: "FELICIA".into(),
            properties: Default::default(),
        },
        GraphEdge {
            source: "HINZE".into(),
            target: "ENGLAND".into(),
            properties: Default::default(),
        },
        GraphEdge {
            source: "ACE_INHIBITORS".into(),
            target: "BETA_BLOCKERS".into(),
            properties: Default::default(),
        },
    ];

    let relationships = vec![
        RetrievedRelationship::new("MONT_ST_MICHEL", "NORMANDY", "LOCATED_IN")
            .with_source_chunk_id("chunk_normandy"),
        RetrievedRelationship::new("HINZE", "FELICIA", "AGREED_WITH")
            .with_source_chunk_id("chunk_hinze_pact"),
        RetrievedRelationship::new("ACE_INHIBITORS", "BETA_BLOCKERS", "COMPARED_TO")
            .with_source_chunk_id("chunk_ace"),
    ];

    CorpusKgSlice {
        entities: vec![
            mont,
            normandy,
            hinze,
            felicia,
            england,
            curgenven,
            erica,
            ace,
            beta,
            distraction,
        ],
        edges,
        relationships,
    }
}

/// Per-case corpus ACC result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorpusCaseResult {
    pub id: String,
    pub routing_passed: bool,
    pub retrieval_passed: bool,
    pub evidence_recall: f32,
    pub retrieved_chunks: Vec<String>,
    pub passed: bool,
}

/// Aggregate corpus ACC report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorpusAccReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f32,
    pub cases: Vec<CorpusCaseResult>,
}

impl CorpusAccReport {
    pub fn is_full_pass(&self) -> bool {
        self.failed == 0
    }
}

/// Run bipartite PPR retrieval ACC against the mini corpus (deterministic).
pub fn run_spec046_corpus_acc_report() -> CorpusAccReport {
    let kg = spec046_mini_corpus_kg();
    let cases = spec046_mini_corpus();
    let mut results = Vec::with_capacity(cases.len());

    let links: Vec<(String, String)> = kg
        .entities
        .iter()
        .flat_map(|e| {
            e.source_chunk_ids
                .iter()
                .map(|c| (e.name.clone(), c.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    for case in &cases {
        let intent = QueryIntent::classify_heuristic(case.question);
        let mode = intent.recommended_mode();
        let routing_passed = intent == case.expected_intent
            && mode == case.expected_mode
            && case.level.preferred_mode() == case.expected_mode;

        let seeds: Vec<String> = case
            .seed_entities
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let ranked = rank_chunks_bipartite_ppr(&kg.edges, &links, &seeds, &PprConfig::default(), 5);

        let retrieval_passed = case
            .required_chunk_ids
            .iter()
            .all(|need| ranked.iter().any(|c| c == *need));

        let tokens: Vec<String> = case
            .gold_answer
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 4)
            .map(|t| t.to_lowercase())
            .collect();
        let evidence_recall = if tokens.is_empty() {
            1.0
        } else {
            keyword_recall_in_text(case.evidence, &tokens)
        };

        let passed = routing_passed && retrieval_passed && evidence_recall >= 0.3;
        results.push(CorpusCaseResult {
            id: case.id.to_string(),
            routing_passed,
            retrieval_passed,
            evidence_recall,
            retrieved_chunks: ranked,
            passed,
        });
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    CorpusAccReport {
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

/// Build a QueryContext from the mini KG (for e2e engine tests).
pub fn mini_corpus_query_context() -> QueryContext {
    let kg = spec046_mini_corpus_kg();
    let mut ctx = QueryContext::new();
    for e in kg.entities {
        ctx.add_entity(e);
    }
    for r in kg.relationships {
        ctx.add_relationship(r);
    }
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_corpus_has_all_levels() {
        let cases = spec046_mini_corpus();
        assert!(cases.len() >= 5);
        assert!(cases
            .iter()
            .any(|c| c.level == GraphRagLevel::FactRetrieval));
        assert!(cases
            .iter()
            .any(|c| c.level == GraphRagLevel::ComplexReasoning));
        assert!(cases.iter().any(|c| c.level == GraphRagLevel::Faithfulness));
    }

    #[test]
    fn corpus_cases_match_heuristic_intent() {
        for case in spec046_mini_corpus() {
            let intent = QueryIntent::classify_heuristic(case.question);
            assert_eq!(
                intent, case.expected_intent,
                "case {}: intent drift for {:?}",
                case.id, case.question
            );
            assert_eq!(intent.recommended_mode(), case.expected_mode);
        }
    }

    #[test]
    fn corpus_acc_is_full_pass() {
        let report = run_spec046_corpus_acc_report();
        assert!(
            report.is_full_pass(),
            "corpus ACC failures: {:?}",
            report
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
    }
}
