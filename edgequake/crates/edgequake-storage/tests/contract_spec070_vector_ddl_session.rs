//! SPEC-070 — vector index DDL uses dedicated session GUCs (not query timeout).

#[test]
fn contract_vector_ddl_session_gucs() {
    let ddl = include_str!("../src/adapters/postgres/vector/ddl.rs");
    assert!(
        ddl.contains("setup_vector_ddl_session"),
        "vector ddl must expose setup_vector_ddl_session"
    );
    assert!(
        ddl.contains("SET statement_timeout = 0"),
        "vector DDL must clear statement_timeout"
    );
    assert!(
        ddl.contains("lock_timeout") && ddl.contains("maintenance_work_mem"),
        "vector DDL must set lock_timeout + maintenance_work_mem"
    );
    assert!(
        ddl.contains("EDGEQUAKE_INDEX_MAINTENANCE_WORK_MEM"),
        "maintenance_work_mem must be env-tunable"
    );
    assert!(
        ddl.contains("execute_index_ddl"),
        "ANN/FTS index CREATE must go through execute_index_ddl"
    );
    // Query path must not set maintenance_work_mem (ANN search is SET LOCAL only).
    let tuning = include_str!("../src/adapters/postgres/vector/search_tuning.rs");
    assert!(
        !tuning.contains("maintenance_work_mem"),
        "search_tuning must not raise maintenance_work_mem on query path"
    );
}
