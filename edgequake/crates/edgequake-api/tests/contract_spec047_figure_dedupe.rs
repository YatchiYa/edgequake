//! SPEC-047 — figure viewer image dedupe contract (Pass A + inject must not duplicate).

use edgequake_pdf::{
    assemble_vision_markdown_with_figures, count_markdown_images_for_asset,
    page_figure_asset_rel_path, scan_inline_image_refs, VisionPageSlice, WrittenFigureAsset,
};
use std::collections::HashMap;

/// Reproduces COLLEAGUE-style layout: Pass A image above "Figure N" caption line.
#[test]
fn contract_figure_not_duplicated_when_pass_a_image_precedes_caption() {
    let rel = page_figure_asset_rel_path(3, 1);
    let pages = vec![VisionPageSlice {
        page_num: 3,
        markdown: format!(
            "# System Overview\n\n![Pipeline diagram](fig1.png)\n\nFigure 1: Architecture overview.\n"
        ),
    }];
    let mut figs = HashMap::new();
    figs.insert(
        3,
        vec![WrittenFigureAsset {
            page_num: 3,
            index: 1,
            rel_path: rel.clone(),
            width: 320,
            height: 160,
            bbox: None,
        }],
    );
    let md = assemble_vision_markdown_with_figures(
        &pages,
        true,
        true,
        Some("spec047-dedupe"),
        None,
        Some(&figs),
        None,
    );
    assert_eq!(
        count_markdown_images_for_asset(&md, &rel),
        1,
        "expected one viewer image for {rel}: {md}"
    );
    assert_eq!(scan_inline_image_refs(&md).len(), 1);
    assert!(md.contains("<drawing"));
}
