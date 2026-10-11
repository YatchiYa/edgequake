//! SPEC-164 source contracts. Live proof is `e2e_spec164_plain_graph`.

#[test]
fn close_query_uses_edge_child_not_parent_label() {
    let src = include_str!("../src/adapters/postgres/graph/query_ops/expand.rs");
    assert!(
        src.contains("FROM {}.\"EDGE\" e"),
        "Q-GRAPH-006 must scan the EDGE child table"
    );
    assert!(
        src.contains("eq_source_id") && src.contains("eq_target_id"),
        "Q-GRAPH-006 must filter eq_* endpoints"
    );
    assert!(
        !src.contains("FROM {}._ag_label_edge") && !src.contains("FROM {}.\"_ag_label_edge\""),
        "Q-GRAPH-006 must not scan the parent edge label"
    );
}

#[test]
fn confirm_drop_sql_refuses_a_red_ledger() {
    let sql = include_str!("../../../migrations/171_spec164_drop_age.sql");
    assert!(sql.contains("SPEC-164 confirm-drop refused"));
    assert!(sql.contains("DROP EXTENSION age"));
    assert!(sql.contains("shadow_ok"));
}

#[test]
fn native_writes_opt_out_is_refused_when_plain_tables_exist() {
    let src = include_str!("../src/adapters/postgres/graph/plain/ledger.rs");
    assert!(src.contains(
        "EDGEQUAKE_NATIVE_GRAPH_WRITES=0 is refused while SPEC-164 plain graph tables exist"
    ));
}

#[test]
fn graph_read_flag_cannot_force_plain_reads() {
    let src = include_str!("../src/adapters/postgres/graph/plain/ledger.rs");
    assert!(src.contains("fn graph_read_forced_age"));
    assert!(
        src.contains("if graph_read_forced_age()"),
        "EDGEQUAKE_GRAPH_READ=age pins AGE; no flag selects plain before the ledger is ready"
    );
}

#[test]
fn plain_search_reads_stored_degree_not_every_edge() {
    let src = include_str!("../src/adapters/postgres/graph/plain/search.rs");
    assert!(
        src.contains("n.degree"),
        "popular and search must read the stored degree"
    );
    assert!(
        !src.contains("eq_graph_edges"),
        "search must not aggregate eq_graph_edges on the request path"
    );
    let sql = include_str!("../../../migrations/170_spec164_plain_graph.sql");
    assert!(sql.contains("idx_eq_graph_nodes_degree"));
    assert!(sql.contains("public.gin_trgm_ops"));
    assert!(sql.contains("WITH SCHEMA public"));
    assert!(
        !sql.contains("search_path = public, ag_catalog"),
        "trigram indexes must not depend on the AGE schema"
    );
    assert!(sql.contains("eq_graph_degree_on_insert"));
    assert!(sql.contains("eq_graph_recompute_degrees"));
}

#[test]
fn migration_170_has_no_serving_floor_and_171_raises_it() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v170 = manifest
        .split("version = 170")
        .nth(1)
        .expect("migration 170");
    let v170_head = v170.split("version = 171").next().unwrap();
    assert!(
        !v170_head.contains("serving_floor"),
        "170 must not raise the serving floor (v0.34 can serve schema 170)"
    );
    assert!(manifest.contains("serving_floor = 171"));
    assert!(manifest.contains("irreversible_drop = [125, 126, 131, 171]"));
}
