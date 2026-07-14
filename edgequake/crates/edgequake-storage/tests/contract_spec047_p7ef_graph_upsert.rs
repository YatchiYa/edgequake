//! SPEC-047 P7e/P7f contract: extraction reuse plan + graph upsert chunk SSOT.

use edgequake_storage::{
    parse_graph_upsert_chunk, resolve_graph_upsert_chunk, DEFAULT_GRAPH_UPSERT_CHUNK,
};

#[test]
fn contract_p7f_defaults_match_battle_plan() {
    assert_eq!(DEFAULT_GRAPH_UPSERT_CHUNK, 500);
    assert_eq!(parse_graph_upsert_chunk("100"), Some(100));
    assert_eq!(parse_graph_upsert_chunk("10"), Some(50));
    assert_eq!(parse_graph_upsert_chunk("99999"), Some(2000));
}

#[test]
fn contract_p7f_resolve_without_env_clamps_adaptive() {
    assert_eq!(resolve_graph_upsert_chunk(250), 250);
    assert_eq!(resolve_graph_upsert_chunk(10), 50);
    assert_eq!(resolve_graph_upsert_chunk(900), 500);
}

#[test]
fn contract_p7f_native_path_uses_resolve_ssot() {
    let nodes = include_str!("../src/adapters/postgres/graph/nodes_ops.rs");
    let edges = include_str!("../src/adapters/postgres/graph/edges_ops.rs");
    assert!(
        nodes.contains("resolve_graph_upsert_chunk"),
        "node upsert must use P7f resolve_graph_upsert_chunk"
    );
    assert!(
        edges.contains("resolve_graph_upsert_chunk"),
        "edge upsert must use P7f resolve_graph_upsert_chunk"
    );
    assert!(
        nodes.contains("pg_upsert_nodes_batch_native"),
        "native node batch path must exist"
    );
    assert!(
        edges.contains("pg_upsert_edges_batch_native"),
        "native edge batch path must exist"
    );
}
