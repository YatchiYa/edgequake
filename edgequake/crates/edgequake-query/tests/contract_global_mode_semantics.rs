//! SPEC-023 I2 / SPEC-046 EQ-046-11 — global mode documentation matches implementation.

#[test]
fn contract_global_mode_docs_do_not_claim_ms_graphrag_reports() {
    let src = include_str!("../src/modes.rs");
    assert!(
        !src.contains("Community-based search using graph clusters"),
        "Global mode rustdoc must not claim GraphRAG community clusters"
    );
    assert!(
        src.contains("Not") && src.contains("GraphRAG"),
        "Global mode must explicitly disclaim MS GraphRAG hierarchical community reports"
    );
    // SPEC-046: optional extractive reports are allowed behind env flag
    assert!(
        src.contains("EDGEQUAKE_COMMUNITY_REPORTS") || src.contains("community_report"),
        "Global mode docs should mention optional extractive community_report path"
    );
}
