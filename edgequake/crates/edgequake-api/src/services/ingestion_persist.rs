//! SPEC-022 P-H1 — single ingestion persistence port for HTTP + worker paths (DIP).
//!
//! All callers delegate here after `Pipeline::process*` so chunk vectors, graph
//! merge, and saga compensation cannot diverge.

use std::sync::Arc;

use edgequake_llm::traits::LLMProvider;
use edgequake_pipeline::{
    ChunkVectorBuildOptions, DefaultIngestionPersister, IngestionPersistContext,
    IngestionPersistOutput, IngestionPersistSettings, IngestionPersister, LineageSink,
    MergeProgressCallback, NoopEntitySink, NoopLineageSink, ProcessingResult, RelationalEntitySink,
};
use edgequake_query::QueryResultCacheInvalidator;
use edgequake_storage::traits::domain::ChunkRepository;
use edgequake_storage::traits::{GraphStorage, KVStorage, VectorStorage};

#[cfg(feature = "postgres")]
use crate::postgres_entity_sink::PostgresEntitySink;
use crate::state::AppState;

/// Parameters for the shared persist path.
pub struct PersistIngestionParams<'a> {
    pub document_id: &'a str,
    pub tenant_id: Option<String>,
    pub workspace_id: String,
    pub result: &'a ProcessingResult,
    pub chunk_options: ChunkVectorBuildOptions,
    /// Optional chunk vector metadata (e.g. `"injection"` for SPEC-0002).
    pub source_type: Option<&'a str>,
    pub source_file_path: Option<&'a str>,
}

impl<'a> PersistIngestionParams<'a> {
    /// Standard document upload/worker params (no source overlay).
    pub fn for_document(
        document_id: &'a str,
        tenant_id: Option<String>,
        workspace_id: String,
        result: &'a ProcessingResult,
        chunk_options: ChunkVectorBuildOptions,
        source_file: Option<&'a str>,
    ) -> Self {
        Self {
            document_id,
            tenant_id,
            workspace_id,
            result,
            chunk_options,
            source_type: None,
            source_file_path: source_file,
        }
    }
}

/// Tag extraction records for knowledge injection (SPEC-0002 / SPEC-023 I1).
pub fn tag_injection_sources(result: &mut ProcessingResult, doc_id: &str) {
    for extraction in &mut result.extractions {
        for entity in &mut extraction.entities {
            entity.source_document_id = Some(doc_id.to_string());
            entity.source_file_path = Some("injection".to_string());
            if entity.source_chunk_ids.is_empty() {
                entity.source_chunk_ids = vec![format!("{doc_id}-chunk-0")];
            }
        }
        for rel in &mut extraction.relationships {
            rel.source_document_id = Some(doc_id.to_string());
            rel.source_file_path = Some("injection".to_string());
            if rel.source_chunk_id.is_none() {
                rel.source_chunk_id = Some(format!("{doc_id}-chunk-0"));
            }
        }
    }
}

/// Resolve the relational entity sink (CQRS dual-write when enabled).
///
/// Under typed embeddings, always returns a fail-closed Postgres sink so fleet
/// mirror has relational FKs even when `entity_sync_mode=disabled`.
pub async fn resolve_relational_sink(state: &AppState) -> Arc<dyn RelationalEntitySink> {
    #[cfg(feature = "postgres")]
    if let Some(ref pool) = state.pg_pool {
        return PostgresEntitySink::create_for_runtime(Arc::new(pool.clone())).await;
    }
    let _ = state;
    Arc::new(NoopEntitySink)
}

/// SPEC-091 W1: relational chunk writer when Postgres pool is available.
/// Authority flag (`EDGEQUAKE_CHUNK_TEXT_AUTHORITY`) still gates whether it is used.
#[cfg(feature = "postgres")]
pub fn resolve_relational_chunk_repo(
    pool: Option<&sqlx::PgPool>,
) -> Option<Arc<dyn ChunkRepository>> {
    pool.map(|pool| {
        Arc::new(edgequake_storage::PostgresChunkRepository::new(
            pool.clone(),
        )) as Arc<dyn ChunkRepository>
    })
}

/// SPEC-091 W3: typed chunk embedding index when Postgres pool is available.
/// Under `typed_embeddings`, this is write SSOT for chunk vectors (legacy
/// `eq_*_vectors` upserts are write-stopped at the adapter).
#[cfg(feature = "postgres")]
pub fn resolve_typed_embedding_index(
    pool: Option<sqlx::PgPool>,
) -> Option<Arc<dyn edgequake_storage::traits::domain::EmbeddingIndex>> {
    pool.map(|pool| {
        let model = std::env::var("EDGEQUAKE_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-small".to_string());
        Arc::new(edgequake_storage::PgChunkEmbeddingIndex::new(pool, model))
            as Arc<dyn edgequake_storage::traits::domain::EmbeddingIndex>
    })
}

/// SPEC-091 IW2: typed fleet embedding index (entity/relationship/report).
#[cfg(feature = "postgres")]
pub fn resolve_fleet_embedding_index(
    pool: Option<sqlx::PgPool>,
) -> Option<Arc<dyn edgequake_storage::traits::FleetEmbeddingIndex>> {
    pool.map(|pool| {
        let model = std::env::var("EDGEQUAKE_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-small".to_string());
        Arc::new(edgequake_storage::PgFleetEmbeddingIndex::new(pool, model))
            as Arc<dyn edgequake_storage::traits::FleetEmbeddingIndex>
    })
}

/// Non-postgres builds have no relational spine; the authority flag defaults
/// to `kv`, so this is never reached in practice.
#[cfg(not(feature = "postgres"))]
pub fn resolve_relational_chunk_repo(_pool: Option<&()>) -> Option<Arc<dyn ChunkRepository>> {
    None
}

#[cfg(feature = "postgres")]
fn relational_chunk_pool(state: &AppState) -> Option<&sqlx::PgPool> {
    state.pg_pool.as_ref()
}

#[cfg(not(feature = "postgres"))]
fn relational_chunk_pool(_state: &AppState) -> Option<&'static ()> {
    None
}

/// Persist pipeline output via `IngestionPersister` and invalidate query result cache.
pub async fn persist_ingestion_result(
    state: &AppState,
    graph_storage: Arc<dyn GraphStorage>,
    vector_storage: Arc<dyn VectorStorage>,
    relational_sink: Arc<dyn RelationalEntitySink>,
    params: PersistIngestionParams<'_>,
) -> Result<IngestionPersistOutput, edgequake_pipeline::error::PipelineError> {
    persist_with_providers_progress_and_embedder(
        Arc::clone(&state.query.llm_provider),
        Some(state.query.engine_impl.as_ref() as &dyn QueryResultCacheInvalidator),
        graph_storage,
        vector_storage,
        Arc::clone(&state.storage.kv_storage),
        relational_sink,
        Arc::new(NoopLineageSink),
        None,
        resolve_relational_chunk_repo(relational_chunk_pool(state)),
        #[cfg(feature = "postgres")]
        state.pg_pool.clone(),
        params,
        None,
    )
    .await
}

/// Same as [`persist_ingestion_result`] but accepts explicit LLM + cache invalidator (worker processor).
pub async fn persist_with_providers(
    llm_provider: Arc<dyn LLMProvider>,
    cache_invalidator: Option<&dyn QueryResultCacheInvalidator>,
    graph_storage: Arc<dyn GraphStorage>,
    vector_storage: Arc<dyn VectorStorage>,
    kv_storage: Arc<dyn KVStorage>,
    relational_sink: Arc<dyn RelationalEntitySink>,
    params: PersistIngestionParams<'_>,
) -> Result<IngestionPersistOutput, edgequake_pipeline::error::PipelineError> {
    persist_with_providers_and_progress(
        llm_provider,
        cache_invalidator,
        graph_storage,
        vector_storage,
        kv_storage,
        relational_sink,
        Arc::new(NoopLineageSink),
        params,
        None,
    )
    .await
}

/// Full variant: accepts an optional merge progress callback and lineage sink (SPEC-032 W-04/W-08).
#[allow(clippy::too_many_arguments)]
pub async fn persist_with_providers_and_progress(
    llm_provider: Arc<dyn LLMProvider>,
    cache_invalidator: Option<&dyn QueryResultCacheInvalidator>,
    graph_storage: Arc<dyn GraphStorage>,
    vector_storage: Arc<dyn VectorStorage>,
    kv_storage: Arc<dyn KVStorage>,
    relational_sink: Arc<dyn RelationalEntitySink>,
    lineage_sink: Arc<dyn LineageSink>,
    params: PersistIngestionParams<'_>,
    merge_progress: Option<MergeProgressCallback>,
) -> Result<IngestionPersistOutput, edgequake_pipeline::error::PipelineError> {
    persist_with_providers_progress_and_embedder(
        llm_provider,
        cache_invalidator,
        graph_storage,
        vector_storage,
        kv_storage,
        relational_sink,
        lineage_sink,
        None,
        None,
        #[cfg(feature = "postgres")]
        None, // no pool in this wrapper: typed hook is a no-op (processor passes pool)
        params,
        merge_progress,
    )
    .await
}

/// Persist with optional text embedder for community_report vectors (SPEC-046).
///
/// SPEC-091: pass `relational_chunks` (from [`resolve_relational_chunk_repo`]) so
/// `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=dual|relational` can write the spine.
#[allow(clippy::too_many_arguments)]
pub async fn persist_with_providers_progress_and_embedder(
    llm_provider: Arc<dyn LLMProvider>,
    cache_invalidator: Option<&dyn QueryResultCacheInvalidator>,
    graph_storage: Arc<dyn GraphStorage>,
    vector_storage: Arc<dyn VectorStorage>,
    kv_storage: Arc<dyn KVStorage>,
    relational_sink: Arc<dyn RelationalEntitySink>,
    lineage_sink: Arc<dyn LineageSink>,
    text_embedder: Option<Arc<dyn edgequake_storage::TextEmbedder>>,
    relational_chunks: Option<Arc<dyn ChunkRepository>>,
    #[cfg(feature = "postgres")] typed_embedding_pool: Option<sqlx::PgPool>,
    params: PersistIngestionParams<'_>,
    merge_progress: Option<MergeProgressCallback>,
) -> Result<IngestionPersistOutput, edgequake_pipeline::error::PipelineError> {
    let workspace_id = params.workspace_id.clone();
    let ctx = IngestionPersistContext::new(
        params.document_id,
        params.tenant_id,
        Some(workspace_id.clone()),
    )
    .with_source_metadata(
        params.source_type.map(str::to_string),
        params.source_file_path.map(str::to_string),
    );

    let mut persister = DefaultIngestionPersister::from_settings(
        graph_storage,
        vector_storage,
        IngestionPersistSettings::default(),
        relational_sink,
        Some(llm_provider),
        Some(kv_storage),
    )
    .with_lineage_sink(lineage_sink)
    .with_relational_chunks(relational_chunks);

    // SPEC-091 W3/IW2: typed chunk + fleet embedding writes. Under
    // `typed_embeddings` the legacy `eq_*_vectors` adapter is write-stopped;
    // these hooks are the SSOT (fail-closed when typed).
    #[cfg(feature = "postgres")]
    {
        let typed_index = resolve_typed_embedding_index(typed_embedding_pool.clone());
        persister = persister.with_typed_embedding_index(typed_index);
        let fleet_index = resolve_fleet_embedding_index(typed_embedding_pool.clone());
        persister = persister.with_fleet_embedding_index(fleet_index);
        // SPEC-091 IP2: transactional outbox (LAW-D3) — same pool as typed writers.
        if let Some(pool) = typed_embedding_pool {
            let outbox: Arc<dyn edgequake_storage::OutboxSink> =
                Arc::new(edgequake_storage::PostgresOutboxSink::new(pool));
            persister = persister.with_outbox(Some(outbox));
        }
    }

    if let Some(embedder) = text_embedder {
        persister = persister.with_text_embedder(embedder);
    }

    if let Some(cb) = merge_progress {
        persister = persister.with_merge_progress(cb);
    }

    let out = persister
        .persist(&ctx, params.result, params.chunk_options)
        .await?;

    if let Some(invalidator) = cache_invalidator {
        invalidator.invalidate_query_result_cache_for_workspace(&workspace_id);
    }

    Ok(out)
}

/// Build KV chunk records for a processed document (outside persister scope).
pub fn build_chunk_kv_records(
    document_id: &str,
    filename: &str,
    result: &ProcessingResult,
) -> Vec<(String, serde_json::Value)> {
    edgequake_pipeline::build_chunk_kv_records(document_id, Some(filename), result)
}
