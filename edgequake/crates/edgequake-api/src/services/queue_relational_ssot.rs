//! Relational queue SSOT janitor + park document mirror (INV-Q10 / A+ control plane).
//!
//! ## SOLID
//! - **S**: Only heals `pending`+active-hold → `held` and mirrors capacity-wait on park.
//! - **O**: Storage backends own the SQL/memory repair; this module orchestrates.
//! - **D**: Depends on [`TaskStorage`] / [`FairnessParkHook`], not concrete Postgres.

use edgequake_observability::ErrorEvent;
use edgequake_tasks::{FairnessParkHook, SharedTaskStorage, Task, TaskResult, TaskStorage};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Result of one held-status reconcile pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeldStatusReconcileReport {
    pub repaired: u64,
    pub remaining_pending_with_hold: u64,
}

/// Self-heal INV-Q10 drift and publish observability.
pub async fn reconcile_held_status_ssot(
    storage: &dyn TaskStorage,
) -> TaskResult<HeldStatusReconcileReport> {
    let repaired = storage.reconcile_held_status_drift().await?;
    let remaining_pending_with_hold = storage.count_pending_with_active_hold().await?;
    edgequake_observability::record_held_status_drift(repaired, remaining_pending_with_hold);
    if repaired > 0 {
        warn!(
            repaired,
            remaining_pending_with_hold,
            "INV-Q10: repaired pending+active-hold → held (relational SSOT janitor)"
        );
    } else if remaining_pending_with_hold > 0 {
        warn!(
            remaining_pending_with_hold,
            "INV-Q10: pending+active-hold drift remains after reconcile"
        );
    } else {
        tracing::debug!("INV-Q10: held-status drift reconcile clean");
    }
    Ok(HeldStatusReconcileReport {
        repaired,
        remaining_pending_with_hold,
    })
}

/// Boot + periodic held-status janitor (non-fatal on error).
pub fn spawn_held_status_janitor(storage: SharedTaskStorage, interval: Duration) {
    tokio::spawn(async move {
        match reconcile_held_status_ssot(storage.as_ref()).await {
            Ok(report) if report.repaired > 0 || report.remaining_pending_with_hold > 0 => {
                info!(
                    repaired = report.repaired,
                    remaining = report.remaining_pending_with_hold,
                    "INV-Q10: boot held-status janitor complete"
                );
            }
            Ok(_) => {}
            Err(e) => {
                ErrorEvent::log_domain_warn(
                    "startup",
                    "reconcile_held_status_ssot",
                    &e.to_string(),
                    json!({ "non_fatal": true }),
                );
            }
        }

        if interval.is_zero() {
            return;
        }
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        ticker.tick().await; // boot already ran
        loop {
            ticker.tick().await;
            if let Err(e) = reconcile_held_status_ssot(storage.as_ref()).await {
                ErrorEvent::log_domain_warn(
                    "startup",
                    "periodic_reconcile_held_status_ssot",
                    &e.to_string(),
                    json!({ "non_fatal": true }),
                );
            }
        }
    });
}

/// Default periodic interval (5 minutes) — override with `EDGEQUAKE_HELD_STATUS_JANITOR_SECS`.
pub fn held_status_janitor_interval_from_env() -> Duration {
    let secs = std::env::var("EDGEQUAKE_HELD_STATUS_JANITOR_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);
    Duration::from_secs(secs)
}

/// Park hook: mirror capacity-wait onto relational document metadata (admission-idle only).
#[cfg(feature = "postgres")]
pub struct CapacityWaitParkHook {
    pool: sqlx::PgPool,
}

#[cfg(feature = "postgres")]
impl CapacityWaitParkHook {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub fn shared(pool: sqlx::PgPool) -> Arc<Self> {
        Arc::new(Self::new(pool))
    }
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl FairnessParkHook for CapacityWaitParkHook {
    async fn on_parked(&self, task: &Task) {
        let Some(document_id) = task.resolved_document_id() else {
            return;
        };
        let wait_message = edgequake_tasks::capacity_wait_reason_from_progress(&task.progress);
        let mirrored =
            crate::services::document_stage_mirror::mirror_capacity_wait_to_relational_with_message(
                &self.pool,
                document_id,
                &task.track_id,
                wait_message.as_deref(),
            )
            .await;
        if mirrored {
            tracing::debug!(
                track_id = %task.track_id,
                document_id,
                "Mirrored capacity-wait to relational document after fairness park"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_tasks::memory::MemoryTaskStorage;
    use edgequake_tasks::{TaskStatus, TaskType};
    use std::time::Duration as StdDuration;
    use uuid::Uuid;

    #[tokio::test]
    async fn reconcile_repairs_pending_with_active_hold() {
        let storage = MemoryTaskStorage::new();
        let tenant = Uuid::new_v4();
        let workspace = Uuid::new_v4();
        let task = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();

        // Simulate legacy drift: pending + active hold TTL (pre-mark_fairness_hold fix).
        {
            let mut drifted = storage.get_task(&track_id).await.unwrap().unwrap();
            drifted.fairness_hold_until =
                Some(chrono::Utc::now() + chrono::Duration::seconds(60));
            drifted.status = TaskStatus::Pending;
            storage.update_task(&drifted).await.unwrap();
        }

        assert_eq!(storage.count_pending_with_active_hold().await.unwrap(), 1);
        let report = reconcile_held_status_ssot(&storage).await.unwrap();
        assert_eq!(report.repaired, 1);
        assert_eq!(report.remaining_pending_with_hold, 0);
        let fixed = storage.get_task(&track_id).await.unwrap().unwrap();
        assert_eq!(fixed.status, TaskStatus::Held);

        // Idempotent second pass.
        let again = reconcile_held_status_ssot(&storage).await.unwrap();
        assert_eq!(again.repaired, 0);
        assert_eq!(again.remaining_pending_with_hold, 0);
        let _ = StdDuration::from_secs(1);
    }
}
