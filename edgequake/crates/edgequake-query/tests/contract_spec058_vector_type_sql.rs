//! SPEC-058 Wave 7 — Local/Global push vector_type to SQL metadata filter.

#[test]
fn contract_local_pushes_entity_vector_type() {
    let src = include_str!("../src/engine_impl/modes/local.rs");
    assert!(
        src.contains(r#"Some("entity")"#),
        "query_local must pass vector_type=entity to make_scope_metadata_filter"
    );
}

#[test]
fn contract_global_pushes_relationship_vector_type() {
    let src = include_str!("../src/engine_impl/modes/global.rs");
    assert!(
        src.contains(r#"Some("relationship")"#),
        "query_global must pass vector_type=relationship to make_scope_metadata_filter"
    );
}

#[test]
fn contract_mix_arm_uses_concurrency_gate() {
    let timed = include_str!("../src/engine_impl/modes/arm_timed.rs");
    let gate = include_str!("../src/engine_impl/modes/arm_concurrency.rs");
    assert!(timed.contains("acquire_arm_permit"));
    assert!(gate.contains("EDGEQUAKE_QUERY_ARM_CONCURRENCY"));
}
