//! SPEC-077 — binary quantize + rerank SQL contract (no database).

#![cfg(feature = "postgres")]

use edgequake_storage::{
    build_binary_hnsw_index_sql, build_binary_rerank_select_sql, BinaryQuantizePolicy,
    DEFAULT_BINARY_CANDIDATE_K,
};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn binary_quantize_env_default_off_and_opt_in() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("EDGEQUAKE_BINARY_QUANTIZE");
    std::env::remove_var("EDGEQUAKE_BINARY_CANDIDATE_K");
    let off = BinaryQuantizePolicy::from_env();
    assert!(!off.enabled, "silent flip forbidden");
    assert_eq!(off.candidate_k, DEFAULT_BINARY_CANDIDATE_K);

    std::env::set_var("EDGEQUAKE_BINARY_QUANTIZE", "1");
    std::env::set_var("EDGEQUAKE_BINARY_CANDIDATE_K", "80");
    let on = BinaryQuantizePolicy::from_env();
    assert!(on.enabled);
    assert_eq!(on.effective_candidate_k(20), 80);
    std::env::remove_var("EDGEQUAKE_BINARY_QUANTIZE");
    std::env::remove_var("EDGEQUAKE_BINARY_CANDIDATE_K");
}

#[test]
fn index_ddl_is_expression_hnsw_bit_hamming() {
    let sql = build_binary_hnsw_index_sql("public.eq_t_vectors", "eq_t_bq_idx", 1536, 16, 64);
    assert!(sql.contains("binary_quantize(embedding)::bit(1536)"));
    assert!(sql.contains("bit_hamming_ops"));
    assert!(sql.contains("CREATE INDEX IF NOT EXISTS"));
}

#[test]
fn query_preserves_workspace_filter_and_exact_rerank() {
    let p = BinaryQuantizePolicy {
        enabled: true,
        candidate_k: 200,
    };
    let sql = build_binary_rerank_select_sql(
        "public.eq_t_vectors",
        "halfvec",
        1536,
        "WHERE workspace_id = $2 AND tenant_id = $3",
        4,
        20,
        &p,
    );
    assert!(sql.contains("workspace_id = $2"), "{sql}");
    assert!(sql.contains("<~>"), "Hamming candidate stage missing: {sql}");
    assert!(
        sql.contains("ORDER BY embedding <=> $1::halfvec"),
        "exact rerank missing: {sql}"
    );
    assert!(sql.contains("LIMIT 200"));
    assert!(sql.contains("LIMIT $4"));
}
