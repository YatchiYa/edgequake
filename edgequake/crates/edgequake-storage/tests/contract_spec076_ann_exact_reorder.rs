//! SPEC-076 A3 — ANN exact reorder policy + SQL contract (no database).

#![cfg(feature = "postgres")]

use edgequake_storage::{
    build_ann_select_sql, AnnExactReorderPolicy, DEFAULT_ANN_REORDER_CANDIDATE_K,
};

#[test]
fn default_exact_reorder_is_off() {
    std::env::remove_var("EDGEQUAKE_ANN_EXACT_REORDER");
    std::env::remove_var("EDGEQUAKE_ANN_REORDER_CANDIDATE_K");
    let p = AnnExactReorderPolicy::from_env();
    assert!(!p.enabled, "silent flip forbidden — default OFF");
    assert_eq!(p.candidate_k, DEFAULT_ANN_REORDER_CANDIDATE_K);
}

#[test]
fn filtered_sql_preserves_workspace_predicate_when_reorder_on() {
    let p = AnnExactReorderPolicy {
        enabled: true,
        candidate_k: 50,
    };
    let sql = build_ann_select_sql(
        "eq_ns_vectors",
        "halfvec",
        "WHERE workspace_id = $2 AND tenant_id = $3",
        4,
        20,
        &p,
    );
    assert!(
        sql.contains("workspace_id = $2"),
        "reorder must keep filter columns inside CTE: {sql}"
    );
    assert!(sql.contains("WITH candidates AS MATERIALIZED"));
    assert!(sql.contains("ORDER BY distance + 0"));
    assert!(sql.contains("LIMIT 50"));
    assert!(sql.contains("LIMIT $4"));
}

#[test]
fn reorder_off_sql_matches_single_stage_shape() {
    let p = AnnExactReorderPolicy::default();
    let sql = build_ann_select_sql("eq_ns_vectors", "vector", "", 2, 20, &p);
    assert!(
        !sql.contains("MATERIALIZED"),
        "default path must stay single-stage: {sql}"
    );
    assert!(sql.contains("ORDER BY embedding <=> $1::vector"));
    assert!(sql.contains("LIMIT $2"));
}

#[test]
fn storage_impl_wires_build_ann_select_sql() {
    let src = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    assert!(
        src.contains("build_ann_select_sql"),
        "query paths must use shared SQL builder"
    );
    assert!(
        src.contains("AnnExactReorderPolicy::from_env"),
        "query paths must read reorder policy from env"
    );
    assert!(
        src.contains("tune_k"),
        "ef_search must tune against candidate_k when reorder on"
    );
}
