//! Cancel-wins decision SSOT (SPEC-120 / INV-C1).
//!
//! Consolidates cancel signals (token, registry intent, durable row) so the
//! worker success/persist path cannot fork cancel-vs-complete logic.

use crate::types::{Task, TaskStatus};

/// Outcome of evaluating cancel sources at end-of-process / persist time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelDecision {
    /// Absorb late success — force terminal Cancelled (INV-C1).
    Cancel,
    /// Proceed with success / failure handling.
    Proceed,
}

impl CancelDecision {
    /// Combine cooperative token, in-memory registry intent, and durable task.
    ///
    /// `stored_task` is a fresh read of the durable row (preferred) or the
    /// in-memory worker copy when a re-read is unavailable.
    pub fn from_sources(
        token_cancelled: bool,
        registry_intent: bool,
        stored_task: Option<&Task>,
    ) -> Self {
        if token_cancelled || registry_intent {
            return Self::Cancel;
        }
        if let Some(task) = stored_task {
            if task.cancel_requested_at.is_some()
                || matches!(
                    task.status,
                    TaskStatus::Cancelling | TaskStatus::Cancelled
                )
            {
                return Self::Cancel;
            }
        }
        Self::Proceed
    }

    #[inline]
    pub fn is_cancel(self) -> bool {
        matches!(self, Self::Cancel)
    }

    /// Persist soft-fail: late Indexed write rejected by cancel-wins trigger/guard.
    ///
    /// SSOT for the worker persist path (INV-C1) — do not fork message matching.
    pub fn soft_fail_late_success_persist(task_status: TaskStatus, err_message: &str) -> bool {
        matches!(task_status, TaskStatus::Indexed)
            && is_cancel_wins_transition_rejection(err_message)
    }
}

/// True when a persist rejection is the cancel-wins illegal transition class.
pub fn is_cancel_wins_transition_rejection(message: &str) -> bool {
    let msg = message.to_ascii_lowercase();
    msg.contains("illegal task status transition")
        && (msg.contains("cancelled ->")
            || msg.contains("cancelling ->")
            || msg.contains("-> indexed")
            || msg.contains("-> completed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskType;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample(status: TaskStatus) -> Task {
        let mut t = Task::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        t.status = status;
        t
    }

    #[test]
    fn token_wins() {
        assert!(CancelDecision::from_sources(true, false, None).is_cancel());
    }

    #[test]
    fn registry_wins() {
        assert!(CancelDecision::from_sources(false, true, None).is_cancel());
    }

    #[test]
    fn durable_cancel_requested_wins() {
        let mut t = sample(TaskStatus::Processing);
        t.cancel_requested_at = Some(Utc::now());
        assert!(CancelDecision::from_sources(false, false, Some(&t)).is_cancel());
    }

    #[test]
    fn durable_cancelled_status_wins() {
        let t = sample(TaskStatus::Cancelled);
        assert!(CancelDecision::from_sources(false, false, Some(&t)).is_cancel());
    }

    #[test]
    fn proceed_when_clean() {
        let t = sample(TaskStatus::Processing);
        assert_eq!(
            CancelDecision::from_sources(false, false, Some(&t)),
            CancelDecision::Proceed
        );
    }

    #[test]
    fn detects_illegal_cancelled_to_indexed() {
        assert!(is_cancel_wins_transition_rejection(
            "illegal task status transition: cancelled -> indexed (track_id=x)"
        ));
        assert!(!is_cancel_wins_transition_rejection("storage unavailable"));
    }

    #[test]
    fn soft_fail_late_success_persist_only_indexed() {
        assert!(CancelDecision::soft_fail_late_success_persist(
            TaskStatus::Indexed,
            "illegal task status transition: cancelled -> indexed"
        ));
        assert!(!CancelDecision::soft_fail_late_success_persist(
            TaskStatus::Failed,
            "illegal task status transition: cancelled -> indexed"
        ));
    }
}
