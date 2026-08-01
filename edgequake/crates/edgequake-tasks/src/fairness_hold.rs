//! Durable fairness hold — claim-invisible park (SPEC-057 INV-06 FP-1 / FP-5).
//!
//! WHY: Process-local `FairnessParkSet` alone leaves Pending rows reclaimable,
//! causing claim/release storms. Storage marks `fairness_hold_until` so
//! `claim_next` skips held work until park wake or TTL expiry.

use std::time::Duration;

use crate::types::{FairnessClass, TaskType};

/// Default hold TTL when the worker does not pass an explicit duration.
///
/// Long enough to cover park wait under normal ingest; short enough that a
/// crashed park waiter does not strand work forever (claim becomes eligible
/// again when `fairness_hold_until <= now()`).
pub const DEFAULT_FAIRNESS_HOLD_TTL: Duration = Duration::from_secs(30);

/// Claim-time fairness policy (FP-2): prefer tenants under configured lane caps.
///
/// `0` for a lane means “no capacity preference for that lane” (unlimited /
/// preference disabled). Holds are always excluded regardless of these values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClaimFairnessPolicy {
    /// Max concurrent ingest tasks per tenant (`MAX_TASKS_PER_TENANT`). `0` = no prefer.
    pub max_ingest_per_tenant: usize,
    /// Max concurrent lifecycle tasks per tenant. `0` = no prefer.
    pub max_lifecycle_per_tenant: usize,
}

impl ClaimFairnessPolicy {
    /// Build from worker pool lane caps.
    pub fn from_lane_caps(max_ingest: usize, max_lifecycle: usize) -> Self {
        Self {
            max_ingest_per_tenant: max_ingest,
            max_lifecycle_per_tenant: max_lifecycle,
        }
    }

    /// Cap for a fairness class (`0` = preference off for that class).
    pub fn max_for_class(self, class: FairnessClass) -> usize {
        match class {
            FairnessClass::Ingest => self.max_ingest_per_tenant,
            FairnessClass::Lifecycle => self.max_lifecycle_per_tenant,
        }
    }
}

/// SQL / memory predicate: task types on the lifecycle fairness lane.
pub fn is_lifecycle_task_type(task_type: TaskType) -> bool {
    matches!(task_type.fairness_class(), FairnessClass::Lifecycle)
}

/// Lifecycle-lane task types — SSOT with [`TaskType::fairness_class`].
pub const LIFECYCLE_TASK_TYPES: &[TaskType] = &[
    TaskType::Deletion,
    TaskType::BatchDeletion,
    TaskType::WorkspaceWipe,
];

/// Postgres `IN (...)` fragment for the lifecycle lane (derived from [`LIFECYCLE_TASK_TYPES`]).
pub fn lifecycle_task_type_sql() -> String {
    LIFECYCLE_TASK_TYPES
        .iter()
        .map(|t| format!("'{t}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_from_lane_caps() {
        let p = ClaimFairnessPolicy::from_lane_caps(2, 4);
        assert_eq!(p.max_for_class(FairnessClass::Ingest), 2);
        assert_eq!(p.max_for_class(FairnessClass::Lifecycle), 4);
    }

    #[test]
    fn lifecycle_types_match_fairness_class() {
        assert!(is_lifecycle_task_type(TaskType::Deletion));
        assert!(is_lifecycle_task_type(TaskType::BatchDeletion));
        assert!(is_lifecycle_task_type(TaskType::WorkspaceWipe));
        assert!(!is_lifecycle_task_type(TaskType::Insert));
        assert!(!is_lifecycle_task_type(TaskType::PdfProcessing));
    }

    #[test]
    fn lifecycle_sql_ssot_covers_all_lifecycle_variants() {
        let sql = lifecycle_task_type_sql();
        for t in LIFECYCLE_TASK_TYPES {
            assert!(sql.contains(&format!("'{t}'")), "SQL missing {t}: {sql}");
            assert_eq!(t.fairness_class(), FairnessClass::Lifecycle);
        }
        // Every TaskType mapped to Lifecycle must appear in the SSOT list.
        for t in [
            TaskType::Upload,
            TaskType::Insert,
            TaskType::Scan,
            TaskType::Reindex,
            TaskType::PdfProcessing,
            TaskType::KnowledgeInjection,
            TaskType::Deletion,
            TaskType::BatchDeletion,
            TaskType::WorkspaceWipe,
        ] {
            let in_list = LIFECYCLE_TASK_TYPES.contains(&t);
            assert_eq!(
                in_list,
                t.fairness_class() == FairnessClass::Lifecycle,
                "LIFECYCLE_TASK_TYPES out of sync with fairness_class for {t}"
            );
        }
    }
}
