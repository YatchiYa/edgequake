//! SPEC-081 — serving view SQL contract (migration source; no DB required).

#[test]
fn migration_defines_serving_functions() {
    let sql = include_str!("../../../migrations/093_eq_serving_chunk_presence.sql");
    assert!(
        sql.contains("CREATE OR REPLACE FUNCTION eq_serving_chunk_presence"),
        "missing eq_serving_chunk_presence"
    );
    assert!(
        sql.contains("CREATE OR REPLACE FUNCTION eq_serving_vector_presence"),
        "missing eq_serving_vector_presence"
    );
    assert!(
        sql.contains("Not RAG ANN SSOT") || sql.contains("not RAG ANN"),
        "must document not ANN SSOT"
    );
    assert!(
        sql.contains("FROM public.chunks"),
        "must read relational chunks spine"
    );
}

#[test]
fn first_principles_forbid_silent_unify() {
    let fp = include_str!("../../../../specs/081-serving-view-dual-ssot/001-first-principles.md");
    assert!(fp.contains("dual SSOT") || fp.contains("dual-SSOT") || fp.contains("Dual"));
    assert!(
        fp.to_lowercase().contains("not") && fp.to_lowercase().contains("unify")
            || fp.contains("Do **not** silently unify")
    );
}
