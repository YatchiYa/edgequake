//! SPEC-098 document lifecycle status CHECK (migration 141).

use sqlx::PgPool;
use tracing::{info, warn};

use super::super::SQL_141_APPLY;

/// Background task: ensure `documents_valid_status` includes deleting/delete_failed.
pub async fn reconcile_migration_141_background(pool: &PgPool) {
    let progress: Option<String> = sqlx::query_scalar(
        "SELECT value::text FROM server_config WHERE key = 'spec098_document_lifecycle_status'",
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
            step = "migration_141_skip",
            "SPEC-098 document lifecycle status CHECK already completed"
        );
        return;
    }

    info!(
        target: "edgequake.migration",
        step = "migration_141_start",
        "Starting SPEC-098 document lifecycle status CHECK reconcile"
    );

    match sqlx::raw_sql(SQL_141_APPLY).execute(pool).await {
        Ok(_) => {
            info!(
                target: "edgequake.migration",
                step = "migration_141_complete",
                "SPEC-098 document lifecycle status CHECK complete"
            );
        }
        Err(e) => {
            warn!(
                target: "edgequake.migration",
                step = "migration_141_failed",
                error = %e,
                "SPEC-098 document lifecycle status CHECK failed — will retry on next restart"
            );
        }
    }
}
