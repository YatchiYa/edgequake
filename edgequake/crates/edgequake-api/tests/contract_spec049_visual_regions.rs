//! SPEC-049 — visual region cascade contract (object-cluster, no invent).

mod common;

use common::spec026_multimodal::TINY_PNG;
use edgequake_pdf::{
    assemble_vision_markdown_with_figures, inject_on_disk_region_assets,
    is_drawing_eligible_asset_rel_path, is_full_page_asset_rel_path, should_write_region_figure,
    VisionPageSlice, WrittenFigureAsset, WrittenTableAsset,
};
use edgequake_pdf2md::{RegionSource, MAX_AREA_FRAC, MIN_AREA_FRAC};
use std::collections::HashMap;

#[test]
fn area_invariants_match_spec049() {
    assert!((MIN_AREA_FRAC - 0.02).abs() < f32::EPSILON);
    assert!((MAX_AREA_FRAC - 0.55).abs() < f32::EPSILON);
}

#[test]
fn g2_full_page_never_drawing() {
    assert!(is_full_page_asset_rel_path("assets/page-0004.png"));
    assert!(!is_drawing_eligible_asset_rel_path("assets/page-0004.png"));
    assert!(is_drawing_eligible_asset_rel_path(
        "assets/page-0004-fig-01.png"
    ));
    assert!(is_drawing_eligible_asset_rel_path(
        "assets/page-0006-table-01.png"
    ));
}

#[test]
fn e4_fig_and_table_same_page_no_chart() {
    let pages = vec![VisionPageSlice {
        page_num: 7,
        markdown: "## Figure 2\n\n## Table 2\n".into(),
    }];
    let mut figs = HashMap::new();
    figs.insert(
        7,
        vec![WrittenFigureAsset {
            page_num: 7,
            index: 1,
            rel_path: "assets/page-0007-fig-01.png".into(),
            width: 40,
            height: 30,
            bbox: None,
        }],
    );
    let mut tables = HashMap::new();
    tables.insert(
        7,
        vec![WrittenTableAsset {
            page_num: 7,
            index: 1,
            rel_path: "assets/page-0007-table-01.png".into(),
            width: 80,
            height: 40,
            label: "Table 2".into(),
        }],
    );
    let mut chart = HashMap::new();
    chart.insert(7usize, "assets/page-0007-chart.png".into());
    let md = assemble_vision_markdown_with_figures(
        &pages,
        true,
        true,
        Some("doc"),
        Some(&chart),
        Some(&figs),
        Some(&tables),
    );
    assert!(md.contains("page-0007-fig-01"));
    assert!(md.contains("page-0007-table-01"));
    assert!(!md.contains("page-0007-chart"));
}

#[test]
fn e9_inject_strips_missing_fig() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("assets")).unwrap();
    std::fs::write(dir.path().join("assets/page-0004-fig-01.png"), TINY_PNG).unwrap();
    let md = "<!-- edgequake-page:3 -->\n![Figure 1](assets/page-0003-fig-01.png)\n\n<!-- edgequake-page:4 -->\n![Figure 1](assets/page-0004-fig-01.png)\n";
    let out = inject_on_disk_region_assets(md, dir.path());
    assert!(!out.contains("page-0003-fig-01"));
    assert!(out.contains("page-0004-fig-01"));
}

#[test]
fn region_source_discriminants_stable() {
    // Ensure public enum stays usable for telemetry.
    assert_ne!(RegionSource::StructTree, RegionSource::ObjectCluster);
}

/// E18 — Form crop beside ImageXObject embed (no any-embed page skip).
#[test]
fn e18_form_figure_written_beside_embed() {
    let existing = [WrittenFigureAsset {
        page_num: 1,
        index: 1,
        rel_path: "assets/page-0001-fig-01.png".into(),
        width: 40,
        height: 30,
        bbox: Some((0.0, 0.0, 50.0, 50.0)),
    }];
    assert!(should_write_region_figure(
        (200.0, 200.0, 320.0, 320.0),
        false,
        true,
        &existing
    ));
    assert!(!should_write_region_figure(
        (5.0, 5.0, 45.0, 45.0),
        false,
        true,
        &existing
    ));
}

/// E20 — chart residual: table pages blocked; fig pages allowed (026 W1-crop-expand).
#[test]
fn e20_chart_residual_candidates_allow_fig_skip_table() {
    use edgequake_pdf::chart_residual_candidate_pages;
    let mut figs = HashMap::new();
    figs.insert(
        1,
        vec![WrittenFigureAsset {
            page_num: 1,
            index: 1,
            rel_path: "assets/page-0001-fig-01.png".into(),
            width: 40,
            height: 30,
            bbox: None,
        }],
    );
    let mut tables: HashMap<usize, Vec<WrittenTableAsset>> = HashMap::new();
    tables.insert(
        2,
        vec![WrittenTableAsset {
            page_num: 2,
            index: 1,
            rel_path: "assets/page-0002-table-01.png".into(),
            width: 10,
            height: 10,
            label: "Table 1".into(),
        }],
    );
    assert_eq!(
        chart_residual_candidate_pages(&[1, 2, 3], &figs, &tables),
        vec![1, 3]
    );
}
