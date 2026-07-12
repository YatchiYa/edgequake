//! SPEC-047 — first-principles PDF asset identity edge cases (contract).
//!
//! Guarantees:
//! - full-page PNG is never Drawing-eligible
//! - fig / table / chart paths classify correctly
//! - assemble never invents missing assets
//! - inject strips missing on-disk hrefs

mod common;

use common::spec026_multimodal::TINY_PNG;
use edgequake_pdf::{
    assemble_vision_markdown_with_figures, inject_on_disk_region_assets,
    is_drawing_eligible_asset_rel_path, is_full_page_asset_rel_path, page_table_asset_rel_path,
    VisionPageSlice, WrittenTableAsset,
};
use edgequake_storage::{
    classify_mm_asset_path, ASSET_KIND_EMBEDDED_FIGURE, ASSET_KIND_PAGE_CHART_CROP,
    ASSET_KIND_PAGE_FULL, ASSET_KIND_TABLE_CROP,
};
use std::collections::HashMap;

#[test]
fn identity_matrix_drawing_eligibility() {
    assert!(is_full_page_asset_rel_path("assets/page-0001.png"));
    assert!(!is_drawing_eligible_asset_rel_path("assets/page-0001.png"));

    assert!(is_drawing_eligible_asset_rel_path(
        "assets/page-0004-fig-01.png"
    ));
    assert!(is_drawing_eligible_asset_rel_path(
        "assets/page-0006-table-01.png"
    ));
    assert!(is_drawing_eligible_asset_rel_path(
        "assets/page-0002-chart.png"
    ));
    assert_eq!(
        page_table_asset_rel_path(6, 1),
        "assets/page-0006-table-01.png"
    );
}

#[test]
fn classify_all_asset_kinds() {
    let cases = [
        ("assets/page-0001.png", ASSET_KIND_PAGE_FULL, Some(1)),
        (
            "assets/page-0004-fig-01.png",
            ASSET_KIND_EMBEDDED_FIGURE,
            Some(4),
        ),
        (
            "assets/page-0006-table-01.png",
            ASSET_KIND_TABLE_CROP,
            Some(6),
        ),
        (
            "assets/page-0002-chart.png",
            ASSET_KIND_PAGE_CHART_CROP,
            Some(2),
        ),
    ];
    for (path, kind, page) in cases {
        let (got_kind, got_page) = classify_mm_asset_path(path);
        assert_eq!(got_kind, kind, "path={path}");
        assert_eq!(got_page, page, "path={path}");
    }
}

#[test]
fn assemble_never_emits_full_page_as_viewer_or_drawing() {
    let pages = vec![
        VisionPageSlice {
            page_num: 1,
            markdown: "Text only prose.".into(),
        },
        VisionPageSlice {
            page_num: 2,
            markdown: String::new(),
        },
    ];
    let md =
        assemble_vision_markdown_with_figures(&pages, true, true, Some("doc"), None, None, None);
    assert!(!md.contains("assets/page-0001.png"));
    assert!(!md.contains("assets/page-0002.png"));
    assert!(!md.contains("<drawing"));
}

#[test]
fn inject_multi_page_strips_only_missing_assets() {
    let dir = tempfile::tempdir().unwrap();
    let assets = dir.path().join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::write(assets.join("page-0004-fig-01.png"), TINY_PNG).unwrap();
    let md = "<!-- edgequake-page:3 -->\n![Figure 1](assets/page-0003-fig-01.png)\n\nA\n\n<!-- edgequake-page:4 -->\n![Figure 1](assets/page-0004-fig-01.png)\n\nB\n";
    let out = inject_on_disk_region_assets(md, dir.path());
    assert!(!out.contains("page-0003-fig-01"));
    assert!(out.contains("page-0004-fig-01"));
    assert!(out.contains('A'));
    assert!(out.contains('B'));
}

#[test]
fn assemble_with_empty_tables_map_is_safe() {
    let pages = vec![VisionPageSlice {
        page_num: 1,
        markdown: "hello".into(),
    }];
    let empty_tables: HashMap<usize, Vec<WrittenTableAsset>> = HashMap::new();
    let md = assemble_vision_markdown_with_figures(
        &pages,
        true,
        true,
        Some("doc"),
        None,
        None,
        Some(&empty_tables),
    );
    assert!(md.contains("hello"));
    assert!(!md.contains("!["));
}
