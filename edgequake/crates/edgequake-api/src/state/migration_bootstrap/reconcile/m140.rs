//! SPEC-098 single EDGE arbiter + relationship spine (migration 140).

use sqlx::PgPool;
use tracing::{info, warn};

use super::super::SQL_140_APPLY;

/// Background task: enforce single EDGE arbiter and ensure relationship spine.
pub async fn reconcile_migration_140_background(pool: &PgPool) {
    let progress: Option<String> = sqlx::query_scalar(
        "SELECT value::text FROM server_config WHERE key = 'spec098_edge_arbiter_progress'",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if progress
        .as_deref()
        .is_some_and(|p| p.contains("completed_at") && !p.contains("failed_at"))
    {
        info!(
            target: "edgequake.migration",
            step = "migration_140_skip",
            "SPEC-098 edge arbiter reconcile already completed"
        );
        return;
    }

    info!(
        target: "edgequake.migration",
        step = "migration_140_start",
        "Starting SPEC-098 edge arbiter + relationship spine reconcile in background"
    );

    match sqlx::raw_sql(SQL_140_APPLY).execute(pool).await {
        Ok(_) => {
            info!(
                target: "edgequake.migration",
                step = "migration_140_complete",
                "SPEC-098 edge arbiter reconcile complete"
            );
        }
        Err(e) => {
            warn!(
                target: "edgequake.migration",
                step = "migration_140_failed",
                error = %e,
                "SPEC-098 edge arbiter reconcile failed — will retry on next restart"
            );
        }
    }
}
