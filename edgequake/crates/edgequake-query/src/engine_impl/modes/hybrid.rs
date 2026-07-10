//! Hybrid query mode — intent-gated LightRAG merge of local, global, and naive arms.
//!
//! SPEC-046 OPS-P1: skip intent-gated arms (DRY via [`resolve_hybrid_arm_plan`]).

use std::sync::Arc;

use crate::context::QueryContext;
use crate::error::Result;
use crate::keywords::ExtractedKeywords;
use crate::mix_weights::{mix_arm_gate_enabled, resolve_hybrid_arm_plan};

use edgequake_storage::traits::VectorStorage;

use super::super::{QueryEmbeddings, QueryEngine};
use super::arm_timed::run_arm_timed;
use super::mix::attach_arm_metadata;

impl QueryEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine_impl) async fn query_hybrid_with_vector_storage(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        allowed_document_ids: Option<&[String]>,
        vector_storage: &Arc<dyn VectorStorage>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        let plan = resolve_hybrid_arm_plan(keywords.query_intent, mix_arm_gate_enabled());

        let (local_res, global_res, naive_res) = tokio::join!(
            run_arm_timed(
                plan.run_local,
                "local",
                "hybrid",
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
                }
            ),
            run_arm_timed(
                plan.run_global,
                "global",
                "hybrid",
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
                }
            ),
            run_arm_timed(
                plan.run_naive,
                "naive",
                "hybrid",
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
                }
            ),
        );

        let (local_context, local_ms) = local_res?;
        let (global_context, global_ms) = global_res?;
        let (naive_context, naive_ms) = naive_res?;

        let fusion_mode = crate::hybrid_merge::hybrid_fusion_mode_from_env();
        tracing::debug!(
            naive_chunks = naive_context.chunks.len(),
            local_chunks = local_context.chunks.len(),
            local_entities = local_context.entities.len(),
            global_chunks = global_context.chunks.len(),
            global_entities = global_context.entities.len(),
            ?fusion_mode,
            max_chunks,
            run_local = plan.run_local,
            run_global = plan.run_global,
            run_naive = plan.run_naive,
            "Hybrid merge: intent-gated LightRAG-style"
        );

        let mut merged = crate::hybrid_merge::merge_hybrid_contexts(
            local_context,
            global_context,
            naive_context,
            max_chunks,
        );

        attach_arm_metadata(&mut merged, plan, local_ms, global_ms, naive_ms);

        tracing::debug!(
            merged_chunks = merged.chunks.len(),
            merged_entities = merged.entities.len(),
            merged_relationships = merged.relationships.len(),
            local_ms,
            global_ms,
            naive_ms,
            "Hybrid merge complete"
        );

        Ok(merged)
    }
}
