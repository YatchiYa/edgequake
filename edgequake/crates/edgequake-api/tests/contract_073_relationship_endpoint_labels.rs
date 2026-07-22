//! 073 — Relationship endpoint labels + remaining opaque display bypasses.

#[test]
fn contract_retrieved_relationship_has_endpoint_labels() {
    let src = include_str!("../../edgequake-query/src/context.rs");
    assert!(
        src.contains("pub source_label: String"),
        "RetrievedRelationship must expose source_label"
    );
    assert!(
        src.contains("pub target_label: String"),
        "RetrievedRelationship must expose target_label"
    );
    assert!(
        src.contains("fn display_source"),
        "RetrievedRelationship must expose display_source helper"
    );
}

#[test]
fn contract_query_helpers_apply_endpoint_labels() {
    let src = include_str!("../../edgequake-query/src/helpers.rs");
    assert!(
        src.contains("fn apply_relationship_endpoint_labels"),
        "helpers must apply endpoint labels"
    );
    assert!(
        src.contains("fn resolve_relationship_endpoint_labels"),
        "helpers must batch-resolve endpoint labels from graph"
    );
    assert!(
        src.contains("resolve_entity_display_label"),
        "endpoint labels must use presentation SSOT"
    );
}

#[test]
fn contract_local_global_resolve_endpoint_labels() {
    let local = include_str!("../../edgequake-query/src/engine_impl/modes/local.rs");
    let global = include_str!("../../edgequake-query/src/engine_impl/modes/global.rs");
    for (name, src) in [("local", local), ("global", global)] {
        assert!(
            src.contains("resolve_relationship_endpoint_labels"),
            "{name} mode must resolve relationship endpoint labels"
        );
    }
}

#[test]
fn contract_context_format_uses_display_labels() {
    let src = include_str!("../../edgequake-query/src/context_format.rs");
    assert!(
        src.contains("display_source") && src.contains("display_target"),
        "format_relationship_line must use display labels"
    );
}

#[test]
fn contract_context_relationship_passes_labels() {
    let types = include_str!("../src/handlers/context_types.rs");
    assert!(types.contains("pub source_label: String"));
    assert!(types.contains("pub target_label: String"));
    let mapper = include_str!("../src/services/context_bundle_mapper.rs");
    assert!(
        mapper.contains("source_label:") && mapper.contains("display_source"),
        "context mapper must pass source_label from display_source"
    );
}

#[test]
fn contract_traversal_start_node_uses_graph_node_label() {
    let src = include_str!("../src/handlers/graph/graph_query/traversal.rs");
    assert!(
        !src.contains("label: n.id.clone()"),
        "traversal must not set label from n.id"
    );
    assert!(
        src.contains("graph_node_label"),
        "traversal must use graph_node_label SSOT"
    );
}

#[test]
fn contract_neighborhood_has_label() {
    let types = include_str!("../src/handlers/entities_types.rs");
    assert!(
        types.contains("pub label: String"),
        "NeighborhoodNode must expose label"
    );
    let svc = include_str!("../src/services/entity_neighborhood.rs");
    assert!(
        svc.contains("graph_node_label"),
        "neighborhood builder must call graph_node_label"
    );
}

#[test]
fn contract_chunk_detail_and_provenance_soft_label() {
    let chunk = include_str!("../src/handlers/lineage/chunk_detail.rs");
    assert!(
        chunk.contains("graph_node_label"),
        "chunk_detail must soft-label entities"
    );
    let prov = include_str!("../src/handlers/lineage/entity_provenance.rs");
    assert!(
        prov.contains("graph_node_label"),
        "entity_provenance must soft-label entity_name"
    );
}

#[test]
fn contract_forbids_label_from_id_aliases() {
    let traversal = include_str!("../src/handlers/graph/graph_query/traversal.rs");
    assert!(
        !traversal.contains("label: n.id") && !traversal.contains("label: node.id"),
        "no label: *.id in traversal"
    );
}
