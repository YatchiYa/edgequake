//! SPEC-091 QW2 — Queue position + ETA projection (LAW-Q4: explicit queued state).
//!
//! A `pending` task's position is the count of pending tasks enqueued before
//! it (FCFS projection); its ETA derives from the **measured** drain rate over
//! a recent window — never a guessed constant (LAW-Q1). When no completions
//! exist in the window the ETA is honestly reported as clamped-unknown
//! (`basis: no_history`) instead of a fabricated number.

use crate::{error::TaskResult, storage::TaskStorage};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Env key: ETA measurement window (seconds). Default 600s.
pub const ETA_WINDOW_SECS_ENV: &str = "EDGEQUAKE_QUEUE_ETA_WINDOW_SECS";
/// Env key: ETA clamp maximum (seconds). Default 14400s (4h).
pub const ETA_CLAMP_MAX_SECS_ENV: &str = "EDGEQUAKE_ADMISSION_ETA_CLAMP_MAX_SECS";
/// Default ETA measurement window.
pub const DEFAULT_ETA_WINDOW_SECS: u64 = 600;
/// Default ETA clamp maximum (4h — risk R-15: ETAs must be marked uncertain).
pub const DEFAULT_ETA_CLAMP_MAX_SECS: u64 = 14_400;

/// Why an ETA is what it is — clients can show honest uncertainty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueEtaBasis {
    /// Derived from measured completions in the recent window.
    Measured,
    /// No completions observed: ETA clamped at max, treat as "unknown".
    NoHistory,
}

impl QueueEtaBasis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Measured => "measured",
            Self::NoHistory => "no_history",
        }
    }
}

/// Queue admission projection for one pending task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueueEstimate {
    /// 1-based position in the FCFS pending queue.
    pub position: u64,
    /// Estimated seconds until claim (clamped to `[0, clamp_max]`).
    pub eta_seconds: u64,
    /// Whether the ETA comes from measured drain or is clamped-unknown.
    pub basis: QueueEtaBasis,
}

/// Pure ETA math (LAW-Q1: capacity derived from measured throughput).
///
/// `position` — pending tasks ahead (0 means next in line).
/// `completed_in_window` / `window_secs` — measured drain over the window.
/// `clamp_max_secs` — honesty ceiling.
pub fn estimate_eta(
    position: u64,
    completed_in_window: u64,
    window_secs: u64,
    clamp_max_secs: u64,
) -> QueueEstimate {
    if completed_in_window == 0 || window_secs == 0 {
        return QueueEstimate {
            position,
            eta_seconds: clamp_max_secs,
            basis: QueueEtaBasis::NoHistory,
        };
    }
    let secs_per_task = window_secs as f64 / completed_in_window as f64;
    let eta = (position as f64 * secs_per_task).round();
    QueueEstimate {
        position,
        eta_seconds: (eta as u64).min(clamp_max_secs),
        basis: QueueEtaBasis::Measured,
    }
}

/// Project queue position + ETA for a pending task created at `created_at`.
pub async fn estimate_queue<S: TaskStorage + ?Sized>(
    storage: &S,
    created_at: DateTime<Utc>,
) -> TaskResult<QueueEstimate> {
    let window_secs = env_u64(ETA_WINDOW_SECS_ENV, DEFAULT_ETA_WINDOW_SECS);
    let clamp_max = env_u64(ETA_CLAMP_MAX_SECS_ENV, DEFAULT_ETA_CLAMP_MAX_SECS);
    let ahead = storage.count_pending_older_than(created_at).await?;
    let completed = storage
        .count_completed_within(Duration::from_secs(window_secs))
        .await?;
    Ok(estimate_eta(ahead, completed, window_secs, clamp_max))
}

/// SPEC-091 IP0: batch queue estimates for many track ids (≤2 storage RTs).
///
/// 1. `pending_queue_ahead_batch` — ranks for pending members of `track_ids`
/// 2. `count_completed_within` — one drain-rate sample for the whole page
///
/// Returns map `track_id → QueueEstimate`. Missing / non-pending ids omitted.
pub async fn estimate_queues_batch<S: TaskStorage + ?Sized>(
    storage: &S,
    track_ids: &[String],
) -> TaskResult<std::collections::HashMap<String, QueueEstimate>> {
    let mut out = std::collections::HashMap::new();
    if track_ids.is_empty() {
        return Ok(out);
    }
    let window_secs = env_u64(ETA_WINDOW_SECS_ENV, DEFAULT_ETA_WINDOW_SECS);
    let clamp_max = env_u64(ETA_CLAMP_MAX_SECS_ENV, DEFAULT_ETA_CLAMP_MAX_SECS);
    let ranks = storage.pending_queue_ahead_batch(track_ids).await?;
    let completed = storage
        .count_completed_within(Duration::from_secs(window_secs))
        .await?;
    for (track_id, ahead) in ranks {
        out.insert(
            track_id,
            estimate_eta(ahead, completed, window_secs, clamp_max),
        );
    }
    Ok(out)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// LAW-Q4: ETA is Little's Law over measured drain — never a constant.
    #[test]
    fn contract_spec091_eta_monotone_in_queue_position() {
        // Drain: 6 completions in 600s → 100s/task.
        let first = estimate_eta(0, 6, 600, 14_400);
        let tenth = estimate_eta(9, 6, 600, 14_400);
        assert_eq!(first.eta_seconds, 0);
        assert_eq!(tenth.eta_seconds, 900);
        assert!(tenth.eta_seconds > first.eta_seconds);
        assert_eq!(tenth.basis, QueueEtaBasis::Measured);
    }

    /// No drain history → honest clamped unknown (R-15), not a fabrication.
    #[test]
    fn no_history_is_clamped_and_marked() {
        let e = estimate_eta(3, 0, 600, 14_400);
        assert_eq!(e.eta_seconds, 14_400);
        assert_eq!(e.basis, QueueEtaBasis::NoHistory);
    }

    /// ETA never exceeds the clamp, however deep the queue.
    #[test]
    fn eta_is_clamped() {
        let e = estimate_eta(10_000, 1, 60, 14_400);
        assert_eq!(e.eta_seconds, 14_400);
        assert_eq!(e.basis, QueueEtaBasis::Measured);
    }

    /// SPEC-091 IP0 / IP-AC-02: batch ranks 50 pending ids with unique FCFS positions.
    #[tokio::test]
    async fn batch_estimate_ranks_pending_page() {
        use crate::memory::MemoryTaskStorage;
        use crate::types::{Task, TaskStatus, TaskType};
        use uuid::Uuid;

        let tenant = Uuid::new_v4();
        let workspace = Uuid::new_v4();
        let storage = MemoryTaskStorage::new();
        let mut ids = Vec::new();
        let base = Utc::now() - chrono::Duration::seconds(100);
        for i in 0..50i64 {
            let mut t = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));
            t.created_at = base + chrono::Duration::seconds(i);
            t.updated_at = t.created_at;
            assert_eq!(t.status, TaskStatus::Pending);
            storage.create_task(&t).await.unwrap();
            ids.push(t.track_id.clone());
        }
        // Non-pending should be omitted.
        let mut done = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));
        done.status = TaskStatus::Indexed;
        done.created_at = base;
        storage.create_task(&done).await.unwrap();
        ids.push(done.track_id.clone());

        let map = estimate_queues_batch(&storage, &ids).await.unwrap();
        assert_eq!(map.len(), 50, "indexed task must be omitted");
        let mut positions: Vec<u64> = map.values().map(|e| e.position).collect();
        positions.sort_unstable();
        assert_eq!(positions, (0..50).collect::<Vec<_>>());
        assert_eq!(map.get(&ids[0]).unwrap().position, 0);
        assert_eq!(map.get(&ids[49]).unwrap().position, 49);
        assert!(!map.contains_key(&done.track_id));
    }

    #[tokio::test]
    async fn batch_estimate_empty_ids() {
        use crate::memory::MemoryTaskStorage;
        let storage = MemoryTaskStorage::new();
        let map = estimate_queues_batch(&storage, &[]).await.unwrap();
        assert!(map.is_empty());
    }
}
