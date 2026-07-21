//! 066 — Drawing entity display_name: Graph API must not use opaque node.id as label
//! when display_name / mm description can resolve a human label.

#[test]
fn contract_graph_handlers_use_graph_node_label() {
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
        assert!(
            !src.contains("label: node.id.clone()") && !src.contains("label: node.id,"),
            "graph handler must not assign label = node.id directly"
        );
    }
}

#[test]
fn contract_graph_label_module_exists() {
    let src = include_str!("../src/handlers/graph/graph_label.rs");
    assert!(src.contains("fn graph_node_label"));
    assert!(src.contains("resolve_mm_display_from_node_props"));
}
