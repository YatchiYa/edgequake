//! Migration 092 — eq_* denorm reconcile (every bootstrap, SPEC-069).
//!
//! First principle: M092 sqlx file is a marker only. Runtime SSOT lives in
//! `migrations/support/092/apply.sql` so graphs created after migrate still get
//! columns/indexes/triggers at boot — never mid-delete under query timeout.

use sqlx::PgPool;
use tracing::info;

use super::super::SQL_092_APPLY;
use super::execute_bootstrap_apply_sql;

/// Ensure eq_* columns, indexes, and sync triggers exist on every AGE graph.
pub async fn reconcile_migration_092(pool: &PgPool) -> Result<bool, sqlx::Error> {
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
        step = "migration_092_apply_start",
        "Reconciling eq_* denorm schema on all AGE graphs (M092 / SPEC-069)"
    );
    execute_bootstrap_apply_sql(pool, SQL_092_APPLY).await?;
    Ok(true)
}
