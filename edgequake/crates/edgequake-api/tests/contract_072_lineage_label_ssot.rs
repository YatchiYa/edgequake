//! 072 — Lineage label SSOT: document lineage never surfaces bare UUID labels.

#[test]
fn contract_entity_summary_has_id_and_label_fields() {
    let src = include_str!("../src/handlers/lineage_types/document.rs");
    assert!(
        src.contains("pub id: String"),
        "EntitySummaryResponse must expose graph node id"
    );
    assert!(
        src.contains("pub label: String"),
        "EntitySummaryResponse must expose presentation label"
    );
    assert!(
        src.contains("pub description: Option<String>"),
        "EntitySummaryResponse must expose optional description"
    );
}

#[test]
fn contract_entity_summary_from_node_uses_graph_node_label() {
    let src = include_str!("../src/services/document_graph_lineage.rs");
    assert!(
        src.contains("graph_node_label"),
        "entity_summary_from_node must call graph_node_label SSOT"
    );
    assert!(
        src.contains("is_opaque_identifier"),
        "lineage name must soft-label opaque bare ids"
    );
}

#[test]
fn contract_chunk_link_fallback_soft_labels() {
    let src = include_str!("../src/services/postgres_chunk_lineage.rs");
    assert!(
        src.contains("graph_node_label") || src.contains("soft_label_opaque"),
        "chunk-link fallback must soft-label opaque entities"
    );
    assert!(
        src.contains("label,") || src.contains("label:"),
        "chunk-link fallback must populate label field"
    );
}

#[test]
fn contract_create_entity_rejects_empty_after_opaque_normalize() {
    let src = include_str!("../src/handlers/entities/entity_crud.rs");
    assert!(
        src.contains("entity_name.is_empty()"),
        "create_entity must reject empty/opaque names with 400"
    );
    assert!(
        src.contains("BadRequest"),
        "create_entity empty name must be BadRequest"
    );
}

#[test]
fn contract_search_labels_soft_labels_opaque() {
    let src = include_str!("../src/handlers/graph/graph_query/search.rs");
    assert!(
        src.contains("is_opaque_identifier"),
        "search_labels must detect opaque results"
    );
    assert!(
        src.contains("graph_node_label"),
        "search_labels must soft-label via graph_node_label"
    );
}

#[test]
fn contract_node_to_entity_response_uses_graph_node_label() {
    let src = include_str!("../src/handlers/entities/mod.rs");
    assert!(
        src.contains("graph_node_label"),
        "entity CRUD presentation must use graph_node_label"
    );
}

#[test]
fn contract_opaque_detector_handles_prefixed_uuid() {
    let src = include_str!("../../edgequake-storage/src/entity_id.rs");
    assert!(
        src.contains("opaque_prefixed_or_token_uuid"),
        "detector must reject PREFIX_UUID shapes"
    );
}
