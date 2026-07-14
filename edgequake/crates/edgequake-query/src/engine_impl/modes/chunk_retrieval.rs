//! Shared chunk retrieval for local/global query modes.
//!
//! SPEC-046 P0.3: supports LightRAG-style `related_chunk_number` and
//! `kg_chunk_pick_method` (vector | weight).
//!
//! SPEC-047 / 021 L-A3: candidate chunk ids are intersected with allowed
//! documents before vector fetch (fail-closed under document_scope).

use std::sync::Arc;

use edgequake_storage::traits::{MetadataFilter, VectorStorage};

use crate::context::{QueryContext, RetrievedChunk};
use crate::engine_impl::{QueryEngine, QueryEngineConfig};
use crate::error::Result;
use crate::graph_ppr::GraphWalkMode;
use crate::helpers::build_chunk_from_result;
use crate::kg_chunk_pick::{
    collect_kg_chunk_ids_scoped, pick_chunks_by_bipartite_ppr, pick_chunks_by_weight,
    KgChunkPickMethod,
};
use crate::lineage_scope::filter_chunk_ids_by_allowed_docs;
use edgequake_storage::traits::GraphEdge;

#[allow(clippy::too_many_arguments)] // retrieval pipeline mirrors QueryEngine workspace arity
pub(super) async fn append_score_ranked_chunks(
    engine: &QueryEngine,
    context: &QueryContext,
    query_text: &str,
    query_embedding: &[f32],
    tenant_id: Option<String>,
    workspace_id: Option<String>,
    vector_storage: &Arc<dyn VectorStorage>,
    retrieval_config: &QueryEngineConfig,
    workspace_mf: Option<&MetadataFilter>,
    allowed_document_ids: Option<&[String]>,
    log_label: &str,
) -> Result<(
    Vec<RetrievedChunk>,
    crate::sparse_retrieval::SparseRetrievalOutcome,
)> {
    let related_n = retrieval_config.related_chunk_number;

    // Dual-node (EQ-046-17): bipartite PPR over entity relations ∪ mentions.
    // Falls back to lite entity-score projection when no relations are present.
    let raw_ids = if retrieval_config.graph_walk == GraphWalkMode::Ppr {
        let entity_edges: Vec<GraphEdge> = context
            .relationships
            .iter()
            .map(|r| GraphEdge {
                source: r.source.clone(),
                target: r.target.clone(),
                properties: std::collections::HashMap::new(),
            })
            .collect();
        let ranked =
            pick_chunks_by_bipartite_ppr(context, &entity_edges, retrieval_config.max_chunks);
        if ranked.is_empty() {
            collect_kg_chunk_ids_scoped(context, related_n, allowed_document_ids)
        } else {
            filter_chunk_ids_by_allowed_docs(&ranked, allowed_document_ids)
        }
    } else {
        match retrieval_config.kg_chunk_pick_method {
            KgChunkPickMethod::Weight => {
                let weighted = pick_chunks_by_weight(context, retrieval_config.max_chunks);
                if weighted.is_empty() {
                    collect_kg_chunk_ids_scoped(context, related_n, allowed_document_ids)
                } else {
                    filter_chunk_ids_by_allowed_docs(&weighted, allowed_document_ids)
                }
            }
            KgChunkPickMethod::Vector => {
                collect_kg_chunk_ids_scoped(context, related_n, allowed_document_ids)
            }
        }
    };

    let chunk_ids_vec = raw_ids;

    tracing::info!(
        total_chunk_ids = chunk_ids_vec.len(),
        entity_count = context.entities.len(),
        relationship_count = context.relationships.len(),
        pick_method = retrieval_config.kg_chunk_pick_method.as_str(),
        graph_walk = ?retrieval_config.graph_walk,
        related_chunk_number = related_n,
        scoped = allowed_document_ids.is_some(),
        log_label,
        "OODA-230: chunk collection (workspace)"
    );

    if chunk_ids_vec.is_empty() {
        return Ok((
            Vec::new(),
            crate::sparse_retrieval::SparseRetrievalOutcome::VectorOnly,
        ));
    }

    // Preserve ranked order for Weight and PPR dual-node picks; Vector uses score filter.
    let preserve_order = retrieval_config.kg_chunk_pick_method == KgChunkPickMethod::Weight
        || retrieval_config.graph_walk == GraphWalkMode::Ppr;

    let results = if preserve_order {
        let unordered = vector_storage
            .query_filtered(
                query_embedding,
                chunk_ids_vec.len().max(retrieval_config.max_chunks),
                Some(&chunk_ids_vec),
                workspace_mf,
            )
            .await?;
        let mut by_id: std::collections::HashMap<String, _> =
            unordered.into_iter().map(|r| (r.id.clone(), r)).collect();
        chunk_ids_vec
            .iter()
            .filter_map(|id| by_id.remove(id))
            .collect::<Vec<_>>()
    } else {
        vector_storage
            .query_filtered(
                query_embedding,
                retrieval_config.max_chunks,
                Some(&chunk_ids_vec),
                workspace_mf,
            )
            .await?
    };

    tracing::debug!(
        candidates = chunk_ids_vec.len(),
        returned = results.len(),
        log_label,
        "OODA-231: Chunk retrieval result"
    );

    let mf_chunk = MetadataFilter::from_tenant_workspace_type(tenant_id, workspace_id, "chunk");

    let (mut chunks, outcome) = if crate::sparse_retrieval::bm25_retrieval_enabled(retrieval_config)
    {
        crate::sparse_retrieval::fuse_vector_and_bm25_chunks(
            query_text,
            &results,
            vector_storage,
            mf_chunk.as_ref(),
            engine.reranker.as_deref(),
            engine.kv_storage.as_deref(),
            retrieval_config,
        )
        .await
    } else {
        (
            results
                .iter()
                .filter(|r| preserve_order || r.score >= retrieval_config.min_score)
                .take(retrieval_config.max_chunks)
                .map(build_chunk_from_result)
                .collect(),
            crate::sparse_retrieval::SparseRetrievalOutcome::VectorOnly,
        )
    };

    crate::chunk_hydration::hydrate_retrieved_chunks(engine.kv_storage.as_deref(), &mut chunks)
        .await;

    Ok((chunks, outcome))
}
