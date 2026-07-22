//! SPEC-069 — cascade progress heartbeats + boot-owned M092 reconcile wiring.

#[test]
fn contract_cascade_emits_progress_ticks() {
    let cascade = include_str!("../src/services/document_graph_cascade.rs");
    assert!(
        cascade.contains("cascade_remove_document_sources_with_progress"),
        "cascade must expose progress callback variant"
    );
    assert!(
        cascade.contains("on_progress") && cascade.contains("items_total"),
        "cascade must report processed/total after discovery"
    );

    let deletion = include_str!("../src/services/document_deletion.rs");
    assert!(
        deletion.contains("cascade_remove_document_sources_with_progress"),
        "document_deletion must use progress-aware cascade"
    );
    assert!(
        deletion.contains("Duration::from_secs(3)") || deletion.contains("from_secs(3)"),
        "deletion must heartbeat during long graph phase"
    );
    assert!(
        deletion.contains("DeletionPhaseKind::RemovingGraph"),
        "heartbeats must re-emit RemovingGraph phase"
    );
}

#[test]
fn contract_m092_reconcile_wired() {
    let bootstrap = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(
        bootstrap.contains("SQL_092_APPLY") && bootstrap.contains("reconcile_migration_092"),
        "migration bootstrap must reconcile M092 every boot"
    );
    let apply = include_str!("../../../migrations/support/092/apply.sql");
    assert!(
        apply.contains("eq_node_id") && apply.contains("statement_timeout = 0"),
        "M092 support SQL must add eq_* with DDL-safe timeouts"
    );
}
