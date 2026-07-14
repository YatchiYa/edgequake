//! SPEC-047 / 019–020 — Query grounding + chunk budget + calibrated refusal e2e.
//!
//! Code is law: page/modality must appear in the LLM context string; chunk floor
//! must protect evidence chunks from graph token tax; grounding must be
//! entailment-first (020 A1) while still allowing honest refusal (019 Q8).

use edgequake_query::tokenizer::MockTokenizer;
use edgequake_query::{
    allows_honest_refusal, balance_context, format_chunk_block, format_query_context,
    grounding_instructions, is_entailment_first, min_chunk_token_budget,
    parse_min_chunk_budget_ratio, RetrievedChunk, RetrievedEntity, RetrievedRelationship,
    TruncationConfig,
};

#[test]
fn e2e_q11_context_string_exposes_page_and_modality() {
    let chunk = RetrievedChunk::new("c-chart", "Series A peaks at 42 in Q3", 0.93)
        .with_page(14)
        .with_modality("chart")
        .with_document_id("doc-mm");
    let entity = RetrievedEntity::new("Acme Corp", "ORG", "Subject of the chart");
    let rel = RetrievedRelationship::new("Acme Corp", "Q3", "REPORTED");

    let ctx = format_query_context(&[entity], &[rel], &[chunk.clone()]);

    assert!(
        ctx.contains("page=14"),
        "LLM must see page= for grounding; got:\n{ctx}"
    );
    assert!(
        ctx.contains("modality=chart"),
        "LLM must see modality= for chart questions; got:\n{ctx}"
    );
    assert!(
        ctx.contains("[1] (score: 0.930)"),
        "chunk citation id required; got:\n{ctx}"
    );
    assert!(ctx.contains("page=N"), "legend should explain page headers");

    let block = format_chunk_block(1, &chunk);
    assert!(block.contains("page=14 modality=chart"));
}

#[test]
fn e2e_q12_grounding_instructions_cite_and_refuse() {
    let g = grounding_instructions();
    assert!(g.contains("page="));
    assert!(g.contains("[N]"));
    assert!(
        allows_honest_refusal(g),
        "must keep honest refusal path; got:\n{g}"
    );
}

#[test]
fn e2e_a1_grounding_is_entailment_first_calibrated() {
    let g = grounding_instructions();
    assert!(
        is_entailment_first(g),
        "020 A1: answer when evidence supports; refuse only when absent; got:\n{g}"
    );
    assert!(allows_honest_refusal(g));
    let lower = g.to_lowercase();
    assert!(
        lower.contains("prefer a partial answer"),
        "partial grounded answers must beat Not answerable"
    );
    assert!(
        !lower.contains("never say not answerable"),
        "must not ban Not answerable"
    );
}

#[test]
fn e2e_q13_chunk_budget_floor_preserves_evidence() {
    assert!((parse_min_chunk_budget_ratio("") - 0.40).abs() < f32::EPSILON);
    let config = TruncationConfig {
        max_entity_tokens: 10_000,
        max_relation_tokens: 10_000,
        max_total_tokens: 500,
        buffer_tokens: 0,
        min_chunk_budget_ratio: 0.40,
    };
    assert_eq!(min_chunk_token_budget(&config), 200);

    let tokenizer = MockTokenizer::with_rate(1.0);
    let entities: Vec<_> = (0..40)
        .map(|i| {
            RetrievedEntity::new(
                format!("Entity{i}"),
                "CONCEPT",
                "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
            )
        })
        .collect();
    let chunks = vec![
        RetrievedChunk::new("ev1", "GOLD_ANSWER_TOKEN_ALPHA", 1.0).with_page(2),
        RetrievedChunk::new("ev2", "GOLD_ANSWER_TOKEN_BETA", 0.9).with_page(3),
    ];

    let (kept_entities, _rels, kept_chunks) =
        balance_context(entities, vec![], chunks, &config, &tokenizer);

    assert!(
        !kept_chunks.is_empty(),
        "floor must retain at least one evidence chunk"
    );
    assert!(
        kept_entities.len() < 40,
        "graph must shrink under chunk floor; kept {}",
        kept_entities.len()
    );
    let joined = kept_chunks
        .iter()
        .map(|c| c.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        joined.contains("GOLD_ANSWER"),
        "evidence content must survive truncation: {joined}"
    );
}

#[test]
fn e2e_query_context_to_string_delegates_to_formatter() {
    use edgequake_query::QueryContext;

    let mut ctx = QueryContext::new();
    ctx.add_chunk(
        RetrievedChunk::new("c1", "Valuation is $20B", 0.88)
            .with_page(5)
            .with_modality("table"),
    );
    let s = ctx.to_context_string();
    assert!(s.contains("page=5"));
    assert!(s.contains("modality=table"));
    assert!(s.contains("Valuation is $20B"));
}
