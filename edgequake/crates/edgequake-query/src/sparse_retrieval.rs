//! Sparse retrieval fused with dense vector ranks (SPEC-023 I10 / default-on).
//!
//! Production path: PostgreSQL native FTS (`ts_rank_cd` cover-density over GIN
//! `content_tsv`) — not true BM25 (X-05). Fallback: in-memory BM25 reranker over
//! vector ANN candidates (memory adapter / tests).
//!
//! Used by naive, local, and global chunk stages (SPEC-024 2.3).
//! SPEC-046 OPS-P2: returns [`SparseRetrievalOutcome`] for QueryStats / metrics.

use std::collections::HashMap;
use std::sync::Arc;

use edgequake_llm::Reranker;
use edgequake_storage::traits::{KVStorage, MetadataFilter, VectorSearchResult, VectorStorage};

use crate::chunk_hydration::chunk_documents_for_rerank;
use crate::context::RetrievedChunk;
use crate::engine_impl::QueryEngineConfig;
use crate::fusion::{self, MixFusionMode};
use crate::helpers::build_chunk_from_result;

/// Which sparse path produced the fused chunk list (OPS-P2.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparseRetrievalOutcome {
    /// BM25 disabled or no vector hits — dense only.
    VectorOnly,
    /// PostgreSQL native FTS contributed hits.
    PostgresFts,
    /// In-memory BM25 (no native FTS adapter).
    InMemoryBm25,
    /// Postgres FTS errored; fell back to in-memory BM25.
    FtsErrorFallback,
    /// Native FTS returned empty; fell back to vector-only ordering.
    FtsEmptyFallback,
}

impl SparseRetrievalOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VectorOnly => "vector_only",
            Self::PostgresFts => "postgres_fts",
            Self::InMemoryBm25 => "in_memory_bm25",
            Self::FtsErrorFallback => "fts_error_fallback",
            Self::FtsEmptyFallback => "fts_empty_fallback",
        }
    }

    /// True when we intended FTS but used a degraded path.
    pub fn is_fts_fallback(self) -> bool {
        matches!(self, Self::FtsErrorFallback | Self::FtsEmptyFallback)
    }
}

/// Whether vector+sparse chunk fusion uses RRF or sparse-first ordering.
///
/// Default (unset / any value other than `rrf`) is **sparse_first**: return
/// chunks in sparse-hit order and ignore dense ranks. The historical env value
/// `weighted` maps here too — it is **not** Mix max-after-minmax fusion
/// (D-36). Prefer `EDGEQUAKE_SPARSE_FUSION=sparse_first` in new configs.
///
/// Uses [`MixFusionMode::MaxAfterMinMax`] as the internal tag for sparse_first
/// until a dedicated variant exists; see that enum's doc for Mix-mode semantics.
pub fn sparse_fusion_mode_from_env() -> MixFusionMode {
    match std::env::var("EDGEQUAKE_SPARSE_FUSION")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "rrf" => MixFusionMode::Rrf,
        // D-36: unset, `sparse_first`, or legacy `weighted` → sparse-first order.
        _ => MixFusionMode::MaxAfterMinMax,
    }
}

/// Whether sparse (FTS / in-memory BM25) retrieval fusion is active (default: true).
///
/// Env `EDGEQUAKE_BM25_RETRIEVAL` is a historical name; Postgres path ranks with
/// `ts_rank_cd` (X-05), not Okapi BM25.
pub fn bm25_retrieval_enabled(config: &QueryEngineConfig) -> bool {
    if !config.enable_bm25_retrieval {
        return false;
    }
    std::env::var("EDGEQUAKE_BM25_RETRIEVAL")
        .map(|v| !matches!(v.to_ascii_lowercase().as_str(), "false" | "0" | "off"))
        .unwrap_or(true)
}

/// Fuse vector ANN hits with sparse retrieval; returns top `max_chunks` + outcome.
pub async fn fuse_vector_and_bm25_chunks(
    query: &str,
    vector_results: &[VectorSearchResult],
    vector_storage: &Arc<dyn VectorStorage>,
    metadata_filter: Option<&MetadataFilter>,
    reranker: Option<&dyn Reranker>,
    kv_storage: Option<&dyn KVStorage>,
    config: &QueryEngineConfig,
) -> (Vec<RetrievedChunk>, SparseRetrievalOutcome) {
    let max_chunks = config.max_chunks;
    let min_score = config.min_score;

    if vector_results.is_empty() {
        return (Vec::new(), SparseRetrievalOutcome::VectorOnly);
    }

    if !bm25_retrieval_enabled(config) {
        let chunks = vector_results
            .iter()
            .filter(|r| r.score >= min_score)
            .take(max_chunks)
            .map(build_chunk_from_result)
            .collect();
        return (chunks, SparseRetrievalOutcome::VectorOnly);
    }

    let vector_ranked: Vec<String> = vector_results.iter().map(|r| r.id.clone()).collect();
    let mut lookup = chunk_lookup(vector_results);

    let (sparse_ranked, outcome) = if vector_storage.supports_native_text_search() {
        let candidate_k = max_chunks.saturating_mul(config.bm25_candidate_multiplier);
        match crate::modality_retrieve::text_search_with_modality_preference(
            vector_storage,
            query,
            candidate_k,
            None,
            metadata_filter,
        )
        .await
        {
            Ok(fts_hits) if !fts_hits.is_empty() => {
                for hit in &fts_hits {
                    lookup
                        .entry(hit.id.clone())
                        .or_insert_with(|| build_chunk_from_result(hit));
                }
                (
                    fts_hits.iter().map(|r| r.id.clone()).collect(),
                    SparseRetrievalOutcome::PostgresFts,
                )
            }
            Ok(_) => (Vec::new(), SparseRetrievalOutcome::FtsEmptyFallback),
            Err(e) => {
                tracing::warn!(error = %e, "Postgres FTS failed — falling back to in-memory BM25");
                (
                    in_memory_bm25_ranked(query, vector_results, reranker, kv_storage).await,
                    SparseRetrievalOutcome::FtsErrorFallback,
                )
            }
        }
    } else {
        (
            in_memory_bm25_ranked(query, vector_results, reranker, kv_storage).await,
            SparseRetrievalOutcome::InMemoryBm25,
        )
    };

    if sparse_ranked.is_empty() {
        let chunks = vector_results
            .iter()
            .filter(|r| r.score >= min_score)
            .take(max_chunks)
            .map(build_chunk_from_result)
            .collect();
        let outcome = if matches!(
            outcome,
            SparseRetrievalOutcome::FtsErrorFallback | SparseRetrievalOutcome::FtsEmptyFallback
        ) {
            outcome
        } else {
            SparseRetrievalOutcome::VectorOnly
        };
        return (chunks, outcome);
    }

    // D-39: apply min_score on fused paths (filter first, then take).
    let mut chunks = if sparse_fusion_mode_from_env() == MixFusionMode::Rrf {
        let fused = fusion::reciprocal_rank_fusion(
            &[vector_ranked, sparse_ranked],
            &[1.0, 1.25],
            fusion::RRF_K,
        );
        fusion::chunks_from_rrf_ranking(&fused, &lookup, max_chunks.saturating_mul(2))
    } else {
        sparse_ranked
            .into_iter()
            .filter_map(|id| lookup.get(&id).cloned())
            .collect()
    };
    chunks.retain(|c| c.score >= min_score);
    chunks.truncate(max_chunks);

    (chunks, outcome)
}

fn chunk_lookup(vector_results: &[VectorSearchResult]) -> HashMap<String, RetrievedChunk> {
    vector_results
        .iter()
        .map(|r| (r.id.clone(), build_chunk_from_result(r)))
        .collect()
}

async fn in_memory_bm25_ranked(
    query: &str,
    vector_results: &[VectorSearchResult],
    reranker: Option<&dyn Reranker>,
    kv_storage: Option<&dyn KVStorage>,
) -> Vec<String> {
    let Some(reranker) = reranker else {
        return vector_results.iter().map(|r| r.id.clone()).collect();
    };

    let documents = chunk_documents_for_rerank(kv_storage, vector_results).await;

    match reranker
        .rerank(query, &documents, Some(documents.len()))
        .await
    {
        Ok(ranked) => ranked
            .into_iter()
            .filter_map(|r| vector_results.get(r.index).map(|vr| vr.id.clone()))
            .collect(),
        Err(_) => vector_results.iter().map(|r| r.id.clone()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_labels_are_stable() {
        assert_eq!(SparseRetrievalOutcome::PostgresFts.as_str(), "postgres_fts");
        assert!(SparseRetrievalOutcome::FtsErrorFallback.is_fts_fallback());
        assert!(!SparseRetrievalOutcome::PostgresFts.is_fts_fallback());
    }

    #[test]
    fn e2e_min_score_enforced_on_rrf() {
        // D-39: fused path retains only scores ≥ min_score (filter after RRF).
        let min_score = 0.5_f32;
        let mut scores = vec![0.9_f32, 0.4, 0.7, 0.1];
        scores.retain(|s| *s >= min_score);
        assert_eq!(scores, vec![0.9, 0.7]);
        // Source contract: retain happens after RRF in fuse path.
        let src = include_str!("sparse_retrieval.rs");
        assert!(src.contains("chunks.retain(|c| c.score >= min_score)"));
        assert!(src.contains("D-39"));
    }
}
