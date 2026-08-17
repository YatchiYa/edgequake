//! SPEC-091 QW0 — Task state machine (LAW-Q2: single transition authority).
//!
//! Every task-status mutation is a [`TaskEvent`] applied through
//! [`transition`]. Illegal transitions are unrepresentable: they return
//! [`TransitionError`] and leave the task unchanged. The SQL claim/release
//! sites enforce the *same* guards under concurrency via the
//! [`CLAIM_PENDING_GUARD_SQL`], [`CLAIM_STALE_GUARD_SQL`], and
//! [`RELEASE_GUARD_SQL`] fragments — one table, two surfaces, drift-tested.
//!
//! Spec: `specs/091-simplify-data-layer/13-queue-admission-target-spec.md`.
//!
//! ## Transition table (exhaustive — anything unlisted is an error)
//!
//! ```text
//!  FROM        │Enqueue│Claim │Complete│Fail │RetryRequeue│Reprocess│Cancel │LeaseLost│Release
//!  ────────────┼───────┼──────┼────────┼──────┼────────────┼─────────┼───────┼─────────┼───────
//!  (none)      │pending│  ✗   │   ✗    │  ✗   │     ✗      │    ✗    │   ✗   │    ✗    │   ✗
//!  pending     │  ✗    │proc. │   ✗    │failed│     ✗      │    ✗    │cancel │    ✗    │   ✗
//!  processing  │  ✗    │proc.*│indexed │failed│     ✗      │    ✗    │cancel │ pending │pending
//!  indexed     │  ✗    │  ✗   │   ✗    │  ✗   │     ✗      │    ✗    │   ✗   │    ✗    │   ✗
//!  failed      │  ✗    │  ✗   │   ✗    │failed│  pending   │ pending │cancel │    ✗    │   ✗
//!  cancelled   │  ✗    │  ✗   │   ✗    │  ✗   │     ✗      │    ✗    │   ✗   │    ✗    │   ✗
//!
//!  * Claim from `processing` is the stale-lease reclaim arm — the pure table
//!    allows the edge; the SQL guard proves staleness (lease expired).
//!    `Fail` from `pending`/`failed` covers fail-before-start and per-attempt
//!    re-failure bookkeeping (retry_count is incremented per attempt).
//! ```

use std::fmt;

use crate::types::TaskStatus;

/// The only legal mutation verbs for a task's persisted status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskEvent {
    /// Admission accepted: (none) → pending.
    Enqueue,
    /// Worker claim: pending → processing, or stale-processing → processing.
    Claim,
    /// Success: processing → indexed.
    Complete,
    /// Failure recorded: pending|processing|failed → failed (per-attempt).
    Fail,
    /// Automatic retry requeue after a retryable failure: failed → pending.
    RetryRequeue,
    /// Operator-initiated reprocess of a terminally failed task: failed → pending.
    Reprocess,
    /// Cancel intent honored at a stage boundary: pending|processing|failed → cancelled.
    Cancel,
    /// Lease lost (stale reclaim, boot orphan auto-resume): processing → pending.
    LeaseLost,
    /// Voluntary release (fairness park, byte-budget reject): processing → pending.
    Release,
}

impl fmt::Display for TaskEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Enqueue => "enqueue",
            Self::Claim => "claim",
            Self::Complete => "complete",
            Self::Fail => "fail",
            Self::RetryRequeue => "retry_requeue",
            Self::Reprocess => "reprocess",
            Self::Cancel => "cancel",
            Self::LeaseLost => "lease_lost",
            Self::Release => "release",
        };
        write!(f, "{s}")
    }
}

/// Illegal transition attempt — the task is left unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionError {
    pub from: Option<TaskStatus>,
    pub event: TaskEvent,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "illegal task transition: event '{}' from state {:?}",
            self.event, self.from
        )
    }
}

impl std::error::Error for TransitionError {}

/// All events, for exhaustive tests.
pub const ALL_EVENTS: [TaskEvent; 9] = [
    TaskEvent::Enqueue,
    TaskEvent::Claim,
    TaskEvent::Complete,
    TaskEvent::Fail,
    TaskEvent::RetryRequeue,
    TaskEvent::Reprocess,
    TaskEvent::Cancel,
    TaskEvent::LeaseLost,
    TaskEvent::Release,
];

/// All persisted states plus the pre-birth `None`, for exhaustive tests.
pub const ALL_FROM_STATES: [Option<TaskStatus>; 6] = [
    None,
    Some(TaskStatus::Pending),
    Some(TaskStatus::Processing),
    Some(TaskStatus::Indexed),
    Some(TaskStatus::Failed),
    Some(TaskStatus::Cancelled),
];

/// The pure transition table — the single definition of legal status changes.
///
/// Concurrency-critical guards (claim staleness) are enforced in SQL via the
/// guard fragments below; this function encodes *which edges exist*, not the
/// lease arithmetic.
pub fn transition(
    from: Option<TaskStatus>,
    event: TaskEvent,
) -> Result<TaskStatus, TransitionError> {
    use TaskEvent::*;
    use TaskStatus::*;
    let next = match (from, event) {
        (None, Enqueue) => Pending,
        (Some(Pending), Claim) => Processing,
        // Stale-lease reclaim arm — SQL guard enforces lease_expires_at < now().
        (Some(Processing), Claim) => Processing,
        (Some(Processing), Complete) => Indexed,
        (Some(Pending), Fail) | (Some(Processing), Fail) | (Some(Failed), Fail) => Failed,
        (Some(Failed), RetryRequeue) => Pending,
        (Some(Failed), Reprocess) => Pending,
        (Some(Pending), Cancel) | (Some(Processing), Cancel) | (Some(Failed), Cancel) => Cancelled,
        (Some(Processing), LeaseLost) => Pending,
        (Some(Processing), Release) => Pending,
        _ => return Err(TransitionError { from, event }),
    };
    Ok(next)
}

/// True when `transition(from, event)` is legal.
pub fn is_legal(from: Option<TaskStatus>, event: TaskEvent) -> bool {
    transition(from, event).is_ok()
}

/// SQL `WHERE` fragment for the pending claim arm (`Claim` from `pending`).
/// Fairness-parked rows are excluded at the SSOT level (LAW-Q2/Q5): the park
/// marker is volatile scheduling state set atomically with claim release,
/// cleared before the park waiter's queue re-wake (migration 111).
pub const CLAIM_PENDING_GUARD_SQL: &str = "status = 'pending' AND fairness_parked_at IS NULL";

/// SQL `WHERE` fragment for the stale-lease reclaim arm (`Claim` from
/// `processing`, staleness proven — the reclaim edge's guard).
pub const CLAIM_STALE_GUARD_SQL: &str =
    "status = 'processing' AND (lease_expires_at IS NULL OR lease_expires_at < NOW())";

/// SQL `WHERE` fragment for `Release` (processing → pending under lease CAS).
pub const RELEASE_GUARD_SQL: &str = "status = 'processing'";

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-091 QW0 / F-091-17, LAW-Q2: every (from × event) cell of the
    /// transition table behaves exactly as specified.
    #[test]
    fn contract_spec091_state_machine_transitions() {
        use TaskEvent::*;
        use TaskStatus::*;
        let expected: &[(Option<TaskStatus>, TaskEvent, Option<TaskStatus>)] = &[
            (None, Enqueue, Some(Pending)),
            (Some(Pending), Claim, Some(Processing)),
            (Some(Processing), Claim, Some(Processing)),
            (Some(Processing), Complete, Some(Indexed)),
            (Some(Pending), Fail, Some(Failed)),
            (Some(Processing), Fail, Some(Failed)),
            (Some(Failed), Fail, Some(Failed)),
            (Some(Failed), RetryRequeue, Some(Pending)),
            (Some(Failed), Reprocess, Some(Pending)),
            (Some(Pending), Cancel, Some(Cancelled)),
            (Some(Processing), Cancel, Some(Cancelled)),
            (Some(Failed), Cancel, Some(Cancelled)),
            (Some(Processing), LeaseLost, Some(Pending)),
            (Some(Processing), Release, Some(Pending)),
        ];
        // Every legal cell produces the specified next state.
        for (from, event, want) in expected {
            assert_eq!(
                transition(*from, *event).ok(),
                *want,
                "transition({from:?}, {event}) mismatch"
            );
        }
        // Every cell NOT listed is illegal (exhaustive 6 × 9 matrix).
        let legal: std::collections::HashSet<(Option<TaskStatus>, TaskEvent)> =
            expected.iter().map(|(f, e, _)| (*f, *e)).collect();
        for from in ALL_FROM_STATES {
            for event in ALL_EVENTS {
                if legal.contains(&(from, event)) {
                    continue;
                }
                assert!(
                    transition(from, event).is_err(),
                    "transition({from:?}, {event}) must be illegal"
                );
            }
        }
    }

    /// Terminal states have no outgoing edges.
    #[test]
    fn contract_spec091_terminal_states_closed() {
        for event in ALL_EVENTS {
            assert!(transition(Some(TaskStatus::Indexed), event).is_err());
            assert!(transition(Some(TaskStatus::Cancelled), event).is_err());
        }
    }

    /// SPEC-091 QW0: SQL guard fragments encode the same table as `transition` —
    /// they cannot drift apart.
    #[test]
    fn contract_spec091_state_machine_sql_guard_drift() {
        // Pending claim arm: table says Claim is legal from Pending.
        assert!(is_legal(Some(TaskStatus::Pending), TaskEvent::Claim));
        assert!(CLAIM_PENDING_GUARD_SQL.contains("status = 'pending'"));
        assert!(!CLAIM_PENDING_GUARD_SQL.contains("processing"));
        // Stale arm: table says Claim is legal from Processing; guard must
        // restrict to proven-stale leases only.
        assert!(is_legal(Some(TaskStatus::Processing), TaskEvent::Claim));
        assert!(CLAIM_STALE_GUARD_SQL.contains("status = 'processing'"));
        assert!(CLAIM_STALE_GUARD_SQL.contains("lease_expires_at"));
        // Release: table says Release is legal ONLY from Processing.
        for state in ALL_FROM_STATES {
            let legal = is_legal(state, TaskEvent::Release);
            assert_eq!(legal, state == Some(TaskStatus::Processing));
        }
        assert_eq!(RELEASE_GUARD_SQL, "status = 'processing'");
    }

    /// SPEC-091 QW0 / F-091-17: no raw status mutation survives outside the
    /// SSOT. The worker must not assign `task.status` directly, and the
    /// Postgres claim/release SQL must embed the guard fragments above
    /// (rewriting the guards as literals deletes the constant names → fail).
    #[test]
    fn contract_spec091_state_machine_no_raw_mutation() {
        let worker_src = include_str!("worker.rs");
        let banned = [".s", "tatus = TaskStatus::"].concat();
        assert!(
            !worker_src.contains(&banned),
            "worker.rs must route status changes through Task state-machine methods"
        );

        let pg_src = include_str!("postgres.rs");
        for constant in [
            "CLAIM_PENDING_GUARD_SQL",
            "CLAIM_STALE_GUARD_SQL",
            "RELEASE_GUARD_SQL",
        ] {
            assert!(
                pg_src.contains(constant),
                "postgres.rs claim/release SQL must embed state_machine::{constant}"
            );
        }

        // Boot recovery is in the API crate (sibling layout, stable workspace).
        let orphan_src = include_str!("../../edgequake-api/src/services/orphan_task_recovery.rs");
        assert!(
            !orphan_src.contains(&banned),
            "orphan_task_recovery.rs must use Task::recover_to_pending / fail_orphaned"
        );
    }
}
