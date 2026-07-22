//! 067 — Opaque entity names: SSOT reject + Graph API soft-label for legacy UUID nodes.

#[test]
fn contract_entity_id_exports_opaque_detector() {
    let src = include_str!("../../edgequake-storage/src/entity_id.rs");
    assert!(
        src.contains("pub fn is_opaque_identifier"),
        "entity_id must export is_opaque_identifier SSOT"
    );
    assert!(
        src.contains("is_opaque_identifier(trimmed)")
            || src.contains("is_opaque_identifier(&normalized)"),
        "normalize_entity_name must call is_opaque_identifier"
    );
}

#[test]
fn contract_graph_label_soft_labels_opaque() {
    // 072: soft-label SSOT lives in pipeline; API graph_node_label delegates.
    let api = include_str!("../src/handlers/graph/graph_label.rs");
    assert!(api.contains("resolve_entity_display_label"));
    assert!(api.contains("fn graph_node_label"));
    let pipeline = include_str!("../../edgequake-pipeline/src/entity_display.rs");
    assert!(pipeline.contains("fn soft_label_opaque"));
    assert!(pipeline.contains("is_opaque_identifier"));
    assert!(pipeline.contains("Opaque ID"));
}

#[test]
fn contract_graph_handlers_still_use_graph_node_label() {
    let files = [
        include_str!("../src/handlers/graph/graph_stream.rs"),
        include_str!("../src/handlers/graph/graph_query/node.rs"),
        include_str!("../src/handlers/graph/graph_query/search.rs"),
        include_str!("../src/handlers/graph/graph_query/traversal.rs"),
        include_str!("../src/handlers/graph/graph_query/popular.rs"),
    ];
    for src in files {
        assert!(
            src.contains("graph_node_label"),
            "graph handler must call graph_node_label SSOT"
        );
    }
}

#[test]
fn contract_sota_prompt_forbids_opaque_ids() {
    let src = include_str!("../../edgequake-pipeline/src/prompts/entity_extraction.rs");
    assert!(
        src.contains("Opaque identifiers"),
        "SOTA prompt must forbid opaque identifiers as entity_name"
    );
    assert!(src.contains("UUID"));
}
