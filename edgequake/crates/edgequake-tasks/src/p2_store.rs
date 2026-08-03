//! SPEC-120 P2 durable job, event, and attempt stores (WP3).
//!
//! These ports sit beside [`TaskStorage`](crate::storage::TaskStorage). Postgres
//! task storage dual-writes best-effort into the P2 tables during rollout.

use crate::{
    error::{TaskError, TaskResult},
    job::{Job, JobState, TaskAttempt, TaskEvent},
};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Create and read user-visible jobs.
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(&self, job: &Job) -> TaskResult<()>;
    async fn get_job(&self, job_id: Uuid) -> TaskResult<Option<Job>>;
    async fn update_job_state(&self, job_id: Uuid, state: JobState) -> TaskResult<()>;
}

/// Append-only per-task event stream.
#[async_trait]
pub trait EventWriter: Send + Sync {
    async fn append_event(
        &self,
        task_id: &str,
        kind: &str,
        payload: Option<Value>,
    ) -> TaskResult<i64>;
}

/// Lease-bearing execution attempts for audit and fencing diagnostics.
#[async_trait]
pub trait AttemptStore: Send + Sync {
    async fn start_attempt(
        &self,
        task_track_id: &str,
        attempt_no: i32,
        worker_id: &str,
        lease_token: Uuid,
        lease_expires_at: chrono::DateTime<Utc>,
    ) -> TaskResult<Uuid>;

    async fn finish_attempt(
        &self,
        attempt_id: Uuid,
        outcome: &str,
        fence_epoch: Option<i64>,
    ) -> TaskResult<()>;
}

/// In-memory P2 store for unit tests.
#[derive(Debug, Default)]
pub struct MemoryP2Store {
    jobs: Mutex<HashMap<Uuid, Job>>,
    events: Mutex<HashMap<String, Vec<TaskEvent>>>,
    attempts: Mutex<HashMap<Uuid, TaskAttempt>>,
    event_seq: Mutex<HashMap<String, i64>>,
}

impl MemoryP2Store {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl JobRepository for MemoryP2Store {
    async fn create_job(&self, job: &Job) -> TaskResult<()> {
        self.jobs
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?
            .insert(job.id, job.clone());
        Ok(())
    }

    async fn get_job(&self, job_id: Uuid) -> TaskResult<Option<Job>> {
        Ok(self
            .jobs
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?
            .get(&job_id)
            .cloned())
    }

    async fn update_job_state(&self, job_id: Uuid, state: JobState) -> TaskResult<()> {
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?;
        let job = jobs
            .get_mut(&job_id)
            .ok_or_else(|| TaskError::StorageError(format!("job not found: {job_id}")))?;
        job.state = state;
        Ok(())
    }
}

#[async_trait]
impl EventWriter for MemoryP2Store {
    async fn append_event(
        &self,
        task_id: &str,
        kind: &str,
        payload: Option<Value>,
    ) -> TaskResult<i64> {
        let mut seq_map = self
            .event_seq
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?;
        let seq = seq_map.entry(task_id.to_string()).or_insert(0);
        *seq += 1;
        let assigned = *seq;

        let event = TaskEvent {
            id: assigned,
            task_id: task_id.to_string(),
            job_id: None,
            seq: assigned,
            kind: kind.to_string(),
            payload,
            at: Utc::now(),
        };
        self.events
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?
            .entry(task_id.to_string())
            .or_default()
            .push(event);
        Ok(assigned)
    }
}

#[async_trait]
impl AttemptStore for MemoryP2Store {
    async fn start_attempt(
        &self,
        task_track_id: &str,
        attempt_no: i32,
        worker_id: &str,
        lease_token: Uuid,
        lease_expires_at: chrono::DateTime<Utc>,
    ) -> TaskResult<Uuid> {
        let id = Uuid::new_v4();
        let attempt = TaskAttempt {
            id,
            task_track_id: task_track_id.to_string(),
            attempt_no,
            worker_id: Some(worker_id.to_string()),
            lease_token: Some(lease_token),
            lease_expires_at: Some(lease_expires_at),
            started_at: Utc::now(),
            finished_at: None,
            outcome: None,
            fence_epoch: None,
        };
        self.attempts
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?
            .insert(id, attempt);
        Ok(id)
    }

    async fn finish_attempt(
        &self,
        attempt_id: Uuid,
        outcome: &str,
        fence_epoch: Option<i64>,
    ) -> TaskResult<()> {
        let mut attempts = self
            .attempts
            .lock()
            .map_err(|e| TaskError::StorageError(e.to_string()))?;
        let attempt = attempts
            .get_mut(&attempt_id)
            .ok_or_else(|| TaskError::StorageError(format!("attempt not found: {attempt_id}")))?;
        attempt.finished_at = Some(Utc::now());
        attempt.outcome = Some(outcome.to_string());
        attempt.fence_epoch = fence_epoch;
        Ok(())
    }
}

pub type SharedMemoryP2Store = Arc<MemoryP2Store>;

#[cfg(feature = "postgres")]
mod postgres_impl {
    use super::*;
    use crate::postgres::PostgresTaskStorage;
    use sqlx::Row;

    #[async_trait]
    impl JobRepository for PostgresTaskStorage {
        async fn create_job(&self, job: &Job) -> TaskResult<()> {
            sqlx::query(
                r#"
                INSERT INTO jobs (
                    id, tenant_id, workspace_id, operation, subject_kind, subject_id,
                    idempotency_key, state, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
                "#,
            )
            .bind(job.id)
            .bind(job.tenant_id)
            .bind(job.workspace_id)
            .bind(&job.operation)
            .bind(&job.subject_kind)
            .bind(&job.subject_id)
            .bind(&job.idempotency_key)
            .bind(match job.state {
                JobState::Requested => "requested",
                JobState::Running => "running",
                JobState::Cancelling => "cancelling",
                JobState::Succeeded => "succeeded",
                JobState::Failed => "failed",
                JobState::Cancelled => "cancelled",
            })
            .bind(job.created_at)
            .execute(self.pool())
            .await
            .map_err(|e| TaskError::StorageError(format!("create_job: {e}")))?;
            Ok(())
        }

        async fn get_job(&self, job_id: Uuid) -> TaskResult<Option<Job>> {
            let row = sqlx::query(
                r#"
                SELECT id, tenant_id, workspace_id, operation, subject_kind, subject_id,
                       idempotency_key, state, created_at
                FROM jobs WHERE id = $1
                "#,
            )
            .bind(job_id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| TaskError::StorageError(format!("get_job: {e}")))?;

            Ok(row.map(|row| Job {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                workspace_id: row.get("workspace_id"),
                operation: row.get("operation"),
                subject_kind: row.get("subject_kind"),
                subject_id: row.get("subject_id"),
                idempotency_key: row.get("idempotency_key"),
                state: row
                    .get::<String, _>("state")
                    .parse()
                    .unwrap_or(JobState::Requested),
                created_at: row.get("created_at"),
            }))
        }

        async fn update_job_state(&self, job_id: Uuid, state: JobState) -> TaskResult<()> {
            sqlx::query("UPDATE jobs SET state = $2 WHERE id = $1")
                .bind(job_id)
                .bind(match state {
                    JobState::Requested => "requested",
                    JobState::Running => "running",
                    JobState::Cancelling => "cancelling",
                    JobState::Succeeded => "succeeded",
                    JobState::Failed => "failed",
                    JobState::Cancelled => "cancelled",
                })
                .execute(self.pool())
                .await
                .map_err(|e| TaskError::StorageError(format!("update_job_state: {e}")))?;
            Ok(())
        }
    }

    #[async_trait]
    impl EventWriter for PostgresTaskStorage {
        async fn append_event(
            &self,
            task_id: &str,
            kind: &str,
            payload: Option<Value>,
        ) -> TaskResult<i64> {
            let row = sqlx::query(
                r#"
                INSERT INTO task_events (task_id, job_id, seq, kind, payload)
                SELECT $1, t.job_id,
                       COALESCE((SELECT MAX(seq) FROM task_events WHERE task_id = $1), 0) + 1,
                       $2, $3
                FROM tasks t
                WHERE t.track_id = $1
                RETURNING seq
                "#,
            )
            .bind(task_id)
            .bind(kind)
            .bind(payload)
            .fetch_one(self.pool())
            .await
            .map_err(|e| TaskError::StorageError(format!("append_event: {e}")))?;
            Ok(row.get("seq"))
        }
    }

    #[async_trait]
    impl AttemptStore for PostgresTaskStorage {
        async fn start_attempt(
            &self,
            task_track_id: &str,
            attempt_no: i32,
            worker_id: &str,
            lease_token: Uuid,
            lease_expires_at: chrono::DateTime<Utc>,
        ) -> TaskResult<Uuid> {
            let id = Uuid::new_v4();
            // Idempotent on reclaim: same (track_id, attempt_no) must not WARN-spam.
            let row = sqlx::query(
                r#"
                INSERT INTO attempts (
                    id, task_track_id, attempt_no, worker_id, lease_token, lease_expires_at
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (task_track_id, attempt_no) DO UPDATE SET
                    worker_id = EXCLUDED.worker_id,
                    lease_token = EXCLUDED.lease_token,
                    lease_expires_at = EXCLUDED.lease_expires_at
                RETURNING id
                "#,
            )
            .bind(id)
            .bind(task_track_id)
            .bind(attempt_no)
            .bind(worker_id)
            .bind(lease_token)
            .bind(lease_expires_at)
            .fetch_one(self.pool())
            .await
            .map_err(|e| TaskError::StorageError(format!("start_attempt: {e}")))?;
            Ok(row.get("id"))
        }

        async fn finish_attempt(
            &self,
            attempt_id: Uuid,
            outcome: &str,
            fence_epoch: Option<i64>,
        ) -> TaskResult<()> {
            sqlx::query(
                r#"
                UPDATE attempts
                SET finished_at = NOW(), outcome = $2, fence_epoch = $3
                WHERE id = $1
                "#,
            )
            .bind(attempt_id)
            .bind(outcome)
            .bind(fence_epoch)
            .execute(self.pool())
            .await
            .map_err(|e| TaskError::StorageError(format!("finish_attempt: {e}")))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_p2_store_round_trip() {
        let store = MemoryP2Store::new();
        let job_id = Uuid::new_v4();
        let job = Job {
            id: job_id,
            tenant_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            operation: "pdf.ingest".to_string(),
            subject_kind: Some("pdf".to_string()),
            subject_id: Some("abc".to_string()),
            idempotency_key: Some("key-1".to_string()),
            state: JobState::Requested,
            created_at: Utc::now(),
        };
        store.create_job(&job).await.unwrap();
        store
            .update_job_state(job_id, JobState::Running)
            .await
            .unwrap();
        let loaded = store.get_job(job_id).await.unwrap().unwrap();
        assert_eq!(loaded.state, JobState::Running);

        let seq = store
            .append_event("task-1", "status_changed", None)
            .await
            .unwrap();
        assert_eq!(seq, 1);

        let attempt_id = store
            .start_attempt(
                "task-1",
                1,
                "worker-a",
                Uuid::new_v4(),
                Utc::now() + chrono::Duration::minutes(5),
            )
            .await
            .unwrap();
        store
            .finish_attempt(attempt_id, "succeeded", None)
            .await
            .unwrap();
    }
}
