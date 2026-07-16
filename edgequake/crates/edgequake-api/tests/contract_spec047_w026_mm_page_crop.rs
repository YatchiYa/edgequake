//! SPEC-047 / 026 W2-mm-page + W1-crop-telemetry + W1-fig-as-chart contract tests.
//!
//! Ensures multimodal sidecar chunks stamp page markers from asset paths so
//! page-aware chunking does not inherit the document's last page (FP: page
//! attribution). Also locks crop-coverage comment shape for bench fidelity.
//! W1-fig-as-chart: ink-empty alongside pages promote fig→chart so coexist
//! can emit a specialize target (political / full-bleed charts).

use edgequake_api::services::multimodal::build_sidecar_block;
use edgequake_api::services::{
    append_mm_chunks_to_text, collect_mm_chunks_from_manifest, ManifestItem, MultimodalChunk,
    MultimodalHeading, MultimodalItemRecord, MultimodalManifest, MultimodalProcessOptions,
};
use edgequake_pdf::{
    page_num_from_asset_rel_path, promote_fig_as_chart_when_ink_empty, CropCoverageReport,
    WrittenFigureAsset,
};
use edgequake_pipeline::chunker::{
    make_page_marker, ChunkerConfig, ChunkingStrategy, PageAwareChunking,
};
use std::collections::HashMap;

#[tokio::test]
async fn mm_sidecar_page_marker_survives_page_aware_chunking() {
    let record = MultimodalItemRecord::success_image(
        "im-3",
        "revenue_chart".into(),
        "Chart".into(),
        "Key value Q4=42 on panel Average.".into(),
    );
    let manifest = MultimodalManifest {
        version: 1,
        items: vec![ManifestItem {
            item_id: "im-3".into(),
            modality: "drawing".into(),
            start: 0,
            end: 0,
            matched: String::new(),
            asset_path: Some("assets/page-0003-chart.png".into()),
            mime_type: Some("image/png".into()),
            body: None,
            caption: None,
            footnote: None,
            footnotes: Vec::new(),
            block_id: None,
            heading: Some(MultimodalHeading {
                level: 0,
                heading: "Charts".into(),
                parent_headings: Vec::new(),
            }),
            analyze_result: Some(record),
        }],
    };
    let opts = MultimodalProcessOptions {
        images: true,
        ..Default::default()
    };
    let chunks = collect_mm_chunks_from_manifest(&manifest, &opts).unwrap();
    assert_eq!(chunks[0].page_start, Some(3));
    assert_eq!(
        page_num_from_asset_rel_path("assets/page-0003-chart.png"),
        Some(3)
    );

    // Document body ends on page 9 — without stamp, mm text would inherit page 9.
    let body = format!(
        "{}\nIntro.\n\n{}\nLast page prose.\n",
        make_page_marker(1),
        make_page_marker(9)
    );
    let enriched = append_mm_chunks_to_text(&body, &chunks);
    assert!(enriched.contains("<!-- edgequake-page:3 -->"));

    let chunker = PageAwareChunking::default();
    let results = chunker
        .chunk(&enriched, &ChunkerConfig::default())
        .await
        .unwrap();
    let mm = results
        .iter()
        .find(|c| c.content.contains("Q4=42"))
        .expect("mm chunk content indexed");
    assert_eq!(
        mm.page_start,
        Some(3),
        "mm sidecar must not inherit last page (got {:?})",
        mm.page_start
    );
}

#[test]
fn crop_coverage_comment_is_machine_parseable() {
    let mut figs = HashMap::new();
    figs.insert(
        1usize,
        vec![WrittenFigureAsset {
            page_num: 1,
            index: 1,
            rel_path: "assets/page-0001-fig-01.png".into(),
            width: 10,
            height: 10,
            bbox: None,
        }],
    );
    let tables = HashMap::new();
    let report = CropCoverageReport::from_pages(&[1, 2, 3], &figs, &tables)
        .with_ink_filter_count(2)
        .with_crops_written(2);
    let comment = report.to_html_comment();
    assert!(comment.starts_with("<!-- edgequake-crop-coverage:"));
    assert!(comment.contains("total_pages=3"));
    assert!(comment.contains("pages_with_fig=1"));
    // W1-crop-expand: fig page is a residual candidate → 3 candidates, 0 table skips
    assert!(comment.contains("residual_candidates=3"));
    assert!(comment.contains("residual_alongside_fig=1"));
    assert!(comment.contains("residual_skipped_due_to_fig_or_table=0"));
    assert!(comment.contains("residual_crops_written=2"));
}

#[test]
fn append_without_page_still_works() {
    let chunk = MultimodalChunk {
        item_id: "im-x".into(),
        modality: "drawing".into(),
        text: "[Chart Name]x\n\nhello".into(),
        sidecar: build_sidecar_block("drawing", "im-x"),
        heading: None,
        llm_cache_list: vec![],
        chunk_order_index: 0,
        page_start: None,
    };
    let out = append_mm_chunks_to_text("body", &[chunk]);
    assert!(out.contains("hello"));
    assert!(!out.contains("<!-- edgequake-page:"));
}

#[test]
fn fig_as_chart_promotion_stamps_chart_asset_page_for_mm_sidecar() {
    // Contract: promote fig→chart → asset path page stamp → mm chunk page_start.
    let dir = tempfile::tempdir().unwrap();
    let assets = dir.path().join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::write(assets.join("page-0008-fig-01.png"), b"\x89PNG-fig8").unwrap();
    let promoted = promote_fig_as_chart_when_ink_empty(dir.path(), &[8usize], &HashMap::new());
    let chart_rel = promoted.get(&8).expect("page 8 promoted");
    assert_eq!(chart_rel, "assets/page-0008-chart.png");
    assert_eq!(page_num_from_asset_rel_path(chart_rel), Some(8));

    let record = MultimodalItemRecord::success_image(
        "im-8-chart",
        "error_taxonomy".into(),
        "Chart".into(),
        "Perceptual Error; Lack of Knowledge; Reasoning Error".into(),
    );
    let manifest = MultimodalManifest {
        version: 1,
        items: vec![ManifestItem {
            item_id: "im-8-chart".into(),
            modality: "drawing".into(),
            start: 0,
            end: 0,
            matched: String::new(),
            asset_path: Some(chart_rel.clone()),
            mime_type: Some("image/png".into()),
            body: None,
            caption: None,
            footnote: None,
            footnotes: Vec::new(),
            block_id: None,
            heading: Some(MultimodalHeading {
                level: 0,
                heading: "Charts".into(),
                parent_headings: Vec::new(),
            }),
            analyze_result: Some(record),
        }],
    };
    let opts = MultimodalProcessOptions {
        images: true,
        ..Default::default()
    };
    let chunks = collect_mm_chunks_from_manifest(&manifest, &opts).unwrap();
    assert_eq!(chunks[0].page_start, Some(8));
}
