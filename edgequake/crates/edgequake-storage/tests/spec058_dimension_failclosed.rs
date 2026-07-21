//! E2E / contract: vector dimension reconcile (SPEC-058 + PreferExisting boot).
//!
//! @implements SPEC-058 fail-closed · empty heal · PreferExisting at AppState boot

#[test]
fn contract_policy_module_exports_prefer_existing() {
    use edgequake_storage::{decide_dimension_action, DimensionAction, DimensionReconcilePolicy};

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
            true,
            false,
            DimensionReconcilePolicy::FailClosed
        ),
        DimensionAction::RecreateEmpty
    );
}

#[test]
fn contract_ensure_dimension_fail_closed_source() {
    let src = include_str!("../src/adapters/postgres/vector/migration.rs");
    assert!(src.contains("EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD"));
    assert!(src.contains("Refusing DROP TABLE"));
    assert!(src.contains("PreferExisting"));
    assert!(src.contains("RecreateEmpty"));
    assert!(src.contains(r#""1" | "true" | "yes" | "on""#));
}

#[test]
fn contract_boot_uses_prefer_existing() {
    let src = include_str!("../../edgequake-api/src/state/postgres.rs");
    assert!(
        src.contains("DimensionReconcilePolicy::PreferExisting"),
        "AppState::new_postgres must PreferExisting on default vector table"
    );
    assert!(
        src.contains("KeptExisting"),
        "boot must rebind on KeptExisting"
    );
}
