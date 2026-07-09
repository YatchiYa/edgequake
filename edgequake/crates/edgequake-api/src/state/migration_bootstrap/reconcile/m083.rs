//! Migration 083 — native UNIQUE index reconcile (every bootstrap).
//!
//! First principle: M074/M083 sqlx migrations only cover graphs that exist at
//! apply time. New AGE graphs (workspace clones, tests) need the same UNIQUE
//! indexes for `EDGEQUAKE_NATIVE_GRAPH_WRITES`. Bootstrap re-runs the idempotent
//! SSOT DDL every boot, plus a JSONB→column stats backfill for legacy drift.

use sqlx::PgPool;
use tracing::info;

use super::super::{SQL_083_APPLY, SQL_083_STATS_BACKFILL};
use super::execute_bootstrap_apply_sql;

/// After sqlx: ensure UNIQUE indexes exist on every AGE graph with Node/EDGE.
/// Always runs when AGE is present (idempotent) — not gated on M083 marker so
/// graphs created after migrate still get indexes before the next deploy.
pub async fn reconcile_migration_083(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let age_available: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
            .fetch_one(pool)
            .await
            .unwrap_or(false);

    if !age_available {
        return Ok(false);
    }

    info!(
        target: "edgequake.migration",
        step = "migration_083_apply_start",
        "Reconciling native UNIQUE indexes on all AGE graphs (M083)"
    );
    execute_bootstrap_apply_sql(pool, SQL_083_APPLY).await?;

    // Best-effort: heal relationship_count/entity_count column drift from JSONB.
    if let Err(e) = execute_bootstrap_apply_sql(pool, SQL_083_STATS_BACKFILL).await {
        tracing::warn!(
            target: "edgequake.migration",
            step = "migration_083_stats_backfill",
            error = %e,
            "Stats column backfill failed (non-fatal)"
        );
    } else {
        info!(
            target: "edgequake.migration",
            step = "migration_083_stats_backfill_ok",
            "Document stats columns reconciled from metadata JSONB where drifted"
        );
    }

    Ok(true)
}
