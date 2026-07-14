//! SPEC-047 P1c / P2a — multimodal fail policy + parallel tables/equations (code-is-law).

#[test]
fn mm_fail_policy_ssot_exported() {
    let gates = include_str!("../src/services/multimodal/gates.rs");
    assert!(
        gates.contains("should_abort_multimodal_hard_error"),
        "SSOT abort helper required for PDF + reanalyze"
    );
    assert!(
        gates.contains("should_abort_on_hard_error"),
        "MultimodalFailMode must expose abort predicate"
    );
}

#[test]
fn mm_tables_and_equations_use_buffer_unordered() {
    let analyzer = include_str!("../src/services/multimodal/analyzer.rs");
    assert!(
        analyzer.contains("multimodal table analyze starting (parallel VLM)"),
        "tables must be parallelized"
    );
    assert!(
        analyzer.contains("multimodal equation analyze starting (parallel VLM)"),
        "equations must be parallelized"
    );
    let unordered = analyzer.matches("buffer_unordered").count();
    assert!(
        unordered >= 3,
        "images + tables + equations should each use buffer_unordered (found {unordered})"
    );
}

#[test]
fn clear_document_derived_deletes_vectors() {
    let helpers = include_str!("../src/handlers/pdf_upload/helpers.rs");
    assert!(
        helpers.contains("delete_by_document"),
        "force_reindex path must wipe vectors via delete_by_document (SPEC-047 P1a)"
    );
}
