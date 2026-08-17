//! SPEC-060 Wave 4 — native graph writes default ON; Cypher MERGE is debug opt-out.

#[test]
fn contract_native_graph_writes_default_on() {
    let src = include_str!("../src/adapters/postgres/graph/mod.rs");
    assert!(
        src.contains("fn native_graph_writes_enabled"),
        "native_graph_writes_enabled must exist"
    );
    // Unset → enabled (Err(_) => true)
    assert!(
        src.contains("Err(_) => true") || src.contains("// Unset → enabled"),
        "native graph writes must default ON when env unset"
    );
}

#[test]
fn contract_single_node_upsert_routes_native_when_enabled() {
    let mutate = include_str!("../src/adapters/postgres/graph/nodes_ops/mutate.rs");
    assert!(
        mutate.contains("native_graph_writes_enabled"),
        "pg_upsert_node must check native_graph_writes_enabled (SPEC-059/060)"
    );
    assert!(
        mutate.contains("pg_upsert_nodes_batch"),
        "single-node path must delegate to batch native when enabled"
    );
}

#[test]
fn contract_batch_uses_eq_id_on_conflict() {
<<<<<<< HEAD
    // SPEC-062: denormalized eq_node_id arbiter + full-replace EXCLUDED.properties
    // (Rust always sends complete property maps; skip eq_merge_graph_properties tax).
    let mutate = include_str!("../src/adapters/postgres/graph/nodes_ops/mutate.rs");
    assert!(
        mutate.contains("eq_node_id") && mutate.contains("EXCLUDED.properties"),
        "native node upsert must ON CONFLICT (eq_node_id) SET properties = EXCLUDED.properties"
=======
    // SPEC-062: denormalized eq_* arbiter; SPEC-058 / SPEC-098: merge properties
    // via eq_merge_graph_properties (source_ids union under concurrency).
    let mutate = include_str!("../src/adapters/postgres/graph/nodes_ops/mutate.rs");
    assert!(
        mutate.contains("eq_node_id") && mutate.contains("eq_merge_graph_properties"),
        "native node upsert must ON CONFLICT (eq_node_id) merge via eq_merge_graph_properties"
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    );
    let edges = include_str!("../src/adapters/postgres/graph/edges_ops.rs");
    assert!(
        edges.contains("eq_source_id")
            && edges.contains("eq_target_id")
            && edges.contains("eq_rel_type")
            && edges.contains("ON CONFLICT (eq_source_id, eq_target_id, eq_rel_type)")
<<<<<<< HEAD
            && edges.contains("EXCLUDED.properties"),
        "native edge upsert must JOIN/CONFLICT on eq_source_id/eq_target_id/eq_rel_type (D-30)"
=======
            && edges.contains("eq_merge_graph_properties"),
        "native edge upsert must JOIN/CONFLICT on eq_* (D-30) and merge properties (SPEC-058)"
    );
}

#[test]
fn contract_replace_mode_skips_eq_merge() {
    // SPEC-098 LAW-098-12: cascade prune must SET EXCLUDED.properties, not union.
    let mutate = include_str!("../src/adapters/postgres/graph/nodes_ops/mutate.rs");
    assert!(
        mutate.contains("GraphPropertyWriteMode::Replace")
            && mutate.contains("properties = EXCLUDED.properties"),
        "native node Replace mode must set EXCLUDED.properties"
    );
    let edges = include_str!("../src/adapters/postgres/graph/edges_ops.rs");
    assert!(
        edges.contains("GraphPropertyWriteMode::Replace")
            && edges.contains("properties = EXCLUDED.properties"),
        "native edge Replace mode must set EXCLUDED.properties"
    );
    let trait_src = include_str!("../src/traits/graph_mutate_ops.rs");
    assert!(
        trait_src.contains("enum GraphPropertyWriteMode")
            && trait_src.contains("upsert_nodes_batch_with_mode")
            && trait_src.contains("upsert_edges_batch_with_mode"),
        "trait must expose GraphPropertyWriteMode + with_mode batch APIs"
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    );
}

#[test]
fn contract_drops_legacy_expression_uniques_when_eq_arbiters_exist() {
    // Dual UNIQUE indexes break ON CONFLICT (non-arbiter violations under concurrency).
    let life = include_str!("../src/adapters/postgres/graph/helpers/graph_lifecycle.rs");
    assert!(
        life.contains("DROP INDEX IF EXISTS")
            && life.contains("idx_edge_source_target_unique")
            && life.contains("idx_node_prop_node_id_unique")
            && life.contains("idx_edge_eq_source_target"),
        "ensure_eq_id_columns / bootstrap must drop legacy expression UNIQUEs once eq_* exist"
    );
    assert!(
        life.contains("Dual-unique hazard") || life.contains("dual unique"),
        "document why legacy expression UNIQUEs are dropped"
    );
}

#[test]
fn contract_env_example_documents_native_writes() {
    let env = include_str!("../../../../.env.example");
    assert!(
        env.contains("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
        ".env.example must document native graph writes"
    );
}
