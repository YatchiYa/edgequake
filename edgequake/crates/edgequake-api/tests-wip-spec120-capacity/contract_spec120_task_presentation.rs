//! SPEC-120 / INV-Q4 — list enrich attaches task OperationPresentation.

#[test]
fn iss120_document_summary_has_presentation_field() {
    let listing = include_str!("../src/handlers/documents_types/listing.rs");
    assert!(
        listing.contains("presentation: Option") || listing.contains("pub presentation:"),
        "DocumentSummary must expose OperationPresentation for active-run UI"
    );
}

#[test]
fn iss120_enrich_wires_operation_presentation() {
    let mapper = include_str!("../src/services/ingestion_status_mapper.rs");
    assert!(
        mapper.contains("operation_presentation::operation_presentation"),
        "enrich must attach Lens-8 operation_presentation for in-flight tasks"
    );
    assert!(
        mapper.contains("summary.presentation = Some"),
        "enrich must set summary.presentation"
    );
}

#[test]
fn iss120_cancel_decision_soft_fail_persist_ssot() {
    let worker = include_str!("../../edgequake-tasks/src/worker.rs");
    assert!(
        worker.contains("CancelDecision::soft_fail_late_success_persist")
            || worker.contains("soft_fail_late_success_persist"),
        "persist path must use CancelDecision soft-fail SSOT"
    );
    let decision = include_str!("../../edgequake-tasks/src/cancel_decision.rs");
    assert!(
        decision.contains("fn soft_fail_late_success_persist"),
        "CancelDecision must own late Indexed persist soft-fail"
    );
}
