//! SPEC-110: source contracts for migration 118/121 conflict-key dedup + repair.
//!
//! Always-on (no Postgres). Complements e2e_spec110_wsdoc_on_conflict.

#[test]
fn contract_spec110_118_uses_distinct_on_doc_id() {
    let sql = include_str!("../../../migrations/118_spec091_wsdoc_backfill.sql");
    assert!(
        sql.contains("DISTINCT ON (doc_id)"),
        "SPEC-110 LAW-M2: 118 must DISTINCT ON (doc_id)"
    );
    assert!(
        sql.contains("ORDER BY doc_id, ws_id"),
        "SPEC-110: deterministic ORDER BY for DISTINCT ON"
    );
    assert!(
        sql.contains("ON CONFLICT (id) DO UPDATE"),
        "118 must keep ON CONFLICT DO UPDATE"
    );
    assert!(
        !sql.contains("SELECT DISTINCT split_part"),
        "118 must not use pre-SPEC-110 SELECT DISTINCT split_part form"
    );
}

#[test]
fn contract_spec110_121_uses_distinct_on_inj_id() {
    let sql = include_str!("../../../migrations/121_spec091_injection_backfill.sql");
    assert!(
        sql.contains("DISTINCT ON (inj_id)"),
        "SPEC-110 LAW-M2: 121 must DISTINCT ON (inj_id)"
    );
    assert!(
        sql.contains("ORDER BY inj_id, ws_id"),
        "SPEC-110: deterministic ORDER BY for DISTINCT ON"
    );
}

#[test]
fn contract_spec110_checksum_repair_modules_exist() {
    let m118 = include_str!("../src/state/migration_bootstrap/reconcile/m118.rs");
    let m121 = include_str!("../src/state/migration_bootstrap/reconcile/m121.rs");
    for (label, src) in [("m118", m118), ("m121", m121)] {
        assert!(
            src.contains("repair_migration_")
                && src.contains("allow_checksum_repair")
                && src.contains("EDGEQUAKE_DEV_MODE")
                && src.contains("Refusing silent repair"),
            "SPEC-110 / X-02: {label} repair must fail loud without DEV_MODE"
        );
        assert!(
            src.contains("CHECKSUM_BROKEN_V0241") && src.contains("CHECKSUM_FIXED_V0242"),
            "{label} must pin broken→fixed SHA constants"
        );
    }
}

#[test]
fn contract_spec110_bootstrap_wires_118_121_repair() {
    let boot = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(
        boot.contains("repair_migration_118_checksum_if_needed")
            && boot.contains("repair_migration_121_checksum_if_needed"),
        "bootstrap must call M118/M121 checksum repair before sqlx"
    );
    assert!(
        boot.contains("MIGRATION_118_VERSION") && boot.contains("MIGRATION_121_VERSION"),
        "bootstrap must declare version constants"
    );
}
