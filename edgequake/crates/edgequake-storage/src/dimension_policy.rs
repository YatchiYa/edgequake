//! Pure vector-dimension reconcile policy (no I/O, no postgres feature).
//!
//! First principles: pgvector column width is schema truth; empty recreate is
//! heal; non-empty DROP needs an operator flag; boot may PreferExisting.

/// How to reconcile stored column dim vs required embedding dim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionReconcilePolicy {
    /// Workspace / write path: non-empty mismatch errors unless rebuild allowed.
    FailClosed,
    /// Server boot for default NS: keep existing schema; caller rebinds storage dim.
    PreferExisting,
}

/// Outcome of dimension reconcile on a vector table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionEnsureOutcome {
    Matched,
    Recreated,
    /// Non-empty mismatch under [`DimensionReconcilePolicy::PreferExisting`].
    KeptExisting {
        stored: usize,
        required: usize,
    },
}

/// Pure decision (unit-testable; no I/O).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionAction {
    Match,
    /// No column yet — create on initialize.
    CreateLater,
    /// Empty table / schema-only mismatch — safe DROP+CREATE.
    RecreateEmpty,
    /// Operator allowed destructive recreate.
    RecreateAllowed,
    /// Boot: keep stored schema; rebind adapter to `stored`.
    KeepExisting,
    /// Workspace path: refuse wipe.
    FailClosed,
}

/// Decide dimension action from facts (SRP: policy in one place).
pub fn decide_dimension_action(
    stored: Option<usize>,
    required: usize,
    table_empty: bool,
    allow_rebuild: bool,
    policy: DimensionReconcilePolicy,
) -> DimensionAction {
    match stored {
        None => DimensionAction::CreateLater,
        Some(dim) if dim == required => DimensionAction::Match,
        Some(_) if table_empty => DimensionAction::RecreateEmpty,
        Some(_) if allow_rebuild => DimensionAction::RecreateAllowed,
        Some(_) => match policy {
            DimensionReconcilePolicy::PreferExisting => DimensionAction::KeepExisting,
            DimensionReconcilePolicy::FailClosed => DimensionAction::FailClosed,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_match_and_create_later() {
        assert_eq!(
            decide_dimension_action(
                Some(768),
                768,
                false,
                false,
                DimensionReconcilePolicy::FailClosed
            ),
            DimensionAction::Match
        );
        assert_eq!(
            decide_dimension_action(None, 768, true, false, DimensionReconcilePolicy::FailClosed),
            DimensionAction::CreateLater
        );
    }

    #[test]
    fn decide_empty_mismatch_recreates_without_allow() {
        assert_eq!(
            decide_dimension_action(
                Some(1024),
                768,
                true,
                false,
                DimensionReconcilePolicy::FailClosed
            ),
            DimensionAction::RecreateEmpty
        );
    }

    #[test]
    fn decide_nonempty_fail_closed_vs_prefer_existing() {
        assert_eq!(
            decide_dimension_action(
                Some(1024),
                768,
                false,
                false,
                DimensionReconcilePolicy::FailClosed
            ),
            DimensionAction::FailClosed
        );
        assert_eq!(
            decide_dimension_action(
                Some(1024),
                768,
                false,
                false,
                DimensionReconcilePolicy::PreferExisting
            ),
            DimensionAction::KeepExisting
        );
    }

    #[test]
    fn decide_allow_rebuild_wins() {
        assert_eq!(
            decide_dimension_action(
                Some(1024),
                768,
                false,
                true,
                DimensionReconcilePolicy::FailClosed
            ),
            DimensionAction::RecreateAllowed
        );
    }
}
