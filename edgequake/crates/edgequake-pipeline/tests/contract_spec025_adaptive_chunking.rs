//! SPEC-025 5.2 / 5.3 — ingestion pipeline builder parity.

use edgequake_llm::MockProvider;
use edgequake_pipeline::prompts::EntityExtractionSchema;
use edgequake_pipeline::{
    build_chunker_config, build_ingestion_pipeline, calculate_adaptive_chunk_size, ChunkStrategy,
    IngestionPipelineOptions,
};
use serial_test::serial;
use std::sync::Arc;

#[test]
fn adaptive_chunk_thresholds_match_library() {
    assert_eq!(calculate_adaptive_chunk_size(30_000), 1200);
    assert_eq!(calculate_adaptive_chunk_size(80_000), 800);
    assert_eq!(calculate_adaptive_chunk_size(150_000), 600);
}

#[test]
fn ingestion_pipeline_applies_document_size() {
    let llm = Arc::new(MockProvider::new()) as Arc<dyn edgequake_llm::LLMProvider>;
    let embedding =
        Arc::new(MockProvider::new()) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>;

    let pipeline = build_ingestion_pipeline(
        llm,
        embedding,
        EntityExtractionSchema::server_default(),
        IngestionPipelineOptions::from_document_size(120_000),
    );

    // Mock embedding provider caps chunk_size to max_tokens/2; adaptive target is 600.
    assert_eq!(calculate_adaptive_chunk_size(120_000), 600);
    assert!(pipeline.config().chunker.chunk_size <= 600);
    assert!(pipeline.config().chunker.chunk_size > 0);
}

#[test]
#[serial]
fn fixed_chunking_env_overrides_adaptive_for_large_docs() {
    let prev_adaptive = std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING").ok();
    let prev_size = std::env::var("EDGEQUAKE_CHUNK_SIZE").ok();
    let prev_overlap = std::env::var("EDGEQUAKE_CHUNK_OVERLAP").ok();
    unsafe {
        std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "0");
        std::env::set_var("EDGEQUAKE_CHUNK_SIZE", "1200");
        std::env::set_var("EDGEQUAKE_CHUNK_OVERLAP", "100");
    }
    let cfg = build_chunker_config(200_000, ChunkStrategy::Recursive, None);
    assert_eq!(cfg.chunk_size, 1200);
    assert_eq!(cfg.chunk_overlap, 100);
    unsafe {
        match prev_adaptive {
            Some(v) => std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", v),
            None => std::env::remove_var("EDGEQUAKE_ADAPTIVE_CHUNKING"),
        }
        match prev_size {
            Some(v) => std::env::set_var("EDGEQUAKE_CHUNK_SIZE", v),
            None => std::env::remove_var("EDGEQUAKE_CHUNK_SIZE"),
        }
        match prev_overlap {
            Some(v) => std::env::set_var("EDGEQUAKE_CHUNK_OVERLAP", v),
            None => std::env::remove_var("EDGEQUAKE_CHUNK_OVERLAP"),
        }
    }
}

#[test]
#[serial]
fn workspace_fixed_policy_beats_env_adaptive() {
    use edgequake_pipeline::{build_chunker_config_with_policy, ChunkingPolicy};
    let prev = std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING").ok();
    unsafe {
        std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "1");
    }
    let cfg = build_chunker_config_with_policy(
        200_000,
        ChunkStrategy::Recursive,
        Some(&ChunkingPolicy::acc_fair()),
        None,
    );
    assert_eq!(cfg.chunk_size, 1200);
    assert_eq!(cfg.chunk_overlap, 100);
    unsafe {
        match prev {
            Some(v) => std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", v),
            None => std::env::remove_var("EDGEQUAKE_ADAPTIVE_CHUNKING"),
        }
    }
}

#[test]
fn document_chunk_options_win_last_over_workspace_fixed() {
    use edgequake_pipeline::{build_chunker_config_with_policy, ChunkOptions, ChunkingPolicy};
    let opts = ChunkOptions {
        chunk_token_size: Some(500),
        chunk_overlap_token_size: Some(50),
        separators: vec![],
    };
    let cfg = build_chunker_config_with_policy(
        200_000,
        ChunkStrategy::Recursive,
        Some(&ChunkingPolicy::acc_fair()),
        Some(&opts),
    );
    assert_eq!(cfg.chunk_size, 500);
    assert_eq!(cfg.chunk_overlap, 50);
}
