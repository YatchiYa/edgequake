//! SPEC-054 / docs/054 — Query · Postgres · AGE · pgvector performance invariants.
//!
//! Wiring contracts (no live DB). Budgets that need Postgres live in
//! `e2e_spec054_query_perf_smoke.rs`.
//!
//! Cross-ref: `docs/054-fix-bugs-17/`.

#[test]
fn contract_filtered_query_modes_use_query_filtered() {
    let local = include_str!("../../edgequake-query/src/engine_impl/modes/local.rs");
    let global = include_str!("../../edgequake-query/src/engine_impl/modes/global.rs");
    let naive = include_str!("../../edgequake-query/src/engine_impl/modes/naive.rs");
    let chunk = include_str!("../../edgequake-query/src/engine_impl/modes/chunk_retrieval.rs");
    for (name, src) in [
        ("local", local),
        ("global", global),
        ("naive", naive),
        ("chunk_retrieval", chunk),
    ] {
        assert!(
            src.contains("query_filtered"),
            "{name} must use query_filtered for scoped ANN (iterative_scan path)"
        );
    }
}

#[test]
fn contract_search_tuning_enables_iterative_scan_when_filtered() {
    let src = include_str!("../src/adapters/postgres/vector/search_tuning.rs");
    assert!(src.contains("hnsw.iterative_scan"));
    assert!(src.contains("max_scan_tuples"));
    assert!(src.contains("relaxed_order"));
    assert!(src.contains("pgvector_supports_iterative_scan"));
    assert!(
        src.contains("if filtered && iterative_scan_supported"),
        "iterative_scan must be gated on filtered + version support"
    );
}

#[test]
fn contract_vector_count_uses_stats_not_raw_scan_first() {
    let src = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    assert!(
        src.contains("SELECT row_count FROM"),
        "count() must prefer stats table (QUERY_CATALOG VEC-03 mitigated)"
    );
    assert!(
        src.contains("ensure_row_count_stats"),
        "stats self-heal must exist"
    );
}

#[test]
fn contract_native_upsert_targets_unique_index_names() {
    let nodes = include_str!("../src/adapters/postgres/graph/nodes_ops.rs");
    let edges = include_str!("../src/adapters/postgres/graph/edges_ops.rs");
    assert!(nodes.contains("idx_node_prop_node_id_unique"));
    assert!(edges.contains("idx_edge_source_target_unique") || edges.contains("source_id"));
    assert!(nodes.contains("pg_upsert_nodes_batch_native"));
    assert!(edges.contains("pg_upsert_edges_batch_native"));
}

#[test]
fn contract_bootstrap_skips_dedup_when_unique_valid() {
    let lifecycle =
        include_str!("../src/adapters/postgres/graph/helpers/graph_lifecycle.rs");
    assert!(
        lifecycle.contains("index_validity"),
        "must probe pg_index.indisvalid before O(N) dedup"
    );
    assert!(
        lifecycle.contains("already valid — skip"),
        "valid UNIQUE must short-circuit dedup/create"
    );
    assert!(
        lifecycle.contains("dedup_nodes_for_unique_index"),
        "dedup remains for missing/INVALID index path"
    );
}

#[test]
fn contract_m083_support_skips_when_unique_exists() {
    let apply = include_str!("../../../migrations/support/083/apply.sql");
    assert!(
        apply.contains("already exists"),
        "M083 boot SSOT must skip O(N) work when UNIQUE exists"
    );
    assert!(
        apply.contains("skip dedup"),
        "M083 must document skip dedup/ANALYZE fast path"
    );
    assert!(
        apply.contains("CHECKSUM SAFETY") || apply.contains("checksum"),
        "M083 support must warn not to edit locked sqlx migration"
    );
    // Frozen sqlx migration must remain present (checksum-locked). Do not require
    // byte-identity with support/ — boot SSOT may diverge for fast-path.
    let locked = include_str!("../../../migrations/083_age_native_unique_index_reconcile.sql");
    assert!(locked.contains("idx_node_prop_node_id_unique"));
}

#[test]
fn contract_docs_054_pack_exists() {
    let readme = include_str!("../../../../docs/054-fix-bugs-17/README.md");
    assert!(readme.contains("First Principles"));
    assert!(readme.contains("pgvector"));
    let fp = include_str!("../../../../docs/054-fix-bugs-17/001-first-principles.md");
    assert!(fp.contains("iterative_scan"));
    assert!(fp.contains("O(N)"));
}

#[test]
fn contract_native_graph_writes_default_on() {
    let src = include_str!("../src/adapters/postgres/graph/mod.rs");
    assert!(
        src.contains("Unset → enabled") || src.contains("Err(_) => true"),
        "native_graph_writes must default ON (docs/054 best performance)"
    );
    assert!(
        src.contains("\"0\" | \"false\" | \"off\" | \"no\""),
        "must support explicit opt-out"
    );
}
