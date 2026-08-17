//! SPEC-120 P2 job, event, attempt, and fairness-ledger value types.
//!
//! These types are intentionally storage-neutral. The P2 migration introduces
//! their durable tables while existing task APIs remain the operational path.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Coarse lifecycle of a user-visible operation spanning one or more tasks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    #[default]
    Requested,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}

impl std::str::FromStr for JobState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "requested" => Self::Requested,
            "running" => Self::Running,
            "cancelling" => Self::Cancelling,
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => return Err(()),
        })
    }
}

/// User-visible operation that may own a task graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub workspace_id: Uuid,
    pub operation: String,
    pub subject_kind: Option<String>,
    pub subject_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub state: JobState,
    pub created_at: DateTime<Utc>,
}

/// Append-only event emitted by a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskEvent {
    pub id: i64,
    pub task_id: String,
    pub job_id: Option<Uuid>,
    pub seq: i64,
    pub kind: String,
    pub payload: Option<serde_json::Value>,
    pub at: DateTime<Utc>,
}

/// One lease-bearing execution attempt for audit and fencing diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttempt {
    pub id: Uuid,
    pub task_track_id: String,
    pub attempt_no: i32,
    pub worker_id: Option<String>,
    pub lease_token: Option<Uuid>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub fence_epoch: Option<i64>,
}

/// Configured share and concurrency cap for a tenant lane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TenantLaneQuota {
    pub tenant_id: Uuid,
    pub fairness_class: String,
    pub weight: f64,
    pub max_concurrent: i32,
}

/// Persisted weighted virtual runtime for a tenant lane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TenantVruntime {
    pub tenant_id: Uuid,
    pub fairness_class: String,
    pub vruntime: f64,
}
