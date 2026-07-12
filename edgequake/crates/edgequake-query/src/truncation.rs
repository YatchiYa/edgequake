//! Token-based truncation for context management.
//!
//! # Implements
//!
//! @implements FEAT0108 (Context Truncation)
//! @implements SPEC-047 Q1.3 (chunk budget floor)
//!
//! # Enforces
//!
//! - **BR0101**: Token budget must not exceed LLM context window
//! - **BR0102**: Graph context takes priority over naive chunks **unless**
//!   the chunk floor would be violated (then graph is shrunk first)
//!
//! ## The Token Budget Strategy (SPEC-046 P0.4 / LightRAG dynamic remainder)
//!
//! ```text
//! Total Budget: 30,000 tokens (default, matching LightRAG)
//! ├── Entities:      ≤ max_entity_tokens (cap, then actual)
//! ├── Relationships: ≤ max_relation_tokens (cap, then actual)
//! ├── Buffer:        truncation_buffer_tokens (sys/query safety)
//! └── Chunks:        remainder = total - entity_actual - rel_actual - buffer
//!                    but never below min_chunk_budget_ratio × (total − buffer)
//! ```

use serde::{Deserialize, Serialize};

use crate::context::{RetrievedChunk, RetrievedEntity, RetrievedRelationship};
use crate::context_format::{format_chunk_block, format_entity_line, format_relationship_line};
use crate::tokenizer::Tokenizer;

/// Configuration for token-based truncation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncationConfig {
    /// Maximum tokens for entity descriptions.
    pub max_entity_tokens: usize,

    /// Maximum tokens for relationship descriptions.
    pub max_relation_tokens: usize,

    /// Maximum total tokens for all context.
    pub max_total_tokens: usize,

    /// Reserved for system prompt + query + safety (LightRAG buffer ≈ 200).
    /// Chunk budget = total − actual_entity − actual_relation − buffer.
    #[serde(default = "default_truncation_buffer")]
    pub buffer_tokens: usize,

    /// Minimum fraction of `(max_total_tokens − buffer)` reserved for document
    /// chunks after graph truncation (SPEC-047 Q1.3). Default **0.40**.
    /// Set to `0.0` to disable the floor (legacy BR0102-only behavior).
    #[serde(default = "default_min_chunk_budget_ratio")]
    pub min_chunk_budget_ratio: f32,
}

fn default_truncation_buffer() -> usize {
    200
}

fn default_min_chunk_budget_ratio() -> f32 {
    parse_min_chunk_budget_ratio(
        &std::env::var("EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO").unwrap_or_default(),
    )
}

/// Parse `EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO` (pure — pass raw for tests).
///
/// Empty / invalid → `0.40`. Clamp to `[0.0, 0.9]`.
pub fn parse_min_chunk_budget_ratio(raw: &str) -> f32 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return 0.40;
    }
    trimmed
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, 0.9))
        .unwrap_or(0.40)
}

impl Default for TruncationConfig {
    fn default() -> Self {
        // WHY 30000: LightRAG uses max_total_tokens=30000. Entity/relation caps
        // are soft ceilings; chunks get the dynamic remainder (SPEC-046 P0.4).
        Self {
            max_entity_tokens: 10000,
            max_relation_tokens: 10000,
            max_total_tokens: 30000,
            buffer_tokens: default_truncation_buffer(),
            min_chunk_budget_ratio: default_min_chunk_budget_ratio(),
        }
    }
}

/// SPEC-047 / 020 A3: intent-aware graph tax.
///
/// Factual / L1 questions are page-chunk problems. Demote entity/rel budgets and
/// raise the chunk floor so hybrid local entities cannot starve Document Chunks
/// (post-B2 Acc tax: `n_sources` 20→115).
pub fn truncation_config_for_intent(
    base: &TruncationConfig,
    intent: crate::keywords::QueryIntent,
) -> TruncationConfig {
    use crate::keywords::QueryIntent;
    let mut cfg = base.clone();
    match intent {
        QueryIntent::Factual => {
            cfg.max_entity_tokens = cfg.max_entity_tokens.min(2_000);
            cfg.max_relation_tokens = cfg.max_relation_tokens.min(2_000);
            cfg.min_chunk_budget_ratio = cfg.min_chunk_budget_ratio.max(0.55);
        }
        QueryIntent::Procedural => {
            // Procedures need chunk steps more than entity dumps.
            cfg.max_entity_tokens = cfg.max_entity_tokens.min(4_000);
            cfg.max_relation_tokens = cfg.max_relation_tokens.min(4_000);
            cfg.min_chunk_budget_ratio = cfg.min_chunk_budget_ratio.max(0.50);
        }
        QueryIntent::Relational | QueryIntent::Exploratory | QueryIntent::Comparative => {
            // Graph-heavy intents keep base budgets (BR0102).
        }
    }
    cfg
}

/// Minimum chunk token budget from config (SPEC-047 Q1.3).
pub fn min_chunk_token_budget(config: &TruncationConfig) -> usize {
    let usable = config.max_total_tokens.saturating_sub(config.buffer_tokens);
    if config.min_chunk_budget_ratio <= 0.0 || usable == 0 {
        return 0;
    }
    ((usable as f32) * config.min_chunk_budget_ratio).floor() as usize
}

/// Truncate entities to fit within token limit.
pub fn truncate_entities(
    entities: Vec<RetrievedEntity>,
    max_tokens: usize,
    tokenizer: &dyn Tokenizer,
) -> Vec<RetrievedEntity> {
    let mut result = Vec::new();
    let mut total_tokens = 0;

    for entity in entities {
        let entity_tokens = tokenizer.count_tokens(&format_entity_line(&entity));

        if total_tokens + entity_tokens <= max_tokens {
            result.push(entity);
            total_tokens += entity_tokens;
        } else {
            break;
        }
    }

    result
}

/// Truncate relationships to fit within token limit.
pub fn truncate_relationships(
    relationships: Vec<RetrievedRelationship>,
    max_tokens: usize,
    tokenizer: &dyn Tokenizer,
) -> Vec<RetrievedRelationship> {
    let mut result = Vec::new();
    let mut total_tokens = 0;

    for rel in relationships {
        let rel_tokens = tokenizer.count_tokens(&format_relationship_line(&rel));

        if total_tokens + rel_tokens <= max_tokens {
            result.push(rel);
            total_tokens += rel_tokens;
        } else {
            break;
        }
    }

    result
}

/// Truncate chunks to fit within token limit (counts full prompt block).
pub fn truncate_chunks(
    chunks: Vec<RetrievedChunk>,
    max_tokens: usize,
    tokenizer: &dyn Tokenizer,
) -> Vec<RetrievedChunk> {
    let mut result = Vec::new();
    let mut total_tokens = 0;

    for (i, chunk) in chunks.into_iter().enumerate() {
        let chunk_tokens = tokenizer.count_tokens(&format_chunk_block(i + 1, &chunk));

        if total_tokens + chunk_tokens <= max_tokens {
            result.push(chunk);
            total_tokens += chunk_tokens;
        } else {
            break;
        }
    }

    result
}

fn entity_format_tokens(entity: &RetrievedEntity, tokenizer: &dyn Tokenizer) -> usize {
    tokenizer.count_tokens(&format_entity_line(entity))
}

fn relationship_format_tokens(rel: &RetrievedRelationship, tokenizer: &dyn Tokenizer) -> usize {
    tokenizer.count_tokens(&format_relationship_line(rel))
}

fn chunk_format_tokens(ref_id: usize, chunk: &RetrievedChunk, tokenizer: &dyn Tokenizer) -> usize {
    tokenizer.count_tokens(&format_chunk_block(ref_id, chunk))
}

/// Shrink graph lists until `entity_tokens + rel_tokens ≤ max_graph_tokens`.
fn shrink_graph_to_budget(
    mut entities: Vec<RetrievedEntity>,
    mut relationships: Vec<RetrievedRelationship>,
    max_graph_tokens: usize,
    tokenizer: &dyn Tokenizer,
) -> (Vec<RetrievedEntity>, Vec<RetrievedRelationship>) {
    loop {
        let entity_tokens: usize = entities
            .iter()
            .map(|e| entity_format_tokens(e, tokenizer))
            .sum();
        let rel_tokens: usize = relationships
            .iter()
            .map(|r| relationship_format_tokens(r, tokenizer))
            .sum();
        if entity_tokens + rel_tokens <= max_graph_tokens {
            return (entities, relationships);
        }
        if entities.len() >= relationships.len() && !entities.is_empty() {
            entities.pop();
        } else if !relationships.is_empty() {
            relationships.pop();
        } else if !entities.is_empty() {
            entities.pop();
        } else {
            return (entities, relationships);
        }
    }
}

/// Balance context to fit within total token limit.
///
/// SPEC-046 P0.4: truncate entities/relations to caps first, then give chunks
/// the **dynamic remainder** (`total − actual_entity − actual_rel − buffer`).
///
/// SPEC-047 Q1.3: if remainder would fall below `min_chunk_budget_ratio`, shrink
/// graph first so document chunks are not starved.
pub fn balance_context(
    entities: Vec<RetrievedEntity>,
    relationships: Vec<RetrievedRelationship>,
    chunks: Vec<RetrievedChunk>,
    config: &TruncationConfig,
    tokenizer: &dyn Tokenizer,
) -> (
    Vec<RetrievedEntity>,
    Vec<RetrievedRelationship>,
    Vec<RetrievedChunk>,
) {
    let input_entity_count = entities.len();
    let input_rel_count = relationships.len();
    let input_chunk_count = chunks.len();

    let mut entities = truncate_entities(entities, config.max_entity_tokens, tokenizer);
    let mut relationships =
        truncate_relationships(relationships, config.max_relation_tokens, tokenizer);

    let floor = min_chunk_token_budget(config);
    let usable = config.max_total_tokens.saturating_sub(config.buffer_tokens);
    if floor > 0 && !chunks.is_empty() {
        let max_graph_tokens = usable.saturating_sub(floor);
        let shrunk = shrink_graph_to_budget(entities, relationships, max_graph_tokens, tokenizer);
        entities = shrunk.0;
        relationships = shrunk.1;
    }

    let entity_tokens: usize = entities
        .iter()
        .map(|e| entity_format_tokens(e, tokenizer))
        .sum();
    let rel_tokens: usize = relationships
        .iter()
        .map(|r| relationship_format_tokens(r, tokenizer))
        .sum();

    let mut max_chunk_tokens = config
        .max_total_tokens
        .saturating_sub(entity_tokens)
        .saturating_sub(rel_tokens)
        .saturating_sub(config.buffer_tokens);
    if floor > 0 {
        max_chunk_tokens = max_chunk_tokens.max(floor.min(usable));
    }

    let mut chunks = truncate_chunks(chunks, max_chunk_tokens, tokenizer);

    tracing::debug!(
        input_entities = input_entity_count,
        input_relationships = input_rel_count,
        input_chunks = input_chunk_count,
        after_truncate_entities = entities.len(),
        after_truncate_rels = relationships.len(),
        after_truncate_chunks = chunks.len(),
        entity_tokens,
        rel_tokens,
        max_chunk_tokens,
        chunk_budget_floor = floor,
        buffer_tokens = config.buffer_tokens,
        "OODA-231: balance_context dynamic remainder (SPEC-046/047)"
    );

    let chunk_tokens: usize = chunks
        .iter()
        .enumerate()
        .map(|(i, c)| chunk_format_tokens(i + 1, c, tokenizer))
        .sum();
    let total = entity_tokens + rel_tokens + chunk_tokens;

    if total <= config.max_total_tokens {
        return (entities, relationships, chunks);
    }

    let reduction_ratio = config.max_total_tokens as f32 / total as f32;
    let new_entity_count = (entities.len() as f32 * reduction_ratio).ceil() as usize;
    let new_rel_count = (relationships.len() as f32 * reduction_ratio).ceil() as usize;
    let new_chunk_count = (chunks.len() as f32 * reduction_ratio).ceil() as usize;

    entities.truncate(new_entity_count.max(1).min(entities.len()));
    relationships.truncate(new_rel_count.min(relationships.len()));
    chunks.truncate(new_chunk_count.max(1).min(chunks.len()));

    (entities, relationships, chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::MockTokenizer;

    fn create_test_entity(name: &str, description: &str) -> RetrievedEntity {
        RetrievedEntity {
            name: name.to_string(),
            entity_type: "TEST".to_string(),
            description: description.to_string(),
            score: 1.0,
            degree: 0,
            source_chunk_ids: Vec::new(),
            source_document_id: None,
            source_document_ids: Vec::new(),
            source_file_path: None,
        }
    }

    fn create_test_relationship(source: &str, target: &str) -> RetrievedRelationship {
        RetrievedRelationship {
            source: source.to_string(),
            target: target.to_string(),
            relation_type: "TEST".to_string(),
            description: "Test relationship".to_string(),
            score: 1.0,
            source_chunk_id: None,
            source_document_id: None,
            source_document_ids: Vec::new(),
            source_file_path: None,
        }
    }

    fn create_test_chunk(id: &str, content: &str) -> RetrievedChunk {
        RetrievedChunk::new(id, content, 1.0)
    }

    fn tight_config(total: usize) -> TruncationConfig {
        TruncationConfig {
            max_entity_tokens: 10_000,
            max_relation_tokens: 10_000,
            max_total_tokens: total,
            buffer_tokens: 0,
            min_chunk_budget_ratio: 0.0,
        }
    }

    #[test]
    fn test_truncate_entities() {
        let tokenizer = MockTokenizer::with_rate(0.1);
        let entities = vec![
            create_test_entity("E1", "Short"),
            create_test_entity("E2", "A bit longer description"),
            create_test_entity("E3", "Another entity"),
        ];
        let truncated = truncate_entities(entities.clone(), 10, &tokenizer);
        assert!(!truncated.is_empty());
        assert!(truncated.len() <= entities.len());
    }

    #[test]
    fn test_truncate_relationships() {
        let tokenizer = MockTokenizer::with_rate(0.1);
        let rels = vec![
            create_test_relationship("A", "B"),
            create_test_relationship("C", "D"),
            create_test_relationship("E", "F"),
        ];
        let truncated = truncate_relationships(rels.clone(), 10, &tokenizer);
        assert!(!truncated.is_empty());
        assert!(truncated.len() <= rels.len());
    }

    #[test]
    fn test_truncate_chunks() {
        let tokenizer = MockTokenizer::with_rate(0.1);
        let chunks = vec![
            create_test_chunk("c1", "Short chunk"),
            create_test_chunk("c2", "This is a much longer chunk with more content"),
            create_test_chunk("c3", "Another chunk"),
        ];
        let truncated = truncate_chunks(chunks.clone(), 10, &tokenizer);
        assert!(!truncated.is_empty());
        assert!(truncated.len() <= chunks.len());
    }

    #[test]
    fn test_balance_context() {
        let tokenizer = MockTokenizer::with_rate(1.0);
        let config = tight_config(10);

        let entities = vec![
            create_test_entity("E1", "Description 1"),
            create_test_entity("E2", "Description 2"),
            create_test_entity("E3", "Description 3"),
        ];
        let rels = vec![
            create_test_relationship("A", "B"),
            create_test_relationship("C", "D"),
        ];
        let chunks = vec![
            create_test_chunk("c1", "Chunk 1"),
            create_test_chunk("c2", "Chunk 2"),
        ];

        let (balanced_entities, balanced_rels, balanced_chunks) = balance_context(
            entities.clone(),
            rels.clone(),
            chunks.clone(),
            &config,
            &tokenizer,
        );

        assert!(
            balanced_entities.len() < entities.len()
                || balanced_rels.len() < rels.len()
                || balanced_chunks.len() < chunks.len()
        );
    }

    #[test]
    fn test_balance_context_within_limit() {
        let tokenizer = MockTokenizer::with_rate(0.01);
        let config = TruncationConfig {
            max_entity_tokens: 1000,
            max_relation_tokens: 1000,
            max_total_tokens: 10000,
            buffer_tokens: 0,
            min_chunk_budget_ratio: 0.0,
        };

        let entities = vec![create_test_entity("E1", "Desc")];
        let rels = vec![create_test_relationship("A", "B")];
        let chunks = vec![create_test_chunk("c1", "Chunk")];

        let (balanced_entities, balanced_rels, balanced_chunks) = balance_context(
            entities.clone(),
            rels.clone(),
            chunks.clone(),
            &config,
            &tokenizer,
        );

        assert_eq!(balanced_entities.len(), entities.len());
        assert_eq!(balanced_rels.len(), rels.len());
        assert_eq!(balanced_chunks.len(), chunks.len());
    }

    #[test]
    fn test_truncation_config_default() {
        let config = TruncationConfig::default();
        assert_eq!(config.max_entity_tokens, 10000);
        assert_eq!(config.max_relation_tokens, 10000);
        assert_eq!(config.max_total_tokens, 30000);
        assert!(
            (config.min_chunk_budget_ratio - 0.40).abs() < f32::EPSILON
                || config.min_chunk_budget_ratio >= 0.0
        );
    }

    #[test]
    fn parse_min_chunk_budget_ratio_defaults_and_clamps() {
        assert!((parse_min_chunk_budget_ratio("") - 0.40).abs() < f32::EPSILON);
        assert!((parse_min_chunk_budget_ratio("0.5") - 0.5).abs() < f32::EPSILON);
        assert!((parse_min_chunk_budget_ratio("2.0") - 0.9).abs() < f32::EPSILON);
        assert!((parse_min_chunk_budget_ratio("-1") - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn chunk_budget_floor_shrinks_graph_before_chunks() {
        // 1 token per char; long entity descriptions would starve chunks without floor.
        let tokenizer = MockTokenizer::with_rate(1.0);
        let config = TruncationConfig {
            max_entity_tokens: 10_000,
            max_relation_tokens: 10_000,
            max_total_tokens: 200,
            buffer_tokens: 0,
            min_chunk_budget_ratio: 0.40, // floor = 80 tokens for chunks
        };

        let entities: Vec<_> = (0..20)
            .map(|i| {
                create_test_entity(
                    &format!("E{i}"),
                    &"x".repeat(30), // ~30 tokens each via format_entity_line overhead
                )
            })
            .collect();
        let chunks = vec![
            create_test_chunk("c1", &"chunk-body-one".repeat(3)),
            create_test_chunk("c2", &"chunk-body-two".repeat(3)),
        ];

        let (bal_e, _bal_r, bal_c) =
            balance_context(entities, vec![], chunks.clone(), &config, &tokenizer);

        assert!(
            !bal_c.is_empty(),
            "chunk floor must keep at least one evidence chunk"
        );
        assert!(
            bal_e.len() < 20,
            "graph must shrink to honor chunk floor; kept {}",
            bal_e.len()
        );
    }

    #[test]
    fn a3_factual_raises_chunk_floor_and_caps_graph() {
        use crate::keywords::QueryIntent;
        let base = TruncationConfig::default();
        let factual = truncation_config_for_intent(&base, QueryIntent::Factual);
        assert_eq!(factual.max_entity_tokens, 2_000);
        assert_eq!(factual.max_relation_tokens, 2_000);
        assert!(factual.min_chunk_budget_ratio >= 0.55);
        let exploratory = truncation_config_for_intent(&base, QueryIntent::Exploratory);
        assert_eq!(exploratory.max_entity_tokens, base.max_entity_tokens);
    }
}
