//! Mix query mode — intent-gated weighted/RRF blend of local, global, and naive arms.
//!
//! SPEC-046 OPS-P1: skip zero-weight / intent-gated arms (DRY via [`resolve_arm_plan`]).

use std::collections::HashMap;
use std::sync::Arc;

use crate::context::{QueryContext, RetrievedChunk};
use crate::error::Result;
use crate::keywords::ExtractedKeywords;
use crate::mix_weights::{
    mix_arm_gate_enabled, resolve_arm_plan, ArmPlan, META_ARMS_GATED, META_ARMS_RUN,
    META_ARM_GLOBAL_CHUNKS, META_ARM_GLOBAL_MS, META_ARM_LOCAL_CHUNKS, META_ARM_LOCAL_MS,
    META_ARM_NAIVE_CHUNKS, META_ARM_NAIVE_MS,
};

use edgequake_storage::traits::VectorStorage;

use super::super::{QueryEmbeddings, QueryEngine};
use super::arm_timed::run_arm_timed;

impl QueryEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine_impl) async fn query_mix_with_vector_storage(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        allowed_document_ids: Option<&[String]>,
        vector_storage: &Arc<dyn VectorStorage>,
        mix_weights: Option<&crate::mix_weights::MixWeightOverride>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        let plan = resolve_arm_plan(
            &self.config,
            mix_weights,
            keywords.query_intent,
            mix_arm_gate_enabled(),
        );

        // Box each arm so join! holds three pointers, not three full retrieval FSMs
        // (same stack-overflow class as Hybrid — SPEC-047).
        let local_fut = Box::pin(run_arm_timed(
            plan.run_local,
            "local",
            "mix",
            query_text,
            max_chunks,
            || {
                self.query_local_with_vector_storage(
                    query_text,
                    keywords,
                    embeddings,
                    tenant_id.clone(),
                    workspace_id.clone(),
                    allowed_document_ids,
                    vector_storage,
                    max_chunks,
                )
            },
        ));
        let global_fut = Box::pin(run_arm_timed(
            plan.run_global,
            "global",
            "mix",
            query_text,
            max_chunks,
            || {
                self.query_global_with_vector_storage(
                    query_text,
                    keywords,
                    embeddings,
                    tenant_id.clone(),
                    workspace_id.clone(),
                    allowed_document_ids,
                    vector_storage,
                    max_chunks,
                )
            },
        ));
        let naive_fut = Box::pin(run_arm_timed(
            plan.run_naive,
            "naive",
            "mix",
            query_text,
            max_chunks,
            || {
                self.query_naive_with_vector_storage(
                    query_text,
                    embeddings,
                    tenant_id.clone(),
                    workspace_id.clone(),
                    allowed_document_ids,
                    vector_storage,
                    max_chunks,
                )
            },
        ));
        let (local_res, global_res, naive_res) = tokio::join!(local_fut, global_fut, naive_fut);

        let (local_context, local_ms) = local_res?;
        let (global_context, global_ms) = global_res?;
        let (naive_context, naive_ms) = naive_res?;

        let local_chunks = local_context.chunks.len();
        let global_chunks = global_context.chunks.len();
        let naive_chunks = naive_context.chunks.len();

        let mut merged = fuse_mix_contexts(
            &local_context,
            &global_context,
            &naive_context,
            plan,
            max_chunks,
        );

        attach_arm_metadata(
            &mut merged,
            plan,
            local_ms,
            global_ms,
            naive_ms,
            local_chunks,
            global_chunks,
            naive_chunks,
        );

        tracing::debug!(
            merged_chunks = merged.chunks.len(),
            merged_entities = merged.entities.len(),
            merged_relationships = merged.relationships.len(),
            run_local = plan.run_local,
            run_global = plan.run_global,
            run_naive = plan.run_naive,
            w_local = plan.w_local,
            w_global = plan.w_global,
            w_naive = plan.w_naive,
            local_ms,
            global_ms,
            naive_ms,
            "Mix merge complete (intent-gated)"
        );

        Ok(merged)
    }
}

/// Shared Mix fusion (weighted or RRF) — pure relative to arm execution.
fn fuse_mix_contexts(
    local_context: &QueryContext,
    global_context: &QueryContext,
    naive_context: &QueryContext,
    plan: ArmPlan,
    max_chunks: usize,
) -> QueryContext {
    let w_local = plan.w_local;
    let w_global = plan.w_global;
    let w_naive = plan.w_naive;

    let mut merged = QueryContext::new();

    if crate::fusion::mix_fusion_mode_from_env() == crate::fusion::MixFusionMode::Rrf {
        let mut chunk_lookup: HashMap<String, RetrievedChunk> = HashMap::new();
        for ctx in [local_context, global_context, naive_context] {
            for chunk in &ctx.chunks {
                chunk_lookup
                    .entry(chunk.id.clone())
                    .or_insert_with(|| chunk.clone());
            }
        }

        let ranked_lists = [
            local_context.chunks.iter().map(|c| c.id.clone()).collect(),
            global_context.chunks.iter().map(|c| c.id.clone()).collect(),
            naive_context.chunks.iter().map(|c| c.id.clone()).collect(),
        ];
        let weights = [w_local, w_global, w_naive];
        let fused =
            crate::fusion::reciprocal_rank_fusion(&ranked_lists, &weights, crate::fusion::RRF_K);
        for chunk in crate::fusion::chunks_from_rrf_ranking(&fused, &chunk_lookup, max_chunks) {
            merged.add_chunk(chunk);
        }
    } else {
        let mut blended: HashMap<String, (RetrievedChunk, f32)> = HashMap::new();
        for (ctx, weight) in [
            (local_context, w_local),
            (global_context, w_global),
            (naive_context, w_naive),
        ] {
            if weight <= 0.0 {
                continue;
            }
            let norm = min_max_normalize_scores(&ctx.chunks);
            for (chunk, &norm_score) in ctx.chunks.iter().zip(norm.iter()) {
                let contribution = weight * norm_score;
                blended
                    .entry(chunk.id.clone())
                    .and_modify(|(_, score)| {
                        if contribution > *score {
                            *score = contribution;
                        }
                    })
                    .or_insert_with(|| (chunk.clone(), contribution));
            }
        }

        let mut chunks: Vec<(RetrievedChunk, f32)> = blended.into_values().collect();
        chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (mut chunk, score) in chunks.into_iter().take(max_chunks) {
            chunk.score = score.max(0.0);
            merged.add_chunk(chunk);
        }
    }

    let mut seen_entities = std::collections::HashSet::new();
    for e in local_context
        .entities
        .iter()
        .chain(global_context.entities.iter())
    {
        if seen_entities.insert(e.name.clone()) {
            merged.add_entity(e.clone());
        }
    }
    let mut seen_rels = std::collections::HashSet::new();
    for rel in local_context
        .relationships
        .iter()
        .chain(global_context.relationships.iter())
    {
        let key = format!("{}-{}-{}", rel.source, rel.relation_type, rel.target);
        if seen_rels.insert(key) {
            merged.add_relationship(rel.clone());
        }
    }

    merged
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach_arm_metadata(
    ctx: &mut QueryContext,
    plan: ArmPlan,
    local_ms: u64,
    global_ms: u64,
    naive_ms: u64,
    local_chunks: usize,
    global_chunks: usize,
    naive_chunks: usize,
) {
    if plan.run_local {
        ctx.metadata
            .insert(META_ARM_LOCAL_MS.into(), serde_json::json!(local_ms));
        ctx.metadata.insert(
            META_ARM_LOCAL_CHUNKS.into(),
            serde_json::json!(local_chunks as u64),
        );
    }
    if plan.run_global {
        ctx.metadata
            .insert(META_ARM_GLOBAL_MS.into(), serde_json::json!(global_ms));
        ctx.metadata.insert(
            META_ARM_GLOBAL_CHUNKS.into(),
            serde_json::json!(global_chunks as u64),
        );
    }
    if plan.run_naive {
        ctx.metadata
            .insert(META_ARM_NAIVE_MS.into(), serde_json::json!(naive_ms));
        ctx.metadata.insert(
            META_ARM_NAIVE_CHUNKS.into(),
            serde_json::json!(naive_chunks as u64),
        );
    }
    let mut run = Vec::new();
    if plan.run_local {
        run.push("local");
    }
    if plan.run_global {
        run.push("global");
    }
    if plan.run_naive {
        run.push("naive");
    }
    ctx.metadata
        .insert(META_ARMS_RUN.into(), serde_json::json!(run.join(",")));
    let gated = !(plan.run_local && plan.run_global && plan.run_naive);
    ctx.metadata
        .insert(META_ARMS_GATED.into(), serde_json::json!(gated));
}

fn min_max_normalize_scores(chunks: &[crate::context::RetrievedChunk]) -> Vec<f32> {
    if chunks.is_empty() {
        return Vec::new();
    }
    let (min, max) = chunks
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(mn, mx), c| {
            (mn.min(c.score), mx.max(c.score))
        });
    let range = max - min;
    chunks
        .iter()
        .map(|c| {
            if range <= 0.0 {
                1.0
            } else {
                (c.score - min) / range
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_impl::QueryEngineConfig;
    use crate::keywords::QueryIntent;
    use crate::mix_weights::{resolve_arm_plan, MixWeightOverride};

    #[test]
    fn attach_metadata_marks_gated_when_subset() {
        let plan = resolve_arm_plan(
            &QueryEngineConfig::default(),
            None,
            QueryIntent::Factual,
            true,
        );
        let mut ctx = QueryContext::new();
        attach_arm_metadata(&mut ctx, plan, 1, 2, 3, 0, 0, 4);
        assert_eq!(ctx.metadata.get(META_ARMS_RUN).unwrap(), "naive");
        assert_eq!(ctx.metadata.get(META_ARMS_GATED).unwrap(), true);
        assert!(ctx.metadata.get(META_ARM_NAIVE_MS).is_some());
        assert_eq!(ctx.metadata.get(META_ARM_NAIVE_CHUNKS).unwrap(), 4);
        assert!(ctx.metadata.get(META_ARM_LOCAL_MS).is_none());
        assert!(ctx.metadata.get(META_ARM_LOCAL_CHUNKS).is_none());
    }

    #[test]
    fn fuse_skips_zero_weight_arms() {
        let mut naive = QueryContext::new();
        naive.add_chunk(RetrievedChunk::new("c1", "x", 1.0));
        let plan = resolve_arm_plan(
            &QueryEngineConfig::default(),
            Some(&MixWeightOverride {
                local: Some(0.0),
                global: Some(0.0),
                naive: Some(1.0),
            }),
            QueryIntent::Comparative,
            false,
        );
        let merged =
            fuse_mix_contexts(&QueryContext::new(), &QueryContext::new(), &naive, plan, 10);
        assert_eq!(merged.chunks.len(), 1);
        assert_eq!(merged.chunks[0].id, "c1");
    }
}
