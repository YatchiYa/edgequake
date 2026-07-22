//! SPEC-070 — delete/cascade hot path must not migrate schema mid-task.

#[test]
fn contract_cascade_and_deletion_avoid_schema_ddl() {
    let cascade = include_str!("../../edgequake-api/src/services/document_graph_cascade.rs");
    let deletion = include_str!("../../edgequake-api/src/services/document_deletion.rs");
    for (name, src) in [("cascade", cascade), ("deletion", deletion)] {
        assert!(
            !src.contains("ensure_eq_id_columns") && !src.contains("ensure_indexes"),
            "{name} must not call graph schema ensure/DDL (boot-owned, SPEC-069/070)"
        );
        assert!(
            !src.contains("ALTER TABLE") && !src.contains("CREATE TRIGGER"),
            "{name} must not embed schema DDL SQL"
        );
    }
}

#[test]
fn contract_discovery_gin_join_still_required() {
    let scan = include_str!("../src/adapters/postgres/graph/scan_ops.rs");
    assert!(
        scan.contains("to_jsonb(pr.probe_id)") || scan.contains("@> to_jsonb"),
        "discovery must use GIN @> probes"
    );
    assert!(
        scan.contains("generate_series") && scan.contains("unnest"),
        "discovery must use unnest + generate_series JOIN shape"
    );
}
