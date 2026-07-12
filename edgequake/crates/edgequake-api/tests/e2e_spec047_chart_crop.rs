//! SPEC-047 EQ-047-MV-24 — chart-region crop path (Pass A gate + specialize ink-crop).

mod common;

use common::spec026_multimodal::{
    allow_tiny_images_in_tests, mock_chart_vlm_responses, restore_vlm_image_limits,
    write_page_png_asset,
};
use edgequake_api::services::{
    collect_mm_chunks_from_manifest, run_multimodal_analyze_stage_outcome, MultimodalProcessOptions,
};
use edgequake_llm::MockProvider;
use edgequake_pdf::{
    assemble_vision_markdown_with_overrides, page_markdown_suggests_chart, scan_inline_image_refs,
    text_suggests_chart, VisionPageSlice,
};
use serial_test::serial;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn pass_a_chart_gate_is_deterministic() {
    assert!(page_markdown_suggests_chart(
        "Figure 2. Growth chart\n\n| Category / X | Value |\n|---|---|\n| A | 12% |"
    ));
    assert!(!page_markdown_suggests_chart("Cover photo of headquarters"));
    assert!(text_suggests_chart("Page 1 chart"));
}

#[test]
fn assemble_prefers_chart_crop_path_for_drawing() {
    let pages = vec![VisionPageSlice {
        page_num: 2,
        markdown: "Sales plot with 8% YoY".into(),
    }];
    let mut overrides = HashMap::new();
    overrides.insert(2, "assets/page-0002-chart.png".into());
    let md = assemble_vision_markdown_with_overrides(&pages, true, Some("doc"), Some(&overrides));
    let refs = scan_inline_image_refs(&md);
    assert_eq!(refs.len(), 1);
    assert_eq!(
        refs[0].asset_path.as_deref(),
        Some("assets/page-0002-chart.png")
    );
    // MV-26: caption carries Pass A body for specialize routing (not bare "Page 2 chart").
    assert!(
        md.contains(r#"caption="Page 2 chart"#),
        "expected chart caption prefix with Pass A hints: {md}"
    );
    assert!(
        md.contains("![") && md.contains("](assets/page-0002-chart.png)"),
        "MV-28 viewer image missing: {md}"
    );
}

/// E2E: drawing that points at a chart-crop asset still specializes and lands numbers.
#[tokio::test]
#[serial]
async fn chart_crop_asset_specialize_lands_numbers() {
    allow_tiny_images_in_tests();
    std::env::set_var("EDGEQUAKE_VISION_PROVIDER", "__test_no_vision__");

    let assets_root = tempfile::tempdir().expect("temp assets dir");
    write_page_png_asset(assets_root.path(), 1);
    // Simulate MV-24 crop asset (same PNG bytes; path is what analyze resolves).
    let crop_path = assets_root.path().join("assets/page-0001-chart.png");
    std::fs::copy(assets_root.path().join("assets/page-0001.png"), &crop_path)
        .expect("copy chart crop");

    let pages = vec![VisionPageSlice {
        page_num: 1,
        markdown: "Quarterly revenue chart with axis labels.".into(),
    }];
    let mut overrides = HashMap::new();
    overrides.insert(1usize, "assets/page-0001-chart.png".into());
    let raw = assemble_vision_markdown_with_overrides(
        &pages,
        true,
        Some("spec047-mv24"),
        Some(&overrides),
    );
    assert_eq!(scan_inline_image_refs(&raw).len(), 1);
    assert!(raw.contains("page-0001-chart.png"));

    let mock = Arc::new(MockProvider::new());
    mock_chart_vlm_responses(mock.as_ref()).await;

    let outcome = run_multimodal_analyze_stage_outcome(
        raw,
        Some("ite"),
        "spec047-mv24-chart-crop.pdf",
        None,
        uuid::Uuid::nil(),
        mock,
        Some(assets_root.path()),
        Some("spec047-mv24-chart-crop"),
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
        "key_values must land after crop specialize: {}",
        outcome.markdown
    );

    let mm_opts = MultimodalProcessOptions {
        images: true,
        ..Default::default()
    };
    let mm_chunks = collect_mm_chunks_from_manifest(&outcome.manifest, &mm_opts).unwrap();
    assert!(!mm_chunks.is_empty());
    assert!(mm_chunks[0].text.contains("42"));

    restore_vlm_image_limits();
}
