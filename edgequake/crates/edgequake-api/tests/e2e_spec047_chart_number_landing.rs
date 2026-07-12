//! SPEC-047 / 015 — chart number-landing contracts (Pass A prompt + specialize path).

mod common;

use common::spec026_multimodal::{
    allow_tiny_images_in_tests, mock_chart_vlm_responses, mock_figure_caption_chart_vlm_responses,
    restore_vlm_image_limits, vision_page_markdown, write_figure_png_asset, write_page_png_asset,
};
use edgequake_api::services::{
    collect_mm_chunks_from_manifest, run_multimodal_analyze_stage_outcome, MultimodalProcessOptions,
};
use edgequake_llm::MockProvider;
use edgequake_pdf::{scan_inline_image_refs, RAG_PAGE_VISION_SYSTEM_PROMPT};
use serial_test::serial;
use std::sync::Arc;

#[test]
fn pass_a_rag_prompt_requires_chart_number_dump() {
    assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("CHARTS / PLOTS"));
    assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("EVERY readable data point"));
    assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("never invent"));
    assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("| Category / X |"));
}

/// Chart specialize dumps key_values + data_table_md into searchable chunk text.
#[tokio::test]
#[serial]
async fn chart_specialize_numbers_land_in_chunk_text() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_page_png_asset(assets_root.path(), 1);

    let raw = vision_page_markdown("spec047-chart-numbers", &[(1, "Quarterly revenue chart.")]);
    assert_eq!(scan_inline_image_refs(&raw).len(), 1);

    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-chart-numbers.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some("spec047-chart-numbers"),
        None,
    )
    .await;

    assert!(
        outcome.summary.success >= 1,
        "expected VLM success, got {:?}",
        outcome.summary
    );
    assert!(
        outcome.markdown.contains("42"),
        "key_values must land: {}",
        outcome.markdown
    );
    assert!(
        outcome.markdown.contains("| Q4 | 42 |") || outcome.markdown.contains("Data table"),
        "data_table_md must land: {}",
        outcome.markdown
    );

    let mm_opts = MultimodalProcessOptions {
        images: true,
        ..Default::default()
    };
    let mm_chunks = collect_mm_chunks_from_manifest(&outcome.manifest, &mm_opts).unwrap();
    assert_eq!(mm_chunks.len(), 1);
    assert!(mm_chunks[0].text.starts_with("[Chart Name]"));
    assert!(mm_chunks[0].text.contains("42"));

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
}

/// Caption says chart but classify returns Illustration → still chart-specialize (context route).
#[tokio::test]
#[serial]
async fn figure_caption_routes_to_chart_specialize() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_page_png_asset(assets_root.path(), 2);

    let raw = vision_page_markdown(
        "spec047-fig-chart-route",
        &[(2, "Figure 3. Revenue by quarter (%) bar chart.")],
    );

    let mock = Arc::new(MockProvider::new());
    mock_figure_caption_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-fig-chart-route.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some("spec047-fig-chart-route"),
        None,
    )
    .await;

    assert!(
        outcome.summary.success >= 1,
        "expected VLM success, got {:?}",
        outcome.summary
    );
    assert!(
        outcome.markdown.contains("42"),
        "context-routed chart specialize must land numbers: {}",
        outcome.markdown
    );

    let mm_opts = MultimodalProcessOptions {
        images: true,
        ..Default::default()
    };
    let mm_chunks = collect_mm_chunks_from_manifest(&outcome.manifest, &mm_opts).unwrap();
    assert_eq!(mm_chunks.len(), 1);
    assert!(
        mm_chunks[0].text.starts_with("[Chart Name]"),
        "mis-routed Illustration must be coerced to Chart chunk: {}",
        mm_chunks[0].text
    );
    assert!(mm_chunks[0].text.contains("42"));

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
}

/// Multi-panel Figure 1 (Illustration classify + performance caption) → chart numbers in markdown.
#[tokio::test]
#[serial]
async fn multi_panel_figure_caption_chart_numbers_land_in_markdown() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_figure_png_asset(assets_root.path(), 1);

    let body = "Figure 1. The impact of three data compositions on model performance across capability dimensions. Starting from the 10T-token corpus.";
    let raw = vision_page_markdown("spec047-multi-panel", &[(1, body)]);

    let mock = Arc::new(MockProvider::new());
    common::spec026_multimodal::mock_multi_panel_figure_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("i"),
        "math_2605.19762v1.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some("spec047-multi-panel"),
        None,
    )
    .await;

    assert!(
        outcome.summary.success >= 1,
        "expected VLM success, got {:?}",
        outcome.summary
    );
    assert!(
        outcome.markdown.contains("52"),
        "multi-panel key_values must land in markdown: {}",
        outcome.markdown
    );
    assert!(
        outcome.markdown.contains("### Vision analysis"),
        "vision analysis block expected: {}",
        outcome.markdown
    );
    assert!(
        outcome.markdown.contains("w/o code") || outcome.markdown.contains("full data"),
        "series labels must land: {}",
        outcome.markdown
    );

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
}
