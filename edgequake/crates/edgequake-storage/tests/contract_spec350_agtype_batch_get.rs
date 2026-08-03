//! GH-350 / SPEC-098: merge-path batch SQL must schema-qualify agtype.
//!
//! Pool hygiene pins `search_path TO public`. Unqualified `::agtype` raises
//! `type "agtype" does not exist` when AGE session setup is skipped or drifts.
//! Native writes already use `::ag_catalog.agtype`; batch reads must match.

#[test]
fn contract_spec350_batch_get_nodes_qualifies_agtype() {
    let read = include_str!("../src/adapters/postgres/graph/nodes_ops/read.rs");
    assert!(
        read.contains("::ag_catalog.agtype"),
        "pg_get_nodes_batch SQL must cast via ::ag_catalog.agtype (GH-350)"
    );
    assert!(
        read.contains("pg_get_nodes_batch"),
        "expected pg_get_nodes_batch in read.rs"
    );
    // Bare ::agtype (not part of ::ag_catalog.agtype) is forbidden on this path.
    let stripped = read.replace("::ag_catalog.agtype", "");
    assert!(
        !stripped.contains("::agtype"),
        "unqualified ::agtype remains in nodes_ops/read.rs — GH-350 residual risk"
    );
}

#[test]
fn contract_spec350_native_mutate_also_qualifies_agtype() {
    // DRY: mutate path is the SSOT pattern for schema-qualified casts.
    let mutate = include_str!("../src/adapters/postgres/graph/nodes_ops/mutate.rs");
    assert!(
        mutate.contains("::ag_catalog.agtype"),
        "native node upsert must keep ::ag_catalog.agtype (DRY with GH-350 read harden)"
    );
}
