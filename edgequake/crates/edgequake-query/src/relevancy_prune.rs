//! Post-fusion relevancy prune (SPEC-001 Phase 1 / 010-lens-retrieval-noise).
//!
//! Env-gated: when enabled, keep top-m score-ranked chunks and soft-prune
//! entities/relationships that do not touch kept chunk lineage.
//! Fail-open: never empty a non-empty chunk list.
//!
//! Score modes (`EDGEQUAKE_MIX_RELEVANCY_SCORE`):
//! - `rrf` (default): use existing retrieval/RRF scores
//! - `cosine`: re-score by query↔chunk embedding cosine (async; postprocess only)

use std::collections::HashSet;

use crate::context::{QueryContext, RetrievedChunk, RetrievedEntity, RetrievedRelationship};

/// How chunk scores are computed before keep-m / floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelevancyScoreMode {
    /// Use fusion / retrieval scores already on chunks.
    #[default]
    Rrf,
    /// Re-score with query–chunk embedding cosine similarity.
    QueryEmbedCosine,
}

impl RelevancyScoreMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rrf => "rrf",
            Self::QueryEmbedCosine => "cosine",
        }
    }

    pub fn from_env_value(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cosine" | "embed" | "query_embed" | "query-embed" => Self::QueryEmbedCosine,
            _ => Self::Rrf,
        }
    }
}

/// Configuration for Mix / postprocess relevancy pruning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelevancyPruneConfig {
    /// Master switch (`EDGEQUAKE_MIX_RELEVANCY_PRUNE`).
    pub enabled: bool,
    /// Keep at most this many chunks after score sort. Default 10.
    pub keep_m: usize,
    /// Never drop below this many chunks when input is larger. Default 5.
    pub min_keep: usize,
    /// Drop chunks with score < top_score * relative_floor (0.0 disables). Default 0.35.
    pub relative_floor: f32,
    /// Soft-prune entities/rels not linked to kept chunks. Default true when enabled.
    pub graph_soft_prune: bool,
    /// Minimum entities to retain after soft-prune (fail-open). Default 5.
    pub min_entities: usize,
    /// Minimum relationships to retain after soft-prune (fail-open). Default 3.
    pub min_relationships: usize,
    /// Score source before keep/floor (`EDGEQUAKE_MIX_RELEVANCY_SCORE`).
    pub score_mode: RelevancyScoreMode,
    /// Max chars of chunk content sent to the embedder (cosine mode). Default 2000.
    pub embed_char_cap: usize,
}

impl Default for RelevancyPruneConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            keep_m: 10,
            min_keep: 5,
            relative_floor: 0.35,
            graph_soft_prune: true,
            min_entities: 5,
            min_relationships: 3,
            score_mode: RelevancyScoreMode::Rrf,
            embed_char_cap: 2000,
        }
    }
}

impl RelevancyPruneConfig {
    /// Read from environment. Off unless `EDGEQUAKE_MIX_RELEVANCY_PRUNE` is truthy.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        cfg.enabled = env_truthy("EDGEQUAKE_MIX_RELEVANCY_PRUNE");
        if let Ok(v) = std::env::var("EDGEQUAKE_MIX_RELEVANCY_KEEP") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.keep_m = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("EDGEQUAKE_MIX_RELEVANCY_MIN_KEEP") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.min_keep = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("EDGEQUAKE_MIX_RELEVANCY_SCORE_FLOOR") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.relative_floor = f.clamp(0.0, 1.0);
            }
        }
        if std::env::var("EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE").is_ok() {
            cfg.graph_soft_prune = env_truthy("EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE");
        }
        if let Ok(v) = std::env::var("EDGEQUAKE_MIX_RELEVANCY_SCORE") {
            cfg.score_mode = RelevancyScoreMode::from_env_value(&v);
        }
        if let Ok(v) = std::env::var("EDGEQUAKE_MIX_RELEVANCY_EMBED_CHARS") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.embed_char_cap = n.max(64);
            }
        }
        cfg
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// True when Mix fuse should apply sync RRF-score prune.
    pub fn applies_in_mix_fuse(&self) -> bool {
        self.enabled && matches!(self.score_mode, RelevancyScoreMode::Rrf)
    }

    /// True when postprocess should run async query-embed cosine prune.
    pub fn uses_query_embed_cosine(&self) -> bool {
        self.enabled && matches!(self.score_mode, RelevancyScoreMode::QueryEmbedCosine)
    }
}

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Cosine similarity in `[-1, 1]`. Returns 0.0 on empty/mismatched dims.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Cap chunk text for embedding calls.
pub fn capped_chunk_text(content: &str, cap: usize) -> String {
    if content.len() <= cap {
        content.to_string()
    } else {
        content.chars().take(cap).collect()
    }
}

/// Rewrite chunk.score from query↔chunk cosine, then keep-m / floor.
pub fn rescore_and_prune_by_cosine(
    mut chunks: Vec<RetrievedChunk>,
    query_vec: &[f32],
    chunk_vecs: &[Vec<f32>],
    config: &RelevancyPruneConfig,
) -> Vec<RetrievedChunk> {
    if chunks.is_empty() {
        return chunks;
    }
    if chunk_vecs.len() != chunks.len() {
        tracing::warn!(
            chunks = chunks.len(),
            vecs = chunk_vecs.len(),
            "relevancy_prune cosine: vec count mismatch — skip rescore"
        );
        return prune_chunks_by_relevancy(chunks, config);
    }
    for (chunk, vec) in chunks.iter_mut().zip(chunk_vecs.iter()) {
        chunk.score = cosine_similarity(query_vec, vec);
    }
    prune_chunks_by_relevancy(chunks, config)
}

/// Keep top-scoring chunks; apply relative floor; fail-open if empty.
pub fn prune_chunks_by_relevancy(
    chunks: Vec<RetrievedChunk>,
    config: &RelevancyPruneConfig,
) -> Vec<RetrievedChunk> {
    if !config.enabled() || chunks.len() <= config.min_keep {
        return chunks;
    }

    let mut ranked = chunks;
    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top = ranked.first().map(|c| c.score).unwrap_or(0.0);
    let floor = if config.relative_floor > 0.0 && top > 0.0 {
        top * config.relative_floor
    } else {
        f32::NEG_INFINITY
    };

    let target = config.keep_m.max(config.min_keep);
    let mut kept: Vec<RetrievedChunk> = ranked
        .iter()
        .filter(|c| c.score >= floor)
        .take(target)
        .cloned()
        .collect();

    if kept.len() < config.min_keep {
        kept = ranked.into_iter().take(config.min_keep.max(1)).collect();
    }

    if kept.is_empty() {
        // Fail-open: should be unreachable if input non-empty, but be safe.
        return Vec::new();
    }
    kept
}

/// Drop entities/relationships that do not touch kept chunk IDs (soft).
///
/// Entities with empty `source_chunk_ids` are kept only if needed to hit
/// `min_entities` after lineage filtering (fail-open by score).
pub fn soft_prune_graph(
    entities: Vec<RetrievedEntity>,
    relationships: Vec<RetrievedRelationship>,
    kept_chunk_ids: &HashSet<String>,
    config: &RelevancyPruneConfig,
) -> (Vec<RetrievedEntity>, Vec<RetrievedRelationship>) {
    if !config.graph_soft_prune || kept_chunk_ids.is_empty() {
        return (entities, relationships);
    }

    let mut linked: Vec<RetrievedEntity> = entities
        .iter()
        .filter(|e| {
            e.source_chunk_ids
                .iter()
                .any(|id| kept_chunk_ids.contains(id))
        })
        .cloned()
        .collect();

    if linked.len() < config.min_entities {
        let mut by_score = entities.clone();
        by_score.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.degree.cmp(&a.degree))
        });
        let mut seen: HashSet<String> = linked.iter().map(|e| e.name.clone()).collect();
        for e in by_score {
            if linked.len() >= config.min_entities {
                break;
            }
            if seen.insert(e.name.clone()) {
                linked.push(e);
            }
        }
    }

    let kept_names: HashSet<String> = linked.iter().map(|e| e.name.clone()).collect();
    let mut kept_rels: Vec<RetrievedRelationship> = relationships
        .iter()
        .filter(|r| kept_names.contains(&r.source) && kept_names.contains(&r.target))
        .cloned()
        .collect();

    if kept_rels.len() < config.min_relationships && !relationships.is_empty() {
        let mut by_score = relationships;
        by_score.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut seen = HashSet::new();
        for r in &kept_rels {
            seen.insert(format!("{}|{}|{}", r.source, r.relation_type, r.target));
        }
        for r in by_score {
            if kept_rels.len() >= config.min_relationships {
                break;
            }
            let key = format!("{}|{}|{}", r.source, r.relation_type, r.target);
            if seen.insert(key) {
                kept_rels.push(r);
            }
        }
    }

    (linked, kept_rels)
}

/// Apply chunk + graph relevancy prune to a full context. No-op when disabled.
pub fn apply_relevancy_prune(mut ctx: QueryContext, config: &RelevancyPruneConfig) -> QueryContext {
    if !config.enabled() {
        return ctx;
    }

    let before_c = ctx.chunks.len();
    let before_e = ctx.entities.len();
    let before_r = ctx.relationships.len();

    let original_chunks = ctx.chunks.clone();
    let pruned = prune_chunks_by_relevancy(std::mem::take(&mut ctx.chunks), config);
    if pruned.is_empty() && !original_chunks.is_empty() {
        // Fail-open: restore chunks if prune emptied them.
        ctx.chunks = original_chunks;
        tracing::warn!("relevancy_prune: empty after prune — restored original chunks");
    } else {
        ctx.chunks = pruned;
    }

    if config.graph_soft_prune {
        let kept_ids: HashSet<String> = ctx.chunks.iter().map(|c| c.id.clone()).collect();
        let (ents, rels) = soft_prune_graph(
            std::mem::take(&mut ctx.entities),
            std::mem::take(&mut ctx.relationships),
            &kept_ids,
            config,
        );
        ctx.entities = ents;
        ctx.relationships = rels;
    }

    if ctx.chunks.len() < before_c
        || ctx.entities.len() < before_e
        || ctx.relationships.len() < before_r
    {
        ctx.is_truncated = true;
    }

    record_prune_metadata(
        &mut ctx,
        config,
        before_c,
        before_e,
        before_r,
        config.score_mode.as_str(),
    );

    tracing::debug!(
        chunks_before = before_c,
        chunks_after = ctx.chunks.len(),
        entities_before = before_e,
        entities_after = ctx.entities.len(),
        relationships_before = before_r,
        relationships_after = ctx.relationships.len(),
        keep_m = config.keep_m,
        score_mode = config.score_mode.as_str(),
        "relevancy_prune applied"
    );

    ctx
}

/// Apply cosine-rescored prune after embeddings are available.
pub fn apply_cosine_rescored_prune(
    mut ctx: QueryContext,
    query_vec: &[f32],
    chunk_vecs: &[Vec<f32>],
    config: &RelevancyPruneConfig,
) -> QueryContext {
    if !config.enabled() {
        return ctx;
    }

    let before_c = ctx.chunks.len();
    let before_e = ctx.entities.len();
    let before_r = ctx.relationships.len();

    let original_chunks = ctx.chunks.clone();
    let pruned = rescore_and_prune_by_cosine(
        std::mem::take(&mut ctx.chunks),
        query_vec,
        chunk_vecs,
        config,
    );
    if pruned.is_empty() && !original_chunks.is_empty() {
        ctx.chunks = original_chunks;
        tracing::warn!("relevancy_prune cosine: empty after prune — restored original chunks");
    } else {
        ctx.chunks = pruned;
    }

    if config.graph_soft_prune {
        let kept_ids: HashSet<String> = ctx.chunks.iter().map(|c| c.id.clone()).collect();
        let (ents, rels) = soft_prune_graph(
            std::mem::take(&mut ctx.entities),
            std::mem::take(&mut ctx.relationships),
            &kept_ids,
            config,
        );
        ctx.entities = ents;
        ctx.relationships = rels;
    }

    if ctx.chunks.len() < before_c
        || ctx.entities.len() < before_e
        || ctx.relationships.len() < before_r
    {
        ctx.is_truncated = true;
    }

    record_prune_metadata(
        &mut ctx,
        config,
        before_c,
        before_e,
        before_r,
        RelevancyScoreMode::QueryEmbedCosine.as_str(),
    );
    ctx
}

fn record_prune_metadata(
    ctx: &mut QueryContext,
    config: &RelevancyPruneConfig,
    before_c: usize,
    before_e: usize,
    before_r: usize,
    score_mode: &str,
) {
    ctx.metadata.insert(
        "relevancy_prune".into(),
        serde_json::json!({
            "enabled": true,
            "score_mode": score_mode,
            "keep_m": config.keep_m,
            "min_keep": config.min_keep,
            "relative_floor": config.relative_floor,
            "graph_soft_prune": config.graph_soft_prune,
            "chunks_before": before_c,
            "chunks_after": ctx.chunks.len(),
            "entities_before": before_e,
            "entities_after": ctx.entities.len(),
            "relationships_before": before_r,
            "relationships_after": ctx.relationships.len(),
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &str, score: f32) -> RetrievedChunk {
        RetrievedChunk::new(id, format!("content-{id}"), score)
    }

    fn entity(name: &str, chunks: &[&str], score: f32) -> RetrievedEntity {
        RetrievedEntity::new(name, "CONCEPT", "desc")
            .with_score(score)
            .with_source_chunk_ids(chunks.iter().map(|s| (*s).to_string()).collect())
    }

    #[test]
    fn disabled_is_noop() {
        let cfg = RelevancyPruneConfig::default();
        let mut ctx = QueryContext::new();
        ctx.add_chunk(chunk("a", 1.0));
        ctx.add_chunk(chunk("b", 0.5));
        let out = apply_relevancy_prune(ctx, &cfg);
        assert_eq!(out.chunks.len(), 2);
    }

    #[test]
    fn keeps_top_m_by_score() {
        let cfg = RelevancyPruneConfig {
            enabled: true,
            keep_m: 2,
            min_keep: 1,
            relative_floor: 0.0,
            graph_soft_prune: false,
            ..Default::default()
        };
        let chunks = vec![chunk("low", 0.1), chunk("mid", 0.5), chunk("hi", 0.9)];
        let kept = prune_chunks_by_relevancy(chunks, &cfg);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].id, "hi");
        assert_eq!(kept[1].id, "mid");
    }

    #[test]
    fn relative_floor_drops_tail() {
        let cfg = RelevancyPruneConfig {
            enabled: true,
            keep_m: 10,
            min_keep: 1,
            relative_floor: 0.5,
            graph_soft_prune: false,
            ..Default::default()
        };
        // top=1.0 → floor=0.5; keep a,b drop c
        let chunks = vec![chunk("a", 1.0), chunk("b", 0.6), chunk("c", 0.2)];
        let kept = prune_chunks_by_relevancy(chunks, &cfg);
        assert_eq!(kept.len(), 2);
        assert!(kept.iter().all(|c| c.id != "c"));
    }

    #[test]
    fn soft_prune_keeps_lineage_entities() {
        let cfg = RelevancyPruneConfig {
            enabled: true,
            keep_m: 1,
            min_keep: 1,
            relative_floor: 0.0,
            graph_soft_prune: true,
            min_entities: 1,
            min_relationships: 0,
            ..Default::default()
        };
        let mut ctx = QueryContext::new();
        ctx.add_chunk(chunk("c1", 1.0));
        ctx.add_chunk(chunk("c2", 0.1));
        ctx.add_entity(entity("KEEP", &["c1"], 1.0));
        ctx.add_entity(entity("DROP", &["c2"], 0.9));
        let out = apply_relevancy_prune(ctx, &cfg);
        assert_eq!(out.chunks.len(), 1);
        assert_eq!(out.chunks[0].id, "c1");
        assert_eq!(out.entities.len(), 1);
        assert_eq!(out.entities[0].name, "KEEP");
    }

    #[test]
    fn fail_open_min_keep() {
        let cfg = RelevancyPruneConfig {
            enabled: true,
            keep_m: 1,
            min_keep: 2,
            relative_floor: 0.99, // would keep only top if not for min_keep restore path
            graph_soft_prune: false,
            ..Default::default()
        };
        let chunks = vec![chunk("a", 1.0), chunk("b", 0.5), chunk("c", 0.4)];
        let kept = prune_chunks_by_relevancy(chunks, &cfg);
        // floor drops b,c → only a → below min_keep → restore top min_keep
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn cosine_identical_is_one() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_rescore_prefers_aligned_chunk() {
        let cfg = RelevancyPruneConfig {
            enabled: true,
            keep_m: 1,
            min_keep: 1,
            relative_floor: 0.0,
            graph_soft_prune: false,
            score_mode: RelevancyScoreMode::QueryEmbedCosine,
            ..Default::default()
        };
        let query = vec![1.0, 0.0];
        // RRF scores inverted vs cosine: "noise" has higher RRF score
        let chunks = vec![chunk("noise", 0.9), chunk("signal", 0.1)];
        let vecs = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let kept = rescore_and_prune_by_cosine(chunks, &query, &vecs, &cfg);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, "signal");
        assert!((kept[0].score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_mode_skips_mix_fuse() {
        let cfg = RelevancyPruneConfig {
            enabled: true,
            score_mode: RelevancyScoreMode::QueryEmbedCosine,
            ..Default::default()
        };
        assert!(!cfg.applies_in_mix_fuse());
        assert!(cfg.uses_query_embed_cosine());
    }
}
