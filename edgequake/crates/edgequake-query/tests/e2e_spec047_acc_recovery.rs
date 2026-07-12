//! SPEC-047 / 020 Acc recovery — empty-arm prune + A3 factual graph tax.

use edgequake_query::context::{
    QueryContext, RetrievedChunk, RetrievedEntity, RetrievedRelationship,
};
use edgequake_query::hybrid_merge::{merge_hybrid_contexts, prune_empty_arm_graph};
use edgequake_query::keywords::QueryIntent;
use edgequake_query::tokenizer::MockTokenizer;
use edgequake_query::{balance_context, truncation_config_for_intent, TruncationConfig};

#[test]
fn e2e_a3_empty_local_does_not_pollute_naive_context() {
    let mut local = QueryContext::new();
    for i in 0..50 {
        local.add_entity(RetrievedEntity::new(
            format!("E{i}"),
            "ORG",
            "desc ".repeat(20),
        ));
        local.add_relationship(RetrievedRelationship::new(
            format!("E{i}"),
            format!("E{}", i + 1),
            "REL",
        ));
    }
    // local: entities only, no chunks — classic post-B2 pollution
    let mut naive = QueryContext::new();
    naive.add_chunk(
        RetrievedChunk::new("page-chunk", "The answer is 42 on page 3", 0.95).with_page(3),
    );

    let merged = merge_hybrid_contexts(local, QueryContext::new(), naive, 10);
    assert!(
        merged.entities.is_empty(),
        "empty local must not inject entities; n={}",
        merged.entities.len()
    );
    assert_eq!(merged.chunks.len(), 1);
    assert!(merged.chunks[0].content.contains("42"));
}

#[test]
fn e2e_a3_factual_truncation_prefers_chunks_over_entity_flood() {
    let tokenizer = MockTokenizer::with_rate(1.0);
    let base = TruncationConfig {
        max_entity_tokens: 10_000,
        max_relation_tokens: 10_000,
        max_total_tokens: 500,
        buffer_tokens: 0,
        min_chunk_budget_ratio: 0.40,
    };
    let cfg = truncation_config_for_intent(&base, QueryIntent::Factual);
    assert!(cfg.min_chunk_budget_ratio >= 0.55);
    assert!(cfg.max_entity_tokens <= 2_000);

    let entities: Vec<_> = (0..40)
        .map(|i| RetrievedEntity::new(format!("Ent{i}"), "T", "x".repeat(40)))
        .collect();
    let chunks = vec![
        RetrievedChunk::new("c1", &"evidence-page-chunk-".repeat(8), 0.9),
        RetrievedChunk::new("c2", &"more-evidence-chunk-".repeat(8), 0.8),
    ];

    let (be, _br, bc) = balance_context(entities, vec![], chunks, &cfg, &tokenizer);
    assert!(!bc.is_empty(), "factual must keep evidence chunks");
    assert!(
        be.len() < 40,
        "factual graph tax must shrink entities; kept {}",
        be.len()
    );
}

#[test]
fn e2e_a3_prune_helper_is_idempotent_on_chunked_arms() {
    let mut ctx = QueryContext::new();
    ctx.add_chunk(RetrievedChunk::new("c", "body", 1.0));
    ctx.add_entity(RetrievedEntity::new("Keep", "T", "d"));
    let pruned = prune_empty_arm_graph(ctx);
    assert_eq!(pruned.entities.len(), 1);
    assert_eq!(pruned.chunks.len(), 1);
}
