//! SPEC-058 Wave 6 — dimension mismatch fail-closed (no silent DROP).

#[test]
fn contract_ensure_dimension_fail_closed_by_default() {
    let src = include_str!("../src/adapters/postgres/vector/migration.rs");
    assert!(
        src.contains("EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD"),
        "ensure_dimension must gate DROP behind allow flag"
    );
    assert!(
        src.contains("Refusing DROP TABLE"),
        "mismatch without flag must error"
    );
    assert!(
        src.contains("allow_vector_table_rebuild"),
        "helper must exist"
    );
}

#[test]
fn contract_allow_vector_table_rebuild_parses_truthy() {
    // Source-level contract: helper + env name are present (env mutation races
    // with parallel tests — avoid set_var here).
    let src = include_str!("../src/adapters/postgres/vector/migration.rs");
    assert!(src.contains(r#""1" | "true" | "yes" | "on""#));
}
