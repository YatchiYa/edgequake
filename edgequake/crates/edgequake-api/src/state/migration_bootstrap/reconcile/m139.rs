//! SPEC-098 entity spine ensure (migration 139).

use sqlx::PgPool;
use tracing::{info, warn};

use super::super::SQL_139_APPLY;

/// Background task: ensure relational entity spine for typed fleet FK resolve.
pub async fn reconcile_migration_139_background(pool: &PgPool) {
    let progress: Option<String> = sqlx::query_scalar(
        "SELECT value::text FROM server_config WHERE key = 'spec098_spine_ensure_progress'",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if progress
        .as_deref()
        .is_some_and(|p| p.contains("completed_at"))
    {
        info!(
            target: "edgequake.migration",
            step = "migration_139_skip",
            "SPEC-098 spine ensure already completed"
        );
        return;
    }

    info!(
        target: "edgequake.migration",
        step = "migration_139_start",
        "Starting SPEC-098 entity spine ensure in background"
    );

    match sqlx::raw_sql(SQL_139_APPLY).execute(pool).await {
        Ok(_) => {
            info!(
                target: "edgequake.migration",
                step = "migration_139_complete",
                "SPEC-098 entity spine ensure complete"
            );
        }
        Err(e) => {
            warn!(
                target: "edgequake.migration",
                step = "migration_139_failed",
                error = %e,
                "SPEC-098 spine ensure failed — will retry on next restart"
            );
        }
    }
}
