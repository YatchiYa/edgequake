//! SPEC-091 migration job ledger SQL (migration 106). Single-writer lease +
//! same-transaction batch ledger (LAW-D6: no data movement without ledger row).

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageError;

/// Row identity + lease for one claimed job.
#[derive(Debug, Clone)]
pub struct JobLease {
    pub job_id: Uuid,
    pub step_id: String,
    pub schema_generation: i32,
    pub cursor_position: Value,
    pub processed_count: i64,
    pub batch_size: i32,
}

/// Insert the job row if absent (registration happens in every mode ≠ off so
/// `/admin/migration-jobs` and the SQL view show pending work immediately).
pub async fn ensure_job_row(
    pool: &PgPool,
    step_id: &str,
    step_sha384: &str,
    schema_generation: i32,
    reversibility: &str,
    batch_size: i32,
    estimated_total: Option<i64>,
) -> Result<(), StorageError> {
    sqlx::query(
        r#"
        INSERT INTO edgequake.edgequake_migration_job
            (step_id, step_sha384, schema_generation, state, reversibility,
             batch_size, estimated_total)
        VALUES ($1, $2, $3, 'pending', $4, $5, $6)
        ON CONFLICT (step_id, schema_generation) DO NOTHING
        "#,
    )
    .bind(step_id)
    .bind(step_sha384)
    .bind(schema_generation)
    .bind(reversibility)
    .bind(batch_size)
    .bind(estimated_total)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration ensure_job_row failed: {e}")))?;
    Ok(())
}

/// Single-writer lease claim (FOR UPDATE-free: the UPDATE predicate is atomic).
///
/// WHY `step_sha384` re-check: a changed descriptor must not silently resume a
/// job created from different bytes — the UNIQUE key includes generation, and
/// the claim refuses mismatched hashes by returning `Ok(None)`.
pub async fn claim_lease(
    pool: &PgPool,
    step_id: &str,
    step_sha384: &str,
    schema_generation: i32,
    owner: &str,
    ttl_secs: i64,
) -> Result<Option<JobLease>, StorageError> {
    let row = sqlx::query_as::<_, (Uuid, Value, i64, i32)>(
        r#"
        UPDATE edgequake.edgequake_migration_job
        SET lease_owner = $4,
            lease_expires_at = now() + make_interval(secs => $5),
            heartbeat_at = now(),
            state = CASE WHEN state = 'pending' THEN 'preflight' ELSE state END,
            started_at = COALESCE(started_at, now())
        WHERE step_id = $1
          AND schema_generation = $3
          AND step_sha384 = $2
          AND state IN ('pending', 'preflight', 'running', 'paused')
          AND (lease_expires_at IS NULL OR lease_expires_at < now() OR lease_owner = $4)
        RETURNING job_id, COALESCE(cursor_position, '{}'), processed_count, batch_size
        "#,
    )
    .bind(step_id)
    .bind(step_sha384)
    .bind(schema_generation)
    .bind(owner)
    .bind(ttl_secs)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration claim_lease failed: {e}")))?;

    Ok(row.map(
        |(job_id, cursor_position, processed_count, batch_size)| JobLease {
            job_id,
            step_id: step_id.to_string(),
            schema_generation,
            cursor_position,
            processed_count,
            batch_size,
        },
    ))
}

/// Progress + heartbeat in one statement (called once per committed batch).
#[allow(clippy::too_many_arguments)]
pub async fn record_batch_progress<'e, E>(
    executor: E,
    job_id: Uuid,
    owner: &str,
    ttl_secs: i64,
    scanned: i64,
    failed: i64,
    cursor: &Value,
    batch_size: i32,
    throttle_reason: Option<&str>,
) -> Result<(), StorageError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        UPDATE edgequake.edgequake_migration_job
        SET state = CASE WHEN state IN ('paused', 'cancelled') THEN state ELSE 'running' END,
            processed_count = processed_count + $2,
            failed_count = failed_count + $8,
            cursor_position = $3,
            batch_size = $4,
            throttle_reason = $5,
            heartbeat_at = now(),
            lease_expires_at = now() + make_interval(secs => $6)
        WHERE job_id = $1 AND lease_owner = $7
        "#,
    )
    .bind(job_id)
    .bind(scanned)
    .bind(cursor)
    .bind(batch_size)
    .bind(throttle_reason)
    .bind(ttl_secs)
    .bind(owner)
    .bind(failed)
    .execute(executor)
    .await
    .map_err(|e| StorageError::Database(format!("migration record_batch_progress failed: {e}")))?;
    Ok(())
}

/// Batch ledger row — MUST run in the same transaction as the data movement.
pub async fn insert_batch_ledger<'e, E>(
    executor: E,
    job_id: Uuid,
    batch_seq: i64,
    cursor_from: &Value,
    cursor_to: &Value,
    row_count: i32,
    duration_ms: i32,
) -> Result<(), StorageError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        INSERT INTO edgequake.edgequake_migration_batch
            (job_id, batch_seq, cursor_from, cursor_to, row_count, duration_ms)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (job_id, batch_seq) DO NOTHING
        "#,
    )
    .bind(job_id)
    .bind(batch_seq)
    .bind(cursor_from)
    .bind(cursor_to)
    .bind(row_count)
    .bind(duration_ms)
    .execute(executor)
    .await
    .map_err(|e| StorageError::Database(format!("migration insert_batch_ledger failed: {e}")))?;
    Ok(())
}

/// Operator control action on a migration job (SPEC-091 P1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobControl {
    Pause,
    Resume,
    Cancel,
}

impl JobControl {
    /// Pure transition table (SSOT — unit-tested without a DB).
    /// Returns the new state when the action is legal from `current`.
    pub fn transition(self, current: &str) -> Option<&'static str> {
        match (self, current) {
            (Self::Pause, "pending" | "preflight" | "running" | "verifying") => Some("paused"),
            (Self::Resume, "paused") => Some("running"),
            (Self::Cancel, "pending" | "preflight" | "running" | "paused") => Some("cancelled"),
            _ => None,
        }
    }
}

/// Apply a control action with a guarded UPDATE (atomic transition check —
/// a racing runner cannot force an illegal transition).
/// Returns the new state; `Err` carries the current state for 409 mapping.
pub async fn control_job(
    pool: &PgPool,
    job_id: Uuid,
    action: JobControl,
) -> Result<String, StorageError> {
    let current: Option<String> =
        sqlx::query_scalar("SELECT state FROM edgequake.edgequake_migration_job WHERE job_id = $1")
            .bind(job_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Database(format!("migration control read failed: {e}")))?;
    let Some(current) = current else {
        return Err(StorageError::InvalidQuery(format!(
            "migration job {job_id} not found"
        )));
    };
    let Some(next) = action.transition(&current) else {
        return Err(StorageError::InvalidQuery(format!(
            "cannot {action:?} migration job in state '{current}'"
        )));
    };
    sqlx::query(
        r#"
        UPDATE edgequake.edgequake_migration_job
        SET state = $2,
            completed_at = CASE WHEN $2 = 'cancelled' THEN now() ELSE completed_at END,
            lease_owner = CASE WHEN $2 = 'cancelled' THEN NULL ELSE lease_owner END,
            lease_expires_at = CASE WHEN $2 = 'cancelled' THEN NULL ELSE lease_expires_at END
        WHERE job_id = $1 AND state = $3
        "#,
    )
    .bind(job_id)
    .bind(next)
    .bind(&current)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration control update failed: {e}")))?;
    Ok(next.to_string())
}

/// Current state of a job (runner poll; `None` when the job vanished).
pub async fn current_state(pool: &PgPool, job_id: Uuid) -> Result<Option<String>, StorageError> {
    sqlx::query_scalar("SELECT state FROM edgequake.edgequake_migration_job WHERE job_id = $1")
        .bind(job_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Database(format!("migration state poll failed: {e}")))
}

/// Standalone lease heartbeat (used by the runner while paused so a long
/// pause does not look like a dead lease owner).
pub async fn heartbeat(
    pool: &PgPool,
    job_id: Uuid,
    owner: &str,
    ttl_secs: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        r#"
        UPDATE edgequake.edgequake_migration_job
        SET heartbeat_at = now(),
            lease_expires_at = now() + make_interval(secs => $3)
        WHERE job_id = $1 AND lease_owner = $2
        "#,
    )
    .bind(job_id)
    .bind(owner)
    .bind(ttl_secs)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration heartbeat failed: {e}")))?;
    Ok(())
}

/// One recent batch ledger row (detail surface + rate/ETA math).
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchSample {
    pub batch_seq: i64,
    pub row_count: i32,
    pub duration_ms: i32,
    pub committed_at: String,
}

/// Job detail: job row + recent batches + derived rate and ETA (P1/P3).
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationJobDetail {
    pub job_id: String,
    pub step_id: String,
    pub state: String,
    pub reversibility: String,
    pub processed_count: i64,
    pub failed_count: i64,
    pub estimated_total: Option<i64>,
    pub completion_pct: Option<f64>,
    pub lease_owner: Option<String>,
    pub throttle_reason: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub last_error: Option<Value>,
    /// Rows/sec derived from the last ≤20 committed batches.
    pub rows_per_sec: Option<f64>,
    /// ETA in seconds = remaining / rate (None when unknown/complete).
    pub eta_seconds: Option<f64>,
    pub recent_batches: Vec<BatchSample>,
}

/// Rate + ETA from the batch ledger (single query — shared by detail
/// endpoint, runner ETA logs, and the `migrate status` CLI; DRY).
pub async fn job_detail(
    pool: &PgPool,
    job_id: Uuid,
) -> Result<Option<MigrationJobDetail>, StorageError> {
    #[derive(sqlx::FromRow)]
    struct JobRow {
        step_id: String,
        state: String,
        reversibility: String,
        processed_count: i64,
        failed_count: i64,
        estimated_total: Option<i64>,
        lease_owner: Option<String>,
        throttle_reason: Option<String>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        last_error: Option<Value>,
    }
    let Some(job) = sqlx::query_as::<_, JobRow>(
        r#"
        SELECT step_id, state, reversibility, processed_count, failed_count,
               estimated_total, lease_owner, throttle_reason,
               started_at, completed_at, last_error
        FROM edgequake.edgequake_migration_job WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration detail failed: {e}")))?
    else {
        return Ok(None);
    };

    #[derive(sqlx::FromRow)]
    struct BatchRow {
        batch_seq: i64,
        row_count: i32,
        duration_ms: i32,
        committed_at: chrono::DateTime<chrono::Utc>,
    }
    let batches = sqlx::query_as::<_, BatchRow>(
        r#"
        SELECT batch_seq, row_count, duration_ms, committed_at
        FROM edgequake.edgequake_migration_batch
        WHERE job_id = $1
        ORDER BY batch_seq DESC
        LIMIT 20
        "#,
    )
    .bind(job_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration batch samples failed: {e}")))?;

    let total_rows: i64 = batches.iter().map(|b| i64::from(b.row_count)).sum();
    let total_ms: i64 = batches.iter().map(|b| i64::from(b.duration_ms)).sum();
    let rows_per_sec =
        (total_ms > 0 && total_rows > 0).then(|| total_rows as f64 * 1000.0 / total_ms as f64);
    let remaining = job
        .estimated_total
        .map(|t| (t - job.processed_count).max(0));
    let eta_seconds = match (remaining, rows_per_sec) {
        (Some(rem), Some(rate)) if rate > 0.0 && rem > 0 => Some(rem as f64 / rate),
        _ => None,
    };
    let completion_pct = match job.estimated_total {
        Some(t) if t > 0 => Some(100.0 * job.processed_count as f64 / t as f64),
        _ => None,
    };

    Ok(Some(MigrationJobDetail {
        job_id: job_id.to_string(),
        step_id: job.step_id,
        state: job.state,
        reversibility: job.reversibility,
        processed_count: job.processed_count,
        failed_count: job.failed_count,
        estimated_total: job.estimated_total,
        completion_pct,
        lease_owner: job.lease_owner,
        throttle_reason: job.throttle_reason,
        started_at: job.started_at.map(|t| t.to_rfc3339()),
        completed_at: job.completed_at.map(|t| t.to_rfc3339()),
        last_error: job.last_error,
        rows_per_sec,
        eta_seconds,
        recent_batches: batches
            .into_iter()
            .map(|b| BatchSample {
                batch_seq: b.batch_seq,
                row_count: b.row_count,
                duration_ms: b.duration_ms,
                committed_at: b.committed_at.to_rfc3339(),
            })
            .collect(),
    }))
}

/// Terminal state transition (completed/failed), releasing the lease.
pub async fn finish_job(
    pool: &PgPool,
    job_id: Uuid,
    owner: &str,
    state: &str,
    last_error: Option<Value>,
) -> Result<(), StorageError> {
    debug_assert!(matches!(state, "completed" | "failed" | "rolled_back"));
    sqlx::query(
        r#"
        UPDATE edgequake.edgequake_migration_job
        SET state = $2,
            completed_at = CASE WHEN $2 = 'completed' THEN now() ELSE completed_at END,
            last_error = $3,
            lease_owner = NULL,
            lease_expires_at = NULL
        WHERE job_id = $1 AND lease_owner = $4
        "#,
    )
    .bind(job_id)
    .bind(state)
    .bind(last_error)
    .bind(owner)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("migration finish_job failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn contract_spec091_lease_ttl_positive() {
        // Regression guard: lease SQL uses make_interval with the configured TTL;
        // a zero/negative TTL would livelock claims. SQL text check.
        let src = include_str!("lease.rs");
        assert!(src.contains("make_interval(secs => $5)"));
        assert!(src.contains("lease_expires_at < now()"));
    }

    #[test]
    fn contract_spec091_ledger_same_tx_documented() {
        let src = include_str!("lease.rs");
        assert!(src.contains("MUST run in the same transaction"));
    }

    #[test]
    fn contract_spec091_control_transition_table() {
        use super::JobControl::*;
        // Pause legal from any active state, not from terminal/paused.
        for s in ["pending", "preflight", "running", "verifying"] {
            assert_eq!(Pause.transition(s), Some("paused"), "pause from {s}");
        }
        for s in ["paused", "completed", "failed", "cancelled", "rolled_back"] {
            assert_eq!(Pause.transition(s), None, "no pause from {s}");
        }
        // Resume only from paused.
        assert_eq!(Resume.transition("paused"), Some("running"));
        assert_eq!(Resume.transition("running"), None);
        assert_eq!(Resume.transition("completed"), None);
        // Cancel from non-terminal states incl. paused; never from terminal.
        for s in ["pending", "preflight", "running", "paused"] {
            assert_eq!(Cancel.transition(s), Some("cancelled"), "cancel from {s}");
        }
        for s in [
            "verifying",
            "completed",
            "failed",
            "cancelled",
            "rolled_back",
        ] {
            assert_eq!(Cancel.transition(s), None, "no cancel from {s}");
        }
    }
}
