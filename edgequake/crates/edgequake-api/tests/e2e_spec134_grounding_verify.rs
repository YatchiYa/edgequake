//! SPEC-134 Slice B E2E (mock): grounding verify pass over a multi-page
//! manuscript-class conversion — high page passes untouched, low page is
//! refined then honestly marked, judge failure fails open.

use std::sync::Arc;

use edgequake_api::services::manuscript_verify::{
    verify_manuscript_markdown, GROUNDING_LOW_MARKER_PREFIX,
};
use edgequake_llm::MockProvider;
use edgequake_pdf::PageModality;

/// Write a tiny valid PNG so the verifier finds pixels on disk.
fn write_tiny_png(root: &std::path::Path, page_num: usize) {
    let dir = root.join("assets");
    std::fs::create_dir_all(&dir).unwrap();
    const PNG: [u8; 68] = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x0d, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x62,
        0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    std::fs::write(dir.join(format!("page-{page_num:04}.png")), PNG).unwrap();
}

fn manuscript_markdown() -> String {
    "<!-- edgequake-page:1 -->\n\n# Notes terrain\n\nContenu lisible.\n\n\
     <!-- edgequake-page:2 -->\n\n# Histogramme de traction\n\nPassage entierement invente.\n\n\
     <!-- edgequake-page:3 -->\n\nContenu de la page trois.\n"
        .to_string()
}

#[tokio::test]
async fn manuscript_verify_end_to_end_mixed_verdicts() {
    let dir = std::env::temp_dir().join(format!("e2e-mv-{}", uuid::Uuid::new_v4()));
    for page in 1..=3 {
        write_tiny_png(&dir, page);
    }
    let mock = MockProvider::new();
    // Page 1: grounded — accepted untouched.
    mock.add_response(r#"{"grounded_score": 0.95, "invented": [], "missing": []}"#)
        .await;
    // Page 2: confabulated — judge low, refine, re-judge still low → marked.
    mock.add_response(
        r#"{"grounded_score": 0.1, "invented": ["Histogramme de traction"], "missing": ["notes fatigue"]}"#,
    )
    .await;
    mock.add_response("# Notes fatigue\n\nTranscription corrigee.")
        .await;
    mock.add_response(r#"{"grounded_score": 0.4, "invented": [], "missing": ["details"]}"#)
        .await;
    // Page 3: judge returns garbage — retry also fails → fail open, original kept.
    mock.add_response("I cannot score this.").await;
    mock.add_response("still cannot score this.").await;

    let input = manuscript_markdown();
    let out = verify_manuscript_markdown(
        &input,
        PageModality::Manuscript,
        Some(dir.as_path()),
        Arc::new(mock),
    )
    .await;

    assert!(out.ran, "verify must run for manuscript modality");
    assert_eq!(out.pages_judged, 2, "page 3 fail-open is not a judgment");
    assert_eq!(out.pages_low_grounding, 1);
    assert_eq!(out.pages_refined, 1);
    assert!(out.fail_open, "page 3 judge failure must flag fail-open");

    // Page 1 untouched.
    assert!(out.markdown.contains("Contenu lisible."));
    // Page 2 refined + honestly marked.
    assert!(out.markdown.contains("Transcription corrigee."));
    assert!(
        out.markdown
            .contains(&format!("{GROUNDING_LOW_MARKER_PREFIX} score=0.40 -->")),
        "still-low page must carry the honesty marker:\n{}",
        out.markdown
    );
    // Page 3 original kept (fail-open).
    assert!(out.markdown.contains("Contenu de la page trois."));
    // Page markers all preserved.
    for page in 1..=3 {
        assert!(out
            .markdown
            .contains(&format!("<!-- edgequake-page:{page} -->")));
    }
    // Mean over judged pages: (0.95 + 0.40) / 2 = 0.675.
    let mean = out.mean_score.expect("two pages scored");
    assert!((mean - 0.675).abs() < 1e-4, "mean {mean}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn quarantine_lane_excludes_marked_pages_from_index_text() {
    // P0 belief gate: the display markdown keeps honestly-marked low-grounding
    // content; the index-bound text must not. Measured leak (2026-08-20): a
    // `grounding:low score=0.00` page still produced TRACTION_TEST_RESULTS.
    let dir = std::env::temp_dir().join(format!("e2e-qg-{}", uuid::Uuid::new_v4()));
    for page in 1..=2 {
        write_tiny_png(&dir, page);
    }
    let mock = MockProvider::new();
    // Page 1: grounded. Page 2: low → refine → still low → marked.
    mock.add_response(r#"{"grounded_score": 0.95, "invented": [], "missing": []}"#)
        .await;
    mock.add_response(r#"{"grounded_score": 0.0, "invented": ["all"], "missing": []}"#)
        .await;
    mock.add_response("Histogramme de traction invente").await;
    mock.add_response(r#"{"grounded_score": 0.0, "invented": ["all"], "missing": []}"#)
        .await;

    let input = "<!-- edgequake-page:1 -->\n\n# Notes terrain\n\nContenu lisible.\n\n\
                 <!-- edgequake-page:2 -->\n\n# Histogramme de traction\n\nPassage invente.\n";
    let out = verify_manuscript_markdown(
        input,
        PageModality::Manuscript,
        Some(dir.as_path()),
        Arc::new(mock),
    )
    .await;
    assert!(out.markdown.contains(GROUNDING_LOW_MARKER_PREFIX));

    let index_text =
        edgequake_api::services::manuscript_verify::strip_low_grounding_sections(&out.markdown);
    assert!(
        index_text.contains("Contenu lisible."),
        "grounded page must remain in the index text"
    );
    assert!(
        !index_text.contains("Histogramme de traction invente"),
        "quarantined page content must not reach chunking/extraction:\n{index_text}"
    );
    assert!(
        index_text.contains("<!-- edgequake-page:2 -->"),
        "page marker provenance is preserved"
    );
    // Display copy is untouched by the quarantine lane.
    assert!(out.markdown.contains("Histogramme de traction invente"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn print_document_byte_identical_regression_guard() {
    // Print documents must pass through byte-identically — the verify pass is
    // a manuscript-class cost gate, never a print-path mutation.
    let mock = MockProvider::new();
    let input = manuscript_markdown();
    let out = verify_manuscript_markdown(&input, PageModality::Print, None, Arc::new(mock)).await;
    assert!(!out.ran);
    assert_eq!(out.markdown, input, "print path must be byte-identical");
}
