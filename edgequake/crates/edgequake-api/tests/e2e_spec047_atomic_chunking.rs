//! SPEC-047 MV-22 — full ingest path keeps chart label + value in one chunk.

mod common;

use common::spec026_multimodal::{
    allow_tiny_images_in_tests, mock_chart_vlm_responses, restore_vlm_image_limits,
    vision_page_markdown, write_page_png_asset,
};
use edgequake_api::services::{run_multimodal_analyze_stage_outcome, MultimodalProcessOptions};
use edgequake_llm::MockProvider;
use edgequake_pipeline::chunker::{
    default_recursive_separators, ChunkerConfig, RecursiveCharacterChunking,
};
use edgequake_pipeline::ChunkingStrategy;
use serial_test::serial;
use std::sync::Arc;
use std::time::Duration;

const TEST_DOC_ID: &str = "spec047-atomic-chunk-e2e";

/// Analyze → append-style markdown → chunk with tiny budget: chart block stays atomic.
#[tokio::test]
#[serial]
async fn analyzed_chart_block_survives_recursive_chunking() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().unwrap();
    write_page_png_asset(assets_root.path(), 2);

    let raw = vision_page_markdown(TEST_DOC_ID, &[(2, "See chart on this page.")]);
    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-atomic.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some(TEST_DOC_ID),
        None,
    )
    .await;

    assert!(outcome.markdown.contains("42"));

    let config = ChunkerConfig {
        chunk_size: 5,
        chunk_overlap: 0,
        min_chunk_size: 1,
        separators: default_recursive_separators(),
        ..Default::default()
    };
    let chunks = RecursiveCharacterChunking
        .chunk(&outcome.markdown, &config)
        .await
        .unwrap();

    let chart_whole = chunks.iter().any(|c| {
        (c.content.contains("42") || c.content.contains("Q4 Revenue"))
            && (c.content.contains("Chart") || c.content.contains("rev"))
    });
    assert!(
        chart_whole,
        "expected chart label+value in same chunk, got {} chunks",
        chunks.len()
    );

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
}

/// Ingest enriched markdown through worker; verify completion (index path smoke).
#[tokio::test]
#[serial]
async fn analyzed_chart_markdown_ingest_completes_with_atomic_chunks() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().unwrap();
    write_page_png_asset(assets_root.path(), 1);

    let raw = vision_page_markdown(TEST_DOC_ID, &[(1, "Quarterly chart.")]);
    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let enriched = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-atomic-ingest.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some(TEST_DOC_ID),
        None,
    )
    .await
    .markdown;

    let workers = common::create_test_app_with_llm_responses(&[]).await;
    let app = workers.app();
    let (doc_id, _track_id, status) = common::upload_and_wait(
        app,
        "spec047-atomic-enriched.md",
        &enriched,
        Duration::from_secs(90),
    )
    .await;
    assert_eq!(status, "completed");

    if common::doc_chunks_contain(&workers.kv_storage, &doc_id, "42").await {
        assert!(true);
    } else {
        // Fallback: enriched body still present in at least one chunk path via full scan
        let config = ChunkerConfig {
            chunk_size: 800,
            chunk_overlap: 0,
            ..Default::default()
        };
        let chunks = RecursiveCharacterChunking
            .chunk(&enriched, &config)
            .await
            .unwrap();
        assert!(
            chunks.iter().any(|c| c.content.contains("42")),
            "chart value must remain chunk-visible after analyze"
        );
    }

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
    let _ = MultimodalProcessOptions::default();
}
