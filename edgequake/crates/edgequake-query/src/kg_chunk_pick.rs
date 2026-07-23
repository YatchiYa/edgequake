//! KG → chunk selection strategies (SPEC-046 P0.3 / LightRAG parity).
//!
//! LightRAG `KG_CHUNK_PICK_METHOD`:
//! - **Vector** (default): rank candidate chunk IDs by query embedding similarity
//! - **Weight**: prefer chunks that appear as sources for more entities/relations

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::context::QueryContext;

thread_local! {
    /// Mix-only: skip per-arm KG→chunk pick while arms run (078 R3 post_truncate).
    static SKIP_ARM_KG_CHUNKS: Cell<bool> = const { Cell::new(false) };
}

/// When to run KG→chunk VECTOR pick relative to entity/relation token truncate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KgChunkPickTiming {
    /// Today's Mix: each arm picks chunks, then fuse, truncate later (default).
    #[default]
    PerArm,
    /// LightRAG `_build_query_context`: truncate E/R → one VECTOR pick → merge naive.
    PostTruncate,
}

impl KgChunkPickTiming {
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_KG_CHUNK_PICK_TIMING")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "post_truncate" | "post-truncate" | "after_truncate" | "lr" => Self::PostTruncate,
            _ => Self::PerArm,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PerArm => "per_arm",
            Self::PostTruncate => "post_truncate",
        }
    }

    pub fn is_post_truncate(self) -> bool {
        matches!(self, Self::PostTruncate)
    }
}

/// RAII: Mix arms skip KG chunk fetch; Mix re-picks after E/R truncate.
pub struct SkipArmKgChunksGuard;

impl SkipArmKgChunksGuard {
    pub fn enter() -> Self {
        SKIP_ARM_KG_CHUNKS.with(|c| c.set(true));
        Self
    }
}

impl Drop for SkipArmKgChunksGuard {
    fn drop(&mut self) {
        SKIP_ARM_KG_CHUNKS.with(|c| c.set(false));
    }
}

pub fn arm_kg_chunks_skipped() -> bool {
    SKIP_ARM_KG_CHUNKS.with(|c| c.get())
}

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

fn env_flag_on(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// 024 Q1: LightRAG Step-3 — sort each entity's chunks by global citation frequency
/// before applying `related_chunk_number` take.
pub fn occurrence_sort_enabled() -> bool {
    env_flag_on("EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT")
}

/// 024 Q2: LightRAG VECTOR path — uncapped pool, then take
/// `related_chunk_number * n_entities / 2`.
pub fn lr_vector_budget_enabled() -> bool {
    env_flag_on("EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET")
}

/// LightRAG VECTOR budget: `related_chunk_number * n_entities / 2` (integer).
pub fn lr_vector_chunk_budget(related_chunk_number: usize, n_entities: usize) -> usize {
    if related_chunk_number == 0 || n_entities == 0 {
        return 0;
    }
    (related_chunk_number * n_entities) / 2
}

/// Global citation counts for entity/relation source chunks (LightRAG occurrence).
pub fn chunk_citation_counts(context: &QueryContext) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entity in &context.entities {
        for chunk_id in &entity.source_chunk_ids {
            *counts.entry(chunk_id.clone()).or_insert(0) += 1;
        }
    }
    for rel in &context.relationships {
        for chunk_id in rel.all_source_chunk_ids() {
            *counts.entry(chunk_id).or_insert(0) += 1;
        }
    }
    counts
}

/// Collect candidate chunk IDs from a KG context, optionally capped per source.
///
/// `related_chunk_number` mirrors LightRAG: max chunks contributed per entity
/// or relationship. `0` means unlimited per source.
///
/// When `EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1`, each entity's chunk list is
/// sorted by global citation frequency (desc) before the take.
///
/// When `allowed_document_ids` is `Some`, only chunk ids whose derived document
/// intersects the allowed set are returned (SPEC-047 / 021 L-A3).
pub fn collect_kg_chunk_ids(context: &QueryContext, related_chunk_number: usize) -> Vec<String> {
    collect_kg_chunk_ids_scoped(context, related_chunk_number, None)
}

/// Like [`collect_kg_chunk_ids`] with optional document scope (021 L-A3).
pub fn collect_kg_chunk_ids_scoped(
    context: &QueryContext,
    related_chunk_number: usize,
    allowed_document_ids: Option<&[String]>,
) -> Vec<String> {
    let sort_by_occurrence = occurrence_sort_enabled();
    let counts = if sort_by_occurrence {
        chunk_citation_counts(context)
    } else {
        HashMap::new()
    };
    let mut ids = HashSet::new();

    for entity in &context.entities {
        let mut chunk_ids = entity.source_chunk_ids.clone();
        if sort_by_occurrence {
            chunk_ids.sort_by(|a, b| {
                let ca = counts.get(a).copied().unwrap_or(0);
                let cb = counts.get(b).copied().unwrap_or(0);
                cb.cmp(&ca).then_with(|| a.cmp(b))
            });
        }
        let take = if related_chunk_number == 0 {
            chunk_ids.len()
        } else {
            related_chunk_number.min(chunk_ids.len())
        };
        for chunk_id in chunk_ids.into_iter().take(take) {
            ids.insert(chunk_id);
        }
    }

    for rel in &context.relationships {
        let _ = related_chunk_number;
        // 052: LightRAG admits every relation source_id part into the chunk pool.
        for chunk_id in rel.all_source_chunk_ids() {
            ids.insert(chunk_id);
        }
    }

    let collected: Vec<String> = ids.into_iter().collect();
    crate::lineage_scope::filter_chunk_ids_by_allowed_docs(&collected, allowed_document_ids)
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
        // 052 / 081 F2: count every multi-chunk relation source_id (not singular only).
        for chunk_id in rel.all_source_chunk_ids() {
            *weights.entry(chunk_id).or_insert(0) += 1;
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

/// Full dual-node: PPR over entity–entity edges ∪ entity–chunk mentions (EQ-046-17).
///
/// When `entity_edges` is empty, falls back to lite [`pick_chunks_by_entity_ppr`].
/// Seeds are entity names with positive score (else all entities with chunk links).
pub fn pick_chunks_by_bipartite_ppr(
    context: &QueryContext,
    entity_edges: &[edgequake_storage::traits::GraphEdge],
    max_chunks: usize,
) -> Vec<String> {
    use crate::graph_ppr::{rank_chunks_bipartite_ppr, PprConfig};

    let mut links: Vec<(String, String)> = Vec::new();
    let mut scored_entities: Vec<(String, f32)> = Vec::new();
    for entity in &context.entities {
        for chunk_id in &entity.source_chunk_ids {
            links.push((entity.name.clone(), chunk_id.clone()));
        }
        if !entity.source_chunk_ids.is_empty() {
            scored_entities.push((entity.name.clone(), entity.score.max(0.0)));
        }
    }
    for rel in &context.relationships {
        if let Some(chunk_id) = &rel.source_chunk_id {
            links.push((rel.source.clone(), chunk_id.clone()));
            links.push((rel.target.clone(), chunk_id.clone()));
        }
    }
    if links.is_empty() {
        return pick_chunks_by_weight(context, max_chunks);
    }
    // Seed only entities with score ≥ 50% of the max (keeps PPR personalized;
    // seeding every positive score equalizes mass and defeats dual-node ranking).
    let max_score = scored_entities
        .iter()
        .map(|(_, s)| *s)
        .fold(0.0f32, f32::max);
    let mut seeds: Vec<String> = if max_score > 0.0 {
        scored_entities
            .iter()
            .filter(|(_, s)| *s >= max_score * 0.5)
            .map(|(n, _)| n.clone())
            .collect()
    } else {
        scored_entities.into_iter().map(|(n, _)| n).collect()
    };
    if seeds.is_empty() {
        seeds = context
            .entities
            .iter()
            .filter(|e| !e.source_chunk_ids.is_empty())
            .map(|e| e.name.clone())
            .collect();
    }
    if seeds.is_empty() || entity_edges.is_empty() {
        // No structural entity edges → lite projection is the honest fallback.
        return pick_chunks_by_entity_ppr(context, max_chunks);
    }
    let ranked = rank_chunks_bipartite_ppr(
        entity_edges,
        &links,
        &seeds,
        &PprConfig::default(),
        max_chunks,
    );
    if ranked.is_empty() {
        return pick_chunks_by_entity_ppr(context, max_chunks);
    }
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{QueryContext, RetrievedEntity, RetrievedRelationship};
    use std::collections::HashMap;

    #[test]
    fn scoped_collect_drops_foreign_chunk_ids() {
        let mut ctx = QueryContext::new();
        let mut e = RetrievedEntity::new("A", "PERSON", "desc");
        e.source_chunk_ids = vec!["doc-a-chunk-0".into(), "doc-z-chunk-9".into()];
        ctx.add_entity(e);
        let allowed = vec!["doc-a".to_string()];
        let ids = collect_kg_chunk_ids_scoped(&ctx, 0, Some(&allowed));
        assert_eq!(ids, vec!["doc-a-chunk-0".to_string()]);
    }

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
    fn collect_admits_all_relation_source_chunk_ids() {
        // 052: LightRAG multi-part relation source_id → Mix pool.
        let mut ctx = QueryContext::new();
        ctx.add_relationship(
            RetrievedRelationship::new("A", "B", "KNOWS")
                .with_source_chunk_ids(vec!["rel-a".into(), "rel-b".into()]),
        );
        let ids = collect_kg_chunk_ids(&ctx, 5);
        assert!(ids.contains(&"rel-a".to_string()), "{ids:?}");
        assert!(ids.contains(&"rel-b".to_string()), "{ids:?}");
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn occurrence_sort_prefers_high_citation_before_take() {
        // Without sort: take(1) keeps storage-first "cold".
        // With sort: "hot" cited by two entities wins.
        std::env::remove_var("EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT");
        let mut ctx = QueryContext::new();
        let mut e1 = RetrievedEntity::new("A", "PERSON", "d");
        e1.source_chunk_ids = vec!["cold".into(), "hot".into()];
        let mut e2 = RetrievedEntity::new("B", "PERSON", "d");
        e2.source_chunk_ids = vec!["hot".into()];
        ctx.add_entity(e1);
        ctx.add_entity(e2);
        let unsorted = collect_kg_chunk_ids(&ctx, 1);
        assert!(unsorted.contains(&"cold".to_string()) || unsorted.contains(&"hot".to_string()));

        std::env::set_var("EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT", "1");
        let sorted = collect_kg_chunk_ids(&ctx, 1);
        std::env::remove_var("EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT");
        assert_eq!(sorted, vec!["hot".to_string()]);
    }

    #[test]
    fn lr_vector_chunk_budget_matches_lightrag_formula() {
        assert_eq!(lr_vector_chunk_budget(5, 10), 25);
        assert_eq!(lr_vector_chunk_budget(5, 1), 2);
        assert_eq!(lr_vector_chunk_budget(0, 10), 0);
    }

    #[test]
    fn kg_chunk_pick_timing_from_env() {
        std::env::remove_var("EDGEQUAKE_KG_CHUNK_PICK_TIMING");
        assert_eq!(KgChunkPickTiming::from_env(), KgChunkPickTiming::PerArm);
        std::env::set_var("EDGEQUAKE_KG_CHUNK_PICK_TIMING", "post_truncate");
        assert!(KgChunkPickTiming::from_env().is_post_truncate());
        std::env::remove_var("EDGEQUAKE_KG_CHUNK_PICK_TIMING");
    }

    #[test]
    fn skip_arm_kg_chunks_guard_toggles() {
        assert!(!arm_kg_chunks_skipped());
        {
            let _g = SkipArmKgChunksGuard::enter();
            assert!(arm_kg_chunks_skipped());
        }
        assert!(!arm_kg_chunks_skipped());
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
    fn weight_pick_counts_all_relation_source_chunk_ids() {
        // 081 F2 / 052: plural relation lineage must weight every chunk id.
        let mut ctx = QueryContext::new();
        ctx.add_relationship(
            RetrievedRelationship::new("A", "B", "KNOWS").with_source_chunk_ids(vec![
                "rel-a".into(),
                "rel-b".into(),
                "rel-b".into(),
            ]),
        );
        let picked = pick_chunks_by_weight(&ctx, 2);
        assert!(picked.contains(&"rel-a".to_string()), "{picked:?}");
        assert!(picked.contains(&"rel-b".to_string()), "{picked:?}");
        // rel-b appears twice in lineage → higher weight → first.
        assert_eq!(picked[0], "rel-b");
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

    #[test]
    fn pick_chunks_by_bipartite_ppr_prefers_seed_chunk() {
        use edgequake_storage::traits::GraphEdge;
        let mut ctx = QueryContext::new();
        let mut seed = RetrievedEntity::new("SEED", "CONCEPT", "d");
        seed.score = 1.0;
        seed.source_chunk_ids = vec!["chunk-hot".into()];
        let mut n1 = RetrievedEntity::new("N1", "CONCEPT", "d");
        n1.score = 0.2;
        n1.source_chunk_ids = vec!["chunk-cold".into()];
        ctx.add_entity(seed);
        ctx.add_entity(n1);
        let edges = vec![GraphEdge {
            source: "SEED".into(),
            target: "N1".into(),
            properties: HashMap::new(),
        }];
        let ranked = pick_chunks_by_bipartite_ppr(&ctx, &edges, 2);
        assert_eq!(ranked.first().map(String::as_str), Some("chunk-hot"));
    }

    #[test]
    fn bipartite_falls_back_to_lite_without_entity_edges() {
        let mut ctx = QueryContext::new();
        let mut hot = RetrievedEntity::new("HOT", "CONCEPT", "d");
        hot.score = 1.0;
        hot.source_chunk_ids = vec!["chunk-hot".into()];
        ctx.add_entity(hot);
        let ranked = pick_chunks_by_bipartite_ppr(&ctx, &[], 1);
        assert_eq!(ranked, vec!["chunk-hot".to_string()]);
    }
}
