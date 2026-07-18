//! SPEC-075 — iterative_scan / max_scan_tuples GUC contract (no database required).
//!
//! Filtered HNSW: iterative_scan + max_scan_tuples on.
//! Unfiltered HNSW: ef_search only — no iterative_scan (pgvector guidance).

#![cfg(feature = "postgres")]

use edgequake_storage::{parse_hnsw_iterative_scan_mode, PgVectorStorage, VectorIndexType};

#[test]
fn iterative_scan_mode_parsing_ssot() {
    assert_eq!(parse_hnsw_iterative_scan_mode(""), "relaxed_order");
    assert_eq!(parse_hnsw_iterative_scan_mode("strict_order"), "strict_order");
    assert_eq!(parse_hnsw_iterative_scan_mode("off"), "off");
}

#[test]
fn filtered_hnsw_emits_iterative_scan_and_max_scan_tuples() {
    let stmts = PgVectorStorage::search_tuning_statements_with_hnsw_mode(
        VectorIndexType::HNSW,
        10,
        true,
        true,
        "relaxed_order",
    );
    assert!(
        stmts
            .iter()
            .any(|s| s == "SET LOCAL hnsw.iterative_scan = relaxed_order"),
        "filtered must enable iterative_scan: {stmts:?}"
    );
    assert!(
        stmts
            .iter()
            .any(|s| s.starts_with("SET LOCAL hnsw.max_scan_tuples =")),
        "filtered must bound max_scan_tuples: {stmts:?}"
    );
}

#[test]
fn unfiltered_hnsw_does_not_enable_iterative_scan() {
    let stmts = PgVectorStorage::search_tuning_statements(VectorIndexType::HNSW, 10, false, true);
    assert!(
        !stmts.iter().any(|s| s.contains("iterative_scan")),
        "unfiltered must not set iterative_scan: {stmts:?}"
    );
    assert!(
        !stmts.iter().any(|s| s.contains("max_scan_tuples")),
        "unfiltered must not set max_scan_tuples: {stmts:?}"
    );
    assert!(stmts.iter().any(|s| s.contains("hnsw.ef_search")));
}
