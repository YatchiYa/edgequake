//! Migration 086 — EDGE BFS index reconcile (every bootstrap, SPEC-070).
//!
//! First principle: M086 sqlx file only covers graphs present at apply time.
//! New AGE graphs need `idx_edge_source_id` / `idx_edge_target_id` for
//! incident-edge batch + degrees. Re-run idempotent SSOT every boot.

use sqlx::PgPool;
use tracing::info;

use super::super::SQL_086_APPLY;
use super::execute_bootstrap_apply_sql;

/// Ensure BFS edge property indexes exist on every AGE graph with an EDGE table.
pub async fn reconcile_migration_086(pool: &PgPool) -> Result<bool, sqlx::Error> {
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
        step = "migration_086_apply_start",
        "Reconciling EDGE BFS indexes on all AGE graphs (M086 / SPEC-070)"
    );
    execute_bootstrap_apply_sql(pool, SQL_086_APPLY).await?;
    Ok(true)
}
