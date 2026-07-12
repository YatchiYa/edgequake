//! SPEC-047 Phase C — vision page `<drawing/>` asset → analyze → ingest E2E (DRY SSOT).

mod common;

use common::spec026_multimodal::{
    allow_tiny_images_in_tests, mock_chart_vlm_responses, restore_vlm_image_limits,
    vision_page_markdown, vision_table_page_markdown, write_figure_png_asset, write_page_png_asset,
    write_table_png_asset,
};
use edgequake_api::services::{
    collect_mm_chunks_from_manifest, run_multimodal_analyze_stage_outcome, MultimodalProcessOptions,
};
use edgequake_llm::MockProvider;
use edgequake_pdf::scan_inline_image_refs;
use serial_test::serial;
use std::sync::Arc;
use std::time::Duration;

const TEST_DOC_ID: &str = "spec047-vision-drawing-e2e";

/// Phase C: vision-style markdown + on-disk page PNG → ite analyze enriches chart key values.
#[tokio::test]
#[serial]
async fn vision_drawing_asset_analyze_stage_enriches_chart() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_page_png_asset(assets_root.path(), 1);
    write_figure_png_asset(assets_root.path(), 1);

    let raw = vision_page_markdown(TEST_DOC_ID, &[(1, "Quarterly revenue summary.")]);
    let refs = scan_inline_image_refs(&raw);
    assert_eq!(refs.len(), 1, "vision markdown must emit one drawing ref");
    assert!(
        refs[0]
            .asset_path
            .as_deref()
            .is_some_and(|p| p.contains("-fig-")),
        "drawing must target embedded figure, got {:?}",
        refs[0].asset_path
    );

    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-chart.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some(TEST_DOC_ID),
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
        "chart specialize key_values must land in markdown: {}",
        outcome.markdown
    );
    assert!(
        outcome.markdown.contains("Q4 Revenue") || outcome.markdown.contains("rev_q4"),
        "chart title/name must land in markdown"
    );
    assert!(
        outcome.markdown.contains("![")
            && outcome.markdown.contains("](assets/page-0001-fig-01.png)"),
        "MV-28: viewer image must be figure crop (never full page): {}",
        outcome.markdown
    );
    assert!(
        !outcome.markdown.contains("](assets/page-0001.png)"),
        "full-page PNG must not appear as viewer/drawing image"
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

/// Table-crop Drawing is first-class: analyze + viewer use `-table-`, never full page.
#[tokio::test]
#[serial]
async fn vision_table_crop_drawing_analyze_stage() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_page_png_asset(assets_root.path(), 6);
    write_table_png_asset(assets_root.path(), 6);

    let raw = vision_table_page_markdown(
        "spec047-table-drawing-e2e",
        6,
        "## Table 1: Pass rates\n\nQuantitative results.",
    );
    let refs = scan_inline_image_refs(&raw);
    assert_eq!(refs.len(), 1, "table markdown must emit one drawing ref");
    assert!(
        refs[0]
            .asset_path
            .as_deref()
            .is_some_and(|p| p.contains("-table-")),
        "drawing must target table crop, got {:?}",
        refs[0].asset_path
    );
    assert!(
        !raw.contains("](assets/page-0006.png)"),
        "full-page must not be viewer image"
    );
    assert!(
        !raw.contains("-chart.png"),
        "chart must not appear when table crop exists"
    );

    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-table.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some("spec047-table-drawing-e2e"),
        None,
    )
    .await;

    assert!(
        outcome.summary.success >= 1,
        "expected VLM success on table crop, got {:?}",
        outcome.summary
    );
    assert!(
        outcome
            .markdown
            .contains("](assets/page-0006-table-01.png)"),
        "viewer image must remain table crop: {}",
        outcome.markdown
    );

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
}

/// Full ingest smoke: enriched Phase C markdown completes worker pipeline.
#[tokio::test]
#[serial]
async fn vision_drawing_enriched_markdown_ingest_completes() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_page_png_asset(assets_root.path(), 1);
    write_figure_png_asset(assets_root.path(), 1);

    let raw = vision_page_markdown(TEST_DOC_ID, &[(1, "See figure below.")]);
    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let enriched = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-ingest.pdf",
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

    let workers = common::create_test_app_with_llm_responses(&[]).await;
    let app = workers.app();
    let (_doc_id, _track_id, status) = common::upload_and_wait(
        app,
        "spec047-vision-drawing-enriched.md",
        &enriched,
        Duration::from_secs(90),
    )
    .await;
    assert_eq!(status, "completed");

    restore_vlm_image_limits();
    std::env::remove_var("EDGEQUAKE_VISION_PROVIDER");
}
