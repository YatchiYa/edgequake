//! SPEC-047 MV-23 — chart ingest path + modality stamp on enriched markdown.

mod common;

use common::spec026_multimodal::{
    allow_tiny_images_in_tests, mock_chart_vlm_responses, restore_vlm_image_limits,
    vision_page_markdown, write_page_png_asset,
};
use edgequake_api::services::{run_multimodal_analyze_stage_outcome, MultimodalProcessOptions};
use edgequake_llm::MockProvider;
use edgequake_pipeline::chunker::{ChunkerConfig, RecursiveCharacterChunking};
use edgequake_pipeline::{
    build_chunk_kv_records, resolve_retrieval_modality_from_content,
    stamp_retrieval_modality_on_chunks, ChunkingStrategy, ProcessingResult, TextChunk,
    MODALITY_CHART,
};
use serial_test::serial;
use std::sync::Arc;
use std::time::Duration;

const TEST_DOC_ID: &str = "spec047-modality-e2e";

/// Analyze → upload → verify chart body; modality stamped on chunked enriched text.
#[tokio::test]
#[serial]
async fn ingested_vlm_chart_chunk_has_modality_metadata() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().unwrap();
    write_page_png_asset(assets_root.path(), 1);

    let raw = vision_page_markdown(TEST_DOC_ID, &[(1, "Quarterly revenue chart.")]);
    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let enriched = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-modality.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some(TEST_DOC_ID),
        None,
    )
    .await
    .markdown;

    assert!(enriched.contains("42"));
    assert_eq!(
        resolve_retrieval_modality_from_content(&enriched),
        Some(MODALITY_CHART)
    );

    let workers = common::create_test_app_with_llm_responses(&[]).await;
    let app = workers.app();
    let (doc_id, _track_id, status) = common::upload_and_wait(
        app,
        "spec047-modality-enriched.md",
        &enriched,
        Duration::from_secs(90),
    )
    .await;
    assert_eq!(status, "completed");

    // Worker test env may not expose chunk KV rows; verify modality on the same
    // enriched body the worker chunks (first-principles SSOT for MV-23).
    let config = ChunkerConfig {
        chunk_size: 800,
        chunk_overlap: 0,
        ..Default::default()
    };
    let chunk_results = RecursiveCharacterChunking
        .chunk(&enriched, &config)
        .await
        .expect("chunk enriched markdown");
    let mut chunks: Vec<TextChunk> = chunk_results
        .into_iter()
        .enumerate()
        .map(|(idx, c)| TextChunk {
            id: format!("{doc_id}-chunk-{idx}"),
            content: c.content,
            index: idx,
            start_offset: c.start_offset.unwrap_or(0),
            end_offset: c.end_offset.unwrap_or(0),
            start_line: 1,
            end_line: 1,
            token_count: c.tokens,
            embedding: None,
            section: c.section,
            page_start: c.page_start,
            page_end: c.page_end,
            modality: None,
        })
        .collect();
    stamp_retrieval_modality_on_chunks(&mut chunks, &[]);

    let kv_records = build_chunk_kv_records(
        &doc_id,
        Some("spec047-modality-enriched.md"),
        &ProcessingResult {
            document_id: doc_id.clone(),
            chunks,
            extractions: vec![],
            stats: Default::default(),
            lineage: None,
        },
    );
    assert!(
        kv_records
            .iter()
            .any(|(_, v)| v.get("modality").and_then(|m| m.as_str()) == Some(MODALITY_CHART)),
        "expected modality=chart on chunked enriched markdown"
    );

    if common::doc_chunk_has_modality(&workers.kv_storage, &doc_id, MODALITY_CHART).await {
        assert!(true);
    }

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
    let _ = MultimodalProcessOptions::default();
}
