//! LightRAG-style Hybrid merge (SPEC-024 Phase 2 / FEAT0104).
//!
//! Combines Local, Global, and Naive retrieval arms with:
//! - **Round-robin interleave** (default): local → global → naive (EQ Acc-era).
//!   LightRAG `_merge_all_chunks` is naive → entity → relation; set
//!   `EDGEQUAKE_RR_ORDER=naive_first` for that order (076 L1.5 peer follow-up).
//! - **RRF** (optional): set `EDGEQUAKE_HYBRID_FUSION=rrf` for reciprocal rank fusion.
//!
//! Entities and relationships are unioned across local+global (scores are not
//! cross-arm comparable). Chunks are truncated to `max_chunks`.
//!
//! SPEC-047 / 020 Acc recovery: arms that return **no chunks** do not inject
//! entity/rel payloads (fail-open prune — empty graph must not pollute Gen).

use std::collections::{HashMap, HashSet};

use crate::context::{QueryContext, RetrievedChunk, RetrievedRelationship};
use crate::fusion;

/// How Hybrid mode merges chunk lists from the three retrieval arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridFusionMode {
    /// LightRAG default: round-robin interleave with KG-first priority.
    RoundRobin,
    /// RRF across ranked lists (equal weights).
    Rrf,
}

/// Read Hybrid chunk fusion mode (default: round-robin / LightRAG).
pub fn hybrid_fusion_mode_from_env() -> HybridFusionMode {
    match std::env::var("EDGEQUAKE_HYBRID_FUSION")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "rrf" => HybridFusionMode::Rrf,
        _ => HybridFusionMode::RoundRobin,
    }
}

/// Operator-visible label for health / dashboards.
pub fn hybrid_fusion_mode_label(mode: HybridFusionMode) -> &'static str {
    match mode {
        HybridFusionMode::RoundRobin => "round_robin",
        HybridFusionMode::Rrf => "rrf",
    }
}

/// Drop entity/relationship payloads from a graph arm that retrieved **no chunks**.
///
/// Law (020 post-B2): scheduling local/global is honest; injecting orphan KG text
/// when the arm found no page-linked chunks is context pollution (Acc tax on
/// unanswerable + factual). Fail-open: keep the arm's chunk list (empty) and clear graph.
pub fn prune_empty_arm_graph(mut ctx: QueryContext) -> QueryContext {
    if ctx.chunks.is_empty() && (!ctx.entities.is_empty() || !ctx.relationships.is_empty()) {
        tracing::debug!(
            entities = ctx.entities.len(),
            relationships = ctx.relationships.len(),
            "020: prune empty-arm graph payloads (no chunks)"
        );
        ctx.entities.clear();
        ctx.relationships.clear();
    }
    ctx
}

/// Merge three retrieval contexts into one Hybrid context.
pub fn merge_hybrid_contexts(
    local: QueryContext,
    global: QueryContext,
    naive: QueryContext,
    max_chunks: usize,
) -> QueryContext {
    let _stage = edgequake_observability::enter_pipeline_stage("query.fuse");
    let fusion = crate::hybrid_merge::hybrid_fusion_mode_label(hybrid_fusion_mode_from_env());
    edgequake_observability::record_observation_meta("fusion", fusion);
    let local = prune_empty_arm_graph(local);
    let global = prune_empty_arm_graph(global);
    // Naive is chunk-only by construction; no entity prune needed.
    let mut merged = QueryContext::new();

    let chunks = merge_hybrid_chunks(
        &local.chunks,
        &global.chunks,
        &naive.chunks,
        max_chunks,
        hybrid_fusion_mode_from_env(),
    );
    for chunk in chunks {
        merged.add_chunk(chunk);
    }

    merge_hybrid_entities_and_relationships(&mut merged, &local, &global);

    merged
}

fn merge_hybrid_chunks(
    local: &[RetrievedChunk],
    global: &[RetrievedChunk],
    naive: &[RetrievedChunk],
    max_chunks: usize,
    mode: HybridFusionMode,
) -> Vec<RetrievedChunk> {
    if max_chunks == 0 {
        return Vec::new();
    }

    match mode {
        HybridFusionMode::Rrf => {
            let mut lookup: HashMap<String, RetrievedChunk> = HashMap::new();
            for chunk in local.iter().chain(global.iter()).chain(naive.iter()) {
                lookup
                    .entry(chunk.id.clone())
                    .or_insert_with(|| chunk.clone());
            }
            let lists = [
                local.iter().map(|c| c.id.clone()).collect(),
                global.iter().map(|c| c.id.clone()).collect(),
                naive.iter().map(|c| c.id.clone()).collect(),
            ];
            fusion::chunks_from_rrf_ranking(
                &fusion::reciprocal_rank_fusion(&lists, &[1.0, 1.0, 1.0], fusion::RRF_K),
                &lookup,
                max_chunks,
            )
        }
        HybridFusionMode::RoundRobin => round_robin_merge_chunks(local, global, naive, max_chunks),
    }
}

/// Round-robin arm order for Mix/Hybrid chunk merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoundRobinArmOrder {
    /// EQ Acc-era: local → global → naive.
    #[default]
    LocalFirst,
    /// LightRAG `_merge_all_chunks`: naive → entity(local) → relation(global).
    NaiveFirst,
}

/// Parse `EDGEQUAKE_RR_ORDER` (`local_first` default · `naive_first` = LR law).
pub fn rr_arm_order_from_env() -> RoundRobinArmOrder {
    match std::env::var("EDGEQUAKE_RR_ORDER")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "naive_first" | "naive-first" | "lightrag" | "vector_first" => {
            RoundRobinArmOrder::NaiveFirst
        }
        _ => RoundRobinArmOrder::LocalFirst,
    }
}

/// Round-robin merge with dedup by chunk id.
///
/// Default order local→global→naive. Pass [`RoundRobinArmOrder::NaiveFirst`] (or
/// set `EDGEQUAKE_RR_ORDER=naive_first`) for LightRAG naive→entity→relation.
pub fn round_robin_merge_chunks(
    local: &[RetrievedChunk],
    global: &[RetrievedChunk],
    naive: &[RetrievedChunk],
    max_chunks: usize,
) -> Vec<RetrievedChunk> {
    round_robin_merge_chunks_ordered(local, global, naive, max_chunks, rr_arm_order_from_env())
}

/// Pure round-robin merge with explicit arm order (testable without env).
pub fn round_robin_merge_chunks_ordered(
    local: &[RetrievedChunk],
    global: &[RetrievedChunk],
    naive: &[RetrievedChunk],
    max_chunks: usize,
    order: RoundRobinArmOrder,
) -> Vec<RetrievedChunk> {
    let mut out = Vec::with_capacity(max_chunks.min(local.len() + global.len() + naive.len()));
    let mut seen = HashSet::new();
    let max_len = local.len().max(global.len()).max(naive.len());
    let sources: [&[RetrievedChunk]; 3] = match order {
        RoundRobinArmOrder::LocalFirst => [local, global, naive],
        RoundRobinArmOrder::NaiveFirst => [naive, local, global],
    };

    'outer: for i in 0..max_len {
        for source in sources {
            if let Some(c) = source.get(i) {
                if seen.insert(c.id.clone()) {
                    out.push(c.clone());
                    if out.len() >= max_chunks {
                        break 'outer;
                    }
                }
            }
        }
    }
    out
}

fn merge_hybrid_entities_and_relationships(
    merged: &mut QueryContext,
    local: &QueryContext,
    global: &QueryContext,
) {
    let mut seen_entities = HashSet::new();
    let max_entity_len = local.entities.len().max(global.entities.len());
    for i in 0..max_entity_len {
        for source in [&local.entities, &global.entities] {
            if let Some(e) = source.get(i) {
                if seen_entities.insert(e.name.clone()) {
                    merged.add_entity(e.clone());
                }
            }
        }
    }

    let mut seen_rels = HashSet::new();
    for rel in local
        .relationships
        .iter()
        .chain(global.relationships.iter())
    {
        if seen_rels.insert(rel_key(rel)) {
            merged.add_relationship(rel.clone());
        }
    }
}

fn rel_key(rel: &RetrievedRelationship) -> String {
    format!("{}-{}-{}", rel.source, rel.relation_type, rel.target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RetrievedEntity;

    fn chunk(id: &str, score: f32) -> RetrievedChunk {
        RetrievedChunk::new(id, id, score)
    }

    #[test]
    fn round_robin_local_first_and_dedup() {
        let local = vec![chunk("shared", 0.9), chunk("local_only", 0.8)];
        let global = vec![chunk("shared", 0.85), chunk("global_only", 0.7)];
        let naive = vec![chunk("naive_only", 0.6)];

        let merged = round_robin_merge_chunks_ordered(
            &local,
            &global,
            &naive,
            10,
            RoundRobinArmOrder::LocalFirst,
        );
        let ids: Vec<_> = merged.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["shared", "naive_only", "local_only", "global_only"]
        );
    }

    #[test]
    fn round_robin_naive_first_matches_lightrag_merge_order() {
        let local = vec![chunk("shared", 0.9), chunk("local_only", 0.8)];
        let global = vec![chunk("shared", 0.85), chunk("global_only", 0.7)];
        let naive = vec![chunk("naive_only", 0.6)];

        let merged = round_robin_merge_chunks_ordered(
            &local,
            &global,
            &naive,
            10,
            RoundRobinArmOrder::NaiveFirst,
        );
        let ids: Vec<_> = merged.iter().map(|c| c.id.as_str()).collect();
        // LR: vector(naive) → entity(local) → relation(global) per index.
        assert_eq!(
            ids,
            vec!["naive_only", "shared", "local_only", "global_only"]
        );
    }

    #[test]
    fn round_robin_respects_max_chunks() {
        let local: Vec<_> = (0..5).map(|i| chunk(&format!("l{i}"), 1.0)).collect();
        let global: Vec<_> = (0..5).map(|i| chunk(&format!("g{i}"), 1.0)).collect();
        let naive: Vec<_> = (0..5).map(|i| chunk(&format!("n{i}"), 1.0)).collect();
        let merged = round_robin_merge_chunks_ordered(
            &local,
            &global,
            &naive,
            3,
            RoundRobinArmOrder::LocalFirst,
        );
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn entities_union_dedup() {
        let mut local = QueryContext::new();
        local.add_entity(RetrievedEntity::new("A", "X", "a"));
        local.add_chunk(chunk("l1", 0.9)); // non-empty arm keeps entities
        let mut global = QueryContext::new();
        global.add_entity(RetrievedEntity::new("A", "X", "a"));
        global.add_entity(RetrievedEntity::new("B", "X", "b"));
        global.add_chunk(chunk("g1", 0.8));

        let merged = merge_hybrid_contexts(local, global, QueryContext::new(), 20);
        assert_eq!(merged.entities.len(), 2);
    }

    #[test]
    fn empty_arm_graph_pruned_before_merge() {
        let mut local = QueryContext::new();
        local.add_entity(RetrievedEntity::new("ORPHAN", "X", "should drop"));
        // no chunks
        let mut naive = QueryContext::new();
        naive.add_chunk(chunk("n1", 0.9));

        let merged = merge_hybrid_contexts(local, QueryContext::new(), naive, 20);
        assert!(
            merged.entities.is_empty(),
            "020: empty local must not inject entities; got {:?}",
            merged.entities.iter().map(|e| &e.name).collect::<Vec<_>>()
        );
        assert_eq!(merged.chunks.len(), 1);
        assert_eq!(merged.chunks[0].id, "n1");
    }

    #[test]
    fn prune_empty_arm_graph_clears_orphans() {
        let mut ctx = QueryContext::new();
        ctx.add_entity(RetrievedEntity::new("E", "T", "d"));
        ctx.add_relationship(RetrievedRelationship::new("A", "B", "REL"));
        let pruned = prune_empty_arm_graph(ctx);
        assert!(pruned.entities.is_empty());
        assert!(pruned.relationships.is_empty());
    }
}
