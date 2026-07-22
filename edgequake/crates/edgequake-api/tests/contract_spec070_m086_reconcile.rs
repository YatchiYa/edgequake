//! SPEC-070 — M086 every-boot BFS index reconcile wiring.

#[test]
fn contract_m086_reconcile_wired() {
    let bootstrap = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(
        bootstrap.contains("SQL_086_APPLY") && bootstrap.contains("reconcile_migration_086"),
        "migration bootstrap must reconcile M086 every boot"
    );
    let apply = include_str!("../../../migrations/support/086/apply.sql");
    assert!(
        apply.contains("idx_edge_source_id")
            && apply.contains("idx_edge_target_id")
            && apply.contains("statement_timeout = 0"),
        "M086 support SQL must create BFS indexes with DDL-safe timeouts"
    );
    let reconcile = include_str!("../src/state/migration_bootstrap/reconcile/m086.rs");
    assert!(
        reconcile.contains("reconcile_migration_086"),
        "m086 reconcile module must export reconcile_migration_086"
    );
}
