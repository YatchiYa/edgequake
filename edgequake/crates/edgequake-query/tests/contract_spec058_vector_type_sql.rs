//! SPEC-058 Wave 7 / 065 — Local/Global push vector_type to SQL metadata filter.
//!
//! KG ANN uses entity/relationship; chunk hydration must use type=chunk (065).

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
fn contract_065_chunk_fetch_ssot_uses_chunk_type() {
    let src = include_str!("../src/engine_impl/modes/chunk_retrieval.rs");
    assert!(
        src.contains("fn chunk_fetch_metadata_filter"),
        "chunk fetch must own SSOT filter helper"
    );
    assert!(
        src.contains(r#"Some("chunk")"#),
        "chunk_fetch_metadata_filter must set vector_type=chunk"
    );
    // Call sites must not pass entity/relationship mf into append_score_ranked_chunks.
    let local = include_str!("../src/engine_impl/modes/local.rs");
    let global = include_str!("../src/engine_impl/modes/global.rs");
    assert!(
        !local.contains("mf.as_ref(),\n            allowed_document_ids"),
        "local must not reuse entity mf for chunk hydration"
    );
    assert!(
        !global.contains("mf.as_ref(),\n            allowed_document_ids"),
        "global must not reuse relationship mf for chunk hydration"
    );
}

#[test]
fn contract_mix_arm_uses_concurrency_gate() {
    let timed = include_str!("../src/engine_impl/modes/arm_timed.rs");
    let gate = include_str!("../src/engine_impl/modes/arm_concurrency.rs");
    assert!(timed.contains("acquire_arm_permit"));
    assert!(gate.contains("EDGEQUAKE_QUERY_ARM_CONCURRENCY"));
}
