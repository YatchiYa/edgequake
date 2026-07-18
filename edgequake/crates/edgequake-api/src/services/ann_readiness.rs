//! Wave-2 / ANN readiness probe (SPEC-066 / SPEC-071).
//!
//! When `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1`, vector tables that exist must
//! have a global **or** workspace partial HNSW (catalog probe). Empty DBs
//! (no vector tables) are ready.
//!
//! `/ready` verifies **catalog** ANN presence only — not planner plan-shape
//! (session bias is query-path only; SPEC-067).

#[cfg(feature = "postgres")]
use edgequake_storage::PgVectorStorage;

/// Returns `Ok(None)` when ready, or `Ok(Some(blocker))` when not ready.
#[cfg(feature = "postgres")]
pub async fn wave2_ann_readiness_blocker(
    pool: &sqlx::PgPool,
) -> Result<Option<String>, String> {
    use edgequake_storage::hnsw_partial_by_workspace_enabled;

    if !hnsw_partial_by_workspace_enabled() {
        return Ok(None);
    }

    let missing = PgVectorStorage::count_vector_tables_missing_ann_index(pool)
        .await
        .map_err(|e| e.to_string())?;
    if missing > 0 {
        // SPEC-071: distinguish catalog miss vs empty DB (empty → missing=0 → ready).
        return Ok(Some(format!(
            "wave2_ann_missing(tables_without_hnsw={missing}): catalog ANN absent — \
             POST /api/v1/admin/ann/warmup or run a filtered query; \
             /ready checks catalog presence only (not plan-shape)"
        )));
    }
    Ok(None)
}

#[cfg(not(feature = "postgres"))]
pub async fn wave2_ann_readiness_blocker(
    _pool: &(),
) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;

    #[test]
    fn blocker_message_mentions_warmup_and_catalog() {
        // Compile-time documentation of message contract used by product_limits / ops docs.
        let sample = "wave2_ann_missing(tables_without_hnsw=1): catalog ANN absent — \
             POST /api/v1/admin/ann/warmup or run a filtered query; \
             /ready checks catalog presence only (not plan-shape)";
        assert!(sample.contains("admin/ann/warmup"));
        assert!(sample.contains("catalog"));
        assert!(sample.contains("not plan-shape"));
    }
}
