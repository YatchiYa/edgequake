//! KG → chunk selection strategies (SPEC-046 P0.3 / LightRAG parity).
//!
//! LightRAG `KG_CHUNK_PICK_METHOD`:
//! - **Vector** (default): rank candidate chunk IDs by query embedding similarity
//! - **Weight**: prefer chunks that appear as sources for more entities/relations

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::context::QueryContext;

/// How to pick text chunks linked from KG entities/relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KgChunkPickMethod {
    /// Cosine / vector score over candidate chunk IDs (current default).
    #[default]
    Vector,
    /// Weighted polling by how often a chunk is cited as a source.
    Weight,
}

impl KgChunkPickMethod {
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_KG_CHUNK_PICK")
            .or_else(|_| std::env::var("KG_CHUNK_PICK_METHOD"))
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "weight" | "weighted" | "polling" => Self::Weight,
            _ => Self::Vector,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vector => "vector",
            Self::Weight => "weight",
        }
    }
}

/// Collect candidate chunk IDs from a KG context, optionally capped per source.
///
/// `related_chunk_number` mirrors LightRAG: max chunks contributed per entity
/// or relationship. `0` means unlimited per source.
pub fn collect_kg_chunk_ids(
    context: &QueryContext,
    related_chunk_number: usize,
) -> Vec<String> {
    let mut ids = HashSet::new();

    for entity in &context.entities {
        let take = if related_chunk_number == 0 {
            entity.source_chunk_ids.len()
        } else {
            related_chunk_number.min(entity.source_chunk_ids.len())
        };
        for chunk_id in entity.source_chunk_ids.iter().take(take) {
            ids.insert(chunk_id.clone());
        }
    }

    for rel in &context.relationships {
        if let Some(chunk_id) = &rel.source_chunk_id {
            // Relations typically have a single source chunk; still respect cap
            // by only inserting when under global uniqueness (set handles that).
            let _ = related_chunk_number; // per-rel cap is 0/1 naturally
            ids.insert(chunk_id.clone());
        }
    }

    ids.into_iter().collect()
}

/// Rank chunk IDs by citation weight (entity/relation source frequency).
///
/// Returns IDs sorted by weight descending, truncated to `max_chunks`.
pub fn pick_chunks_by_weight(context: &QueryContext, max_chunks: usize) -> Vec<String> {
    let mut weights: HashMap<String, usize> = HashMap::new();

    for entity in &context.entities {
        for chunk_id in &entity.source_chunk_ids {
            *weights.entry(chunk_id.clone()).or_insert(0) += 1;
        }
    }
    for rel in &context.relationships {
        if let Some(chunk_id) = &rel.source_chunk_id {
            *weights.entry(chunk_id.clone()).or_insert(0) += 1;
        }
    }

    let mut ranked: Vec<(String, usize)> = weights.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(max_chunks)
        .map(|(id, _)| id)
        .collect()
}

/// Dual-node lite: map entity retrieval scores onto `source_chunk_ids` (SPEC-046 EQ-046-07).
///
/// Uses property lineage (no separate chunk graph nodes). Prefer when
/// `GraphWalkMode::Ppr` so PPR mass on entities influences chunk order.
pub fn pick_chunks_by_entity_ppr(context: &QueryContext, max_chunks: usize) -> Vec<String> {
    use crate::graph_ppr::chunk_scores_from_entity_ppr;

    let mut entity_to_chunks: HashMap<String, Vec<String>> = HashMap::new();
    let mut entity_scores: HashMap<String, f32> = HashMap::new();
    for entity in &context.entities {
        if entity.source_chunk_ids.is_empty() {
            continue;
        }
        entity_to_chunks.insert(entity.name.clone(), entity.source_chunk_ids.clone());
        entity_scores.insert(entity.name.clone(), entity.score.max(0.0));
    }
    if entity_to_chunks.is_empty() {
        return pick_chunks_by_weight(context, max_chunks);
    }
    let chunk_scores = chunk_scores_from_entity_ppr(&entity_to_chunks, &entity_scores);
    let mut ranked: Vec<(String, f32)> = chunk_scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked
        .into_iter()
        .take(max_chunks)
        .map(|(id, _)| id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{QueryContext, RetrievedEntity, RetrievedRelationship};

    #[test]
    fn related_chunk_number_caps_per_entity() {
        let mut ctx = QueryContext::new();
        let mut e = RetrievedEntity::new("A", "PERSON", "desc");
        e.source_chunk_ids = vec!["c1".into(), "c2".into(), "c3".into()];
        ctx.add_entity(e);
        let ids = collect_kg_chunk_ids(&ctx, 2);
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn weight_pick_prefers_frequent_chunks() {
        let mut ctx = QueryContext::new();
        let mut e1 = RetrievedEntity::new("A", "PERSON", "d");
        e1.source_chunk_ids = vec!["hot".into(), "cold".into()];
        let mut e2 = RetrievedEntity::new("B", "PERSON", "d");
        e2.source_chunk_ids = vec!["hot".into()];
        ctx.add_entity(e1);
        ctx.add_entity(e2);
        ctx.add_relationship(
            RetrievedRelationship::new("A", "B", "KNOWS").with_source_chunk_id("hot"),
        );
        let picked = pick_chunks_by_weight(&ctx, 2);
        assert_eq!(picked[0], "hot");
    }

    #[test]
    fn pick_chunks_by_entity_ppr_orders_by_entity_score() {
        let mut ctx = QueryContext::new();
        let mut hot = RetrievedEntity::new("HOT", "CONCEPT", "d");
        hot.score = 1.0;
        hot.source_chunk_ids = vec!["chunk-hot".into()];
        let mut cold = RetrievedEntity::new("COLD", "CONCEPT", "d");
        cold.score = 0.1;
        cold.source_chunk_ids = vec!["chunk-cold".into()];
        ctx.add_entity(hot);
        ctx.add_entity(cold);
        let ranked = pick_chunks_by_entity_ppr(&ctx, 2);
        assert_eq!(ranked.first().map(String::as_str), Some("chunk-hot"));
    }
}
