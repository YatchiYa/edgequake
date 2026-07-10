//! Retrieval telemetry metadata keys (SPEC-046 OPS-P2).
//!
//! SOLID: single place for QueryContext metadata keys shared by local/global/sparse
//! and absorbed into [`crate::types::QueryStats`] (same pattern as Mix arm META_*).

use crate::context::QueryContext;

/// Set when local/global falls back to `get_popular_nodes_with_degree`.
pub const META_POPULAR_NODE_FALLBACK: &str = "popular_node_fallback";
/// Arm that triggered popular-node fallback (`local` | `global`).
pub const META_POPULAR_NODE_ARM: &str = "popular_node_arm";
/// Sparse fusion path label (see [`crate::sparse_retrieval::SparseRetrievalOutcome`]).
pub const META_SPARSE_OUTCOME: &str = "sparse_outcome";
/// True when sparse path used in-memory BM25 after Postgres FTS failure/empty.
pub const META_FTS_FALLBACK: &str = "fts_fallback";

/// Mark popular-node fallback on a query context (DRY for local + global).
pub fn mark_popular_node_fallback(ctx: &mut QueryContext, arm: &str) {
    ctx.metadata
        .insert(META_POPULAR_NODE_FALLBACK.into(), serde_json::json!(true));
    ctx.metadata
        .insert(META_POPULAR_NODE_ARM.into(), serde_json::json!(arm));
}

/// Mark sparse / FTS outcome on a query context.
pub fn mark_sparse_outcome(ctx: &mut QueryContext, outcome: &str, fts_fallback: bool) {
    ctx.metadata
        .insert(META_SPARSE_OUTCOME.into(), serde_json::json!(outcome));
    if fts_fallback {
        ctx.metadata
            .insert(META_FTS_FALLBACK.into(), serde_json::json!(true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_popular_node_sets_keys() {
        let mut ctx = QueryContext::new();
        mark_popular_node_fallback(&mut ctx, "local");
        assert_eq!(ctx.metadata.get(META_POPULAR_NODE_FALLBACK).unwrap(), true);
        assert_eq!(ctx.metadata.get(META_POPULAR_NODE_ARM).unwrap(), "local");
    }

    #[test]
    fn mark_sparse_outcome_sets_fallback_flag() {
        let mut ctx = QueryContext::new();
        mark_sparse_outcome(&mut ctx, "fts_error_fallback", true);
        assert_eq!(
            ctx.metadata.get(META_SPARSE_OUTCOME).unwrap(),
            "fts_error_fallback"
        );
        assert_eq!(ctx.metadata.get(META_FTS_FALLBACK).unwrap(), true);
    }
}
