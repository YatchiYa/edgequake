//! SPEC-083 D-30 — M092 boot SSOT + probes require multigraph `eq_rel_type`.
//!
//! Checksum safety: only `migrations/support/092/apply.sql` (and 083 support) are
//! asserted here — never edit versioned `092_*.sql` / `097_*.sql` markers.

#[test]
fn contract_support_092_adds_eq_rel_type_and_rel_unique() {
    let apply = include_str!("../../../migrations/support/092/apply.sql");
    assert!(
        apply.contains("ADD COLUMN IF NOT EXISTS eq_rel_type"),
        "M092 support SSOT must ADD eq_rel_type"
    );
    assert!(
        apply.contains("idx_edge_eq_source_target_rel"),
        "M092 support SSOT must create 3-col unique"
    );
    assert!(
        apply.contains("DROP INDEX IF EXISTS %I.idx_edge_eq_source_target"),
        "M092 support SSOT must drop legacy 2-col unique after _rel"
    );
    assert!(
        apply.contains("NEW.eq_rel_type"),
        "EDGE sync function must set NEW.eq_rel_type"
    );
    assert!(
        apply.contains("CHECKSUM SAFETY") && apply.contains("NOT sqlx-scanned"),
        "document that support/ edits do not break sqlx/flyway checksums"
    );
}

#[test]
fn contract_eq_columns_present_requires_eq_rel_type() {
    let mod_src = include_str!("../src/adapters/postgres/graph/mod.rs");
    let probe = mod_src
        .split("async fn eq_columns_present")
        .nth(1)
        .and_then(|s| s.split("pub fn invalidate_eq_columns_cache").next())
        .expect("eq_columns_present body");
    assert!(
        probe.contains("eq_rel_type"),
        "eq_columns_present must require eq_rel_type (D-30)"
    );
}

#[test]
fn contract_support_083_recognizes_rel_arbiter() {
    let apply = include_str!("../../../migrations/support/083/apply.sql");
    assert!(
        apply.contains("idx_edge_eq_source_target_rel"),
        "M083 must recognize D-30 3-col arbiter so it does not recreate expression UNIQUE"
    );
}
