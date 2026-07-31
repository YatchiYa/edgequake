//! Claim eligibility SSOT (SPEC-057 INV-06 / INV-Q2 / INV-Q3).
//!
//! Single source of truth for:
//! - which rows may be claimed (`is_claimable` / SQL fragments)
//! - which rows consume tenant lane capacity (`counts_toward_lane`)
//! - the shared Memory↔Postgres case matrix (`eligibility_matrix`)
//!
//! Memory and Postgres `claim_next` MUST use these helpers so the claimable set
//! cannot drift (held-claim deadlock class).

use chrono::{DateTime, Utc};

use crate::types::{Task, TaskStatus};

/// True when the task may be selected by `claim_next` at `now`.
///
/// Claimable:
/// - `pending` | `held` with inactive fairness hold and available_at due
/// - `processing` with expired/missing lease (and inactive hold)
pub fn is_claimable(task: &Task, now: DateTime<Utc>) -> bool {
    if task.is_fairness_held(now) {
        return false;
    }
    if task
        .available_at
        .is_some_and(|available_at| available_at > now)
    {
        return false;
    }
    match task.status {
        TaskStatus::Pending | TaskStatus::Held => true,
        TaskStatus::Processing => task.lease_is_expired(now),
        _ => false,
    }
}

/// True when the task occupies a tenant fairness lane at `now` (INV-Q3).
///
/// Counts:
/// - live `processing` / `cancelling` lease
/// - *active* fairness hold on pending/held
///
/// Does **not** count expired/null-hold `held` rows (those are claimable and
/// must not permanently saturate the tenant).
pub fn counts_toward_lane(task: &Task, now: DateTime<Utc>) -> bool {
    match task.status {
        TaskStatus::Processing | TaskStatus::Cancelling => !task.lease_is_expired(now),
        TaskStatus::Pending | TaskStatus::Held => task.is_fairness_held(now),
        _ => false,
    }
}

/// SQL predicate (no leading AND) for claimable pending/held rows.
///
/// Used by fair_pick `bounded_pending` and claim_arm candidate filters.
pub fn claimable_pending_sql() -> &'static str {
    "status IN ('pending', 'held') \
     AND (fairness_hold_until IS NULL OR fairness_hold_until <= NOW()) \
     AND (available_at IS NULL OR available_at <= NOW())"
}

/// SQL predicate for claimable pending/held with table alias `t2.`
pub fn claimable_pending_sql_t2() -> &'static str {
    "t2.status IN ('pending', 'held') \
     AND (t2.fairness_hold_until IS NULL OR t2.fairness_hold_until <= NOW()) \
     AND (t2.available_at IS NULL OR t2.available_at <= NOW())"
}

/// SQL predicate for stale processing reclaim candidates (fair_pick bounded_stale).
pub fn claimable_stale_processing_sql() -> &'static str {
    "status = 'processing' \
     AND (lease_expires_at IS NULL OR lease_expires_at < NOW()) \
     AND (fairness_hold_until IS NULL OR fairness_hold_until <= NOW())"
}

/// SQL predicate for stale processing with alias `t2.`
pub fn claimable_stale_processing_sql_t2() -> &'static str {
    "t2.status = 'processing' \
     AND (t2.lease_expires_at IS NULL OR t2.lease_expires_at < NOW()) \
     AND (t2.fairness_hold_until IS NULL OR t2.fairness_hold_until <= NOW()) \
     AND (t2.available_at IS NULL OR t2.available_at <= NOW())"
}

/// SQL for the active-hold arm of tenant lane_load (pending|held with live TTL).
pub fn active_lane_hold_sql() -> &'static str {
    "status IN ('pending', 'held') \
     AND fairness_hold_until IS NOT NULL \
     AND fairness_hold_until > NOW()"
}

/// SQL for live processing/cancelling leases in lane_load.
pub fn active_lane_lease_sql() -> &'static str {
    "status IN ('processing', 'cancelling') \
     AND lease_expires_at IS NOT NULL \
     AND lease_expires_at >= NOW()"
}

/// Alias: pending/held rows parked by an active fairness hold (queue presentation).
///
/// Identical to [`active_lane_hold_sql`] — exposed under the presentation name so
/// statistics / pipeline status do not reinvent the predicate.
pub fn held_or_fairness_held_sql() -> &'static str {
    active_lane_hold_sql()
}

/// True when a pending/held task is capacity-parked (active fairness hold).
pub fn is_held_or_fairness_held(task: &Task, now: DateTime<Utc>) -> bool {
    matches!(task.status, TaskStatus::Pending | TaskStatus::Held)
        && task.is_fairness_held(now)
}

/// Honest capacity-wait signal for UI (never "workers idle").
///
/// True when at least one task is processing **and** at least one waiter is
/// fairness-held / held — typical under `MAX_TASKS_PER_TENANT=1`.
pub fn capacity_wait(processing: u64, held_or_fairness_held: u64) -> bool {
    processing > 0 && held_or_fairness_held > 0
}

/// Fairness-hold fixture for the shared eligibility matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldSpec {
    None,
    /// Hold still active (`now + secs`).
    ActiveSecs(i64),
    /// Hold already expired (`now - secs`).
    ExpiredSecs(i64),
}

/// Lease fixture for the shared eligibility matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseSpec {
    None,
    LiveSecs(i64),
    ExpiredSecs(i64),
}

/// One row of the Memory↔Postgres claim-eligibility matrix (INV-Q2).
#[derive(Debug, Clone, Copy)]
pub struct EligibilityCase {
    pub name: &'static str,
    pub status: TaskStatus,
    pub hold: HoldSpec,
    pub lease: LeaseSpec,
    /// When set, `available_at = now + secs` (blocks claim while future).
    pub available_future_secs: Option<i64>,
    pub expect_claimable: bool,
    pub expect_lane: bool,
}

/// Shared case table — pure predicates and both backends must agree.
pub fn eligibility_matrix() -> &'static [EligibilityCase] {
    &[
        EligibilityCase {
            name: "pending",
            status: TaskStatus::Pending,
            hold: HoldSpec::None,
            lease: LeaseSpec::None,
            available_future_secs: None,
            expect_claimable: true,
            expect_lane: false,
        },
        EligibilityCase {
            name: "held_active_hold",
            status: TaskStatus::Held,
            hold: HoldSpec::ActiveSecs(30),
            lease: LeaseSpec::None,
            available_future_secs: None,
            expect_claimable: false,
            expect_lane: true,
        },
        EligibilityCase {
            name: "held_expired_hold",
            status: TaskStatus::Held,
            hold: HoldSpec::ExpiredSecs(1),
            lease: LeaseSpec::None,
            available_future_secs: None,
            expect_claimable: true,
            expect_lane: false,
        },
        EligibilityCase {
            name: "held_null_hold",
            status: TaskStatus::Held,
            hold: HoldSpec::None,
            lease: LeaseSpec::None,
            available_future_secs: None,
            expect_claimable: true,
            expect_lane: false,
        },
        EligibilityCase {
            name: "processing_live_lease",
            status: TaskStatus::Processing,
            hold: HoldSpec::None,
            lease: LeaseSpec::LiveSecs(60),
            available_future_secs: None,
            expect_claimable: false,
            expect_lane: true,
        },
        EligibilityCase {
            name: "processing_expired_lease",
            status: TaskStatus::Processing,
            hold: HoldSpec::None,
            lease: LeaseSpec::ExpiredSecs(1),
            available_future_secs: None,
            expect_claimable: true,
            expect_lane: false,
        },
        EligibilityCase {
            name: "pending_available_at_future",
            status: TaskStatus::Pending,
            hold: HoldSpec::None,
            lease: LeaseSpec::None,
            available_future_secs: Some(10),
            expect_claimable: false,
            expect_lane: false,
        },
    ]
}

/// Build a task matching `case` at `now` (no storage I/O).
pub fn materialize_case(
    case: &EligibilityCase,
    now: DateTime<Utc>,
    tenant: uuid::Uuid,
    workspace: uuid::Uuid,
) -> Task {
    use crate::types::TaskType;

    let mut task = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));
    task.status = case.status;
    task.fairness_hold_until = match case.hold {
        HoldSpec::None => None,
        HoldSpec::ActiveSecs(s) => Some(now + chrono::Duration::seconds(s)),
        HoldSpec::ExpiredSecs(s) => Some(now - chrono::Duration::seconds(s)),
    };
    task.lease_expires_at = match case.lease {
        LeaseSpec::None => None,
        LeaseSpec::LiveSecs(s) => Some(now + chrono::Duration::seconds(s)),
        LeaseSpec::ExpiredSecs(s) => Some(now - chrono::Duration::seconds(s)),
    };
    if matches!(case.lease, LeaseSpec::LiveSecs(_) | LeaseSpec::ExpiredSecs(_)) {
        task.lease_owner = Some("matrix-owner".into());
        task.lease_token = Some(uuid::Uuid::new_v4());
    }
    task.available_at = case
        .available_future_secs
        .map(|s| now + chrono::Duration::seconds(s));
    task
}

/// Assert pure predicates for one matrix row.
pub fn assert_eligibility_predicates(case: &EligibilityCase, now: DateTime<Utc>) {
    let task = materialize_case(case, now, uuid::Uuid::nil(), uuid::Uuid::nil());
    assert_eq!(
        is_claimable(&task, now),
        case.expect_claimable,
        "{}: is_claimable",
        case.name
    );
    assert_eq!(
        counts_toward_lane(&task, now),
        case.expect_lane,
        "{}: counts_toward_lane",
        case.name
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_predicates_cover_all_cases() {
        let now = Utc::now();
        let rows = eligibility_matrix();
        assert!(
            rows.len() >= 7,
            "matrix must cover pending/held/stale/available_at"
        );
        for case in rows {
            assert_eligibility_predicates(case, now);
        }
    }

    #[test]
    fn sql_fragments_are_non_empty() {
        assert!(claimable_pending_sql().contains("pending"));
        assert!(claimable_pending_sql_t2().contains("t2.status"));
        assert!(active_lane_hold_sql().contains("fairness_hold_until > NOW()"));
        assert!(held_or_fairness_held_sql().contains("fairness_hold_until > NOW()"));
        assert!(active_lane_lease_sql().contains("processing"));
    }

    #[test]
    fn capacity_wait_requires_processing_and_held_waiters() {
        assert!(!capacity_wait(0, 3));
        assert!(!capacity_wait(1, 0));
        assert!(capacity_wait(1, 3));
    }
}
