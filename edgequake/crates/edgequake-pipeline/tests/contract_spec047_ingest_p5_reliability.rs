//! SPEC-047 P4/P5 — ingest profile + slim ProcessingResult + chunked merge upserts.

#[test]
fn ingest_profile_exported_and_applied_from_env() {
    let config = include_str!("../src/pipeline/config.rs");
    assert!(
        config.contains("enum IngestProfile"),
        "IngestProfile SSOT required"
    );
    assert!(
        config.contains("EDGEQUAKE_INGEST_PROFILE"),
        "env wiring required for chunk_only / retrieve_only"
    );
    let ingestion = include_str!("../src/ingestion_pipeline.rs");
    assert!(
        ingestion.contains("PipelineConfig::from_env()"),
        "build_ingestion_pipeline must honour from_env (profile + tunables)"
    );
}

#[test]
fn processing_result_strip_embeddings_api() {
    let types = include_str!("../src/pipeline/types.rs");
    assert!(
        types.contains("fn strip_embeddings"),
        "ProcessingResult::strip_embeddings required for slim checkpoints"
    );
    assert!(
        types.contains("fn needs_reembed"),
        "ProcessingResult::needs_reembed required for resume"
    );
    let emb = include_str!("../src/pipeline/helpers/embeddings.rs");
    assert!(
        emb.contains("pub async fn ensure_embeddings"),
        "Pipeline::ensure_embeddings required for slim-checkpoint resume"
    );
}

#[test]
fn merger_upserts_vectors_in_progress_chunks() {
    let merger = include_str!("../src/merger/mod.rs");
    assert!(
        merger.contains("upsert_vectors_chunked"),
        "merger must chunk vector upserts for progress honesty"
    );
    assert!(
        merger.contains("vector_upsert_chunk_size"),
        "must share EDGEQUAKE_VECTOR_UPSERT_CHUNK with storage"
    );
}
