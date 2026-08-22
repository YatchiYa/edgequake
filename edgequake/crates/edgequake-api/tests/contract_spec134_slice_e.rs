//! SPEC-134 Slice E behavioral contracts: Pass-A pixels forwarded, page-as-unit
//! assemble, EdgeParse veto, mixed stitch (not `include_str!` folklore).

use edgequake_pdf::vision_markdown::{
    assemble_vision_markdown_with_figures, assemble_vision_markdown_with_policy,
    stitch_page_markdown_in_order, VisionPageSlice,
};
use edgequake_pdf::{
    PageClassResult, PageClassification, PageConvertPlan, PageModality,
    EMPTY_VISION_PAGE_PLACEHOLDER,
};
use std::collections::HashMap;

fn prod_src() -> &'static str {
    include_str!("../src/processor/pdf_processing.rs")
}

fn vision_src() -> &'static str {
    include_str!("../../edgequake-pdf/src/backend/vision.rs")
}

#[test]
fn pass_a_forwards_max_rendered_pixels_to_pdf2md() {
    let src = vision_src();
    assert!(
        src.contains("builder = builder.max_rendered_pixels(max_px)"),
        "Pass-A must forward max_rendered_pixels into pdf2md ConversionConfig"
    );
}

#[test]
fn empty_pass_a_does_not_inject_fig_hrefs() {
    let pages = vec![VisionPageSlice {
        page_num: 1,
        markdown: String::new(),
    }];
    let mut figs = HashMap::new();
    figs.insert(
        1,
        vec![edgequake_pdf::WrittenFigureAsset {
            page_num: 1,
            index: 1,
            rel_path: "assets/page-0001-fig-01.png".into(),
            width: 40,
            height: 30,
            bbox: None,
        }],
    );
    let md = assemble_vision_markdown_with_figures(
        &pages,
        true,
        true,
        Some("doc"),
        None,
        Some(&figs),
        None,
    );
    assert!(md.contains(EMPTY_VISION_PAGE_PLACEHOLDER));
    assert!(
        !md.contains("page-0001-fig-01.png"),
        "empty Pass-A must not grow crop galleries: {md}"
    );
}

#[test]
fn manuscript_page_as_unit_suppresses_figures_with_text() {
    let pages = vec![VisionPageSlice {
        page_num: 1,
        markdown: "Résultats".into(),
    }];
    let mut figs = HashMap::new();
    figs.insert(
        1,
        vec![edgequake_pdf::WrittenFigureAsset {
            page_num: 1,
            index: 1,
            rel_path: "assets/page-0001-fig-01.png".into(),
            width: 40,
            height: 30,
            bbox: None,
        }],
    );
    let md = assemble_vision_markdown_with_policy(
        &pages,
        true,
        true,
        Some("ms"),
        None,
        Some(&figs),
        None,
        true,
    );
    assert!(md.contains("Résultats"));
    assert!(!md.contains("fig-01.png"));
}

#[test]
fn mixed_plan_print_pages_do_not_use_manuscript_prompt_group() {
    let plan = PageConvertPlan::from_classifications(
        &[
            PageClassification {
                page_num: 1,
                result: PageClassResult {
                    modality: PageModality::Manuscript,
                    score: 0.9,
                },
            },
            PageClassification {
                page_num: 2,
                result: PageClassResult {
                    modality: PageModality::Print,
                    score: 0.9,
                },
            },
            PageClassification {
                page_num: 3,
                result: PageClassResult {
                    modality: PageModality::Print,
                    score: 0.9,
                },
            },
        ],
        3,
    );
    let groups = plan.groups();
    assert_eq!(groups[0].0, PageModality::Print);
    assert_eq!(groups[0].1.as_deref(), Some(&[2, 3][..]));
    assert_eq!(groups[1].0, PageModality::Manuscript);
    assert_eq!(groups[1].1.as_deref(), Some(&[1][..]));
    let print_prompt = edgequake_pdf::pass_a_system_prompt_for(groups[0].0);
    let ms_prompt = edgequake_pdf::pass_a_system_prompt_for(groups[1].0);
    assert!(
        print_prompt.contains("Write all output in English") || print_prompt.contains("English")
    );
    assert!(
        !ms_prompt.contains("Write ALL output in English")
            && !ms_prompt.contains("Write all output in English")
    );
}

#[test]
fn stitch_mixed_groups_preserves_page_order() {
    let print = "<!-- edgequake-page:2 -->\n\nPrint Acc body\n".to_string();
    let ms = "<!-- edgequake-page:1 -->\n\nManuscript body\n".to_string();
    let out = stitch_page_markdown_in_order(&[print, ms]);
    assert!(out.find("Manuscript body").unwrap() < out.find("Print Acc body").unwrap());
}

#[test]
fn production_path_grouped_convert_and_edgeparse_veto() {
    let src = prod_src();
    assert!(src.contains("conversion_config_for_group("));
    assert!(src.contains("stitch_page_markdown_in_order"));
    assert!(src.contains("should_skip_edgeparse"));
    let classify = src.find("classify_pages_from_bytes").unwrap();
    let edge = src.find("try_edgeparse_fast_path").unwrap();
    assert!(classify < edge);
}

#[test]
fn vision_skips_caption_reinject_on_manuscript() {
    let src = vision_src();
    assert!(src.contains("if !page_as_unit"));
    assert!(src.contains("plan.write_charts && !page_as_unit"));
}

#[test]
fn print_group_keeps_figure_filter_and_print_prompt() {
    let src = prod_src();
    assert!(
        src.contains("attach_figure_filter_if_enabled"),
        "print convert group must still attach SPEC-049/128 figure filter"
    );
    assert!(
        src.contains("print_figure_filter"),
        "figure-filter provider must be reserved for the print group"
    );
    let e2e = include_str!("e2e_spec134_grounding_verify.rs");
    assert!(
        e2e.contains("print_document_byte_identical_regression_guard"),
        "print verify path must stay byte-identical"
    );
}

#[test]
fn pass_a_ms_pixels_set_on_group_config() {
    let src = prod_src();
    assert!(
        src.contains("vision.max_rendered_pixels = Some(profile.max_rendered_pixels)"),
        "MS convert group must set Pass-A max_rendered_pixels from ManuscriptProfile"
    );
}
