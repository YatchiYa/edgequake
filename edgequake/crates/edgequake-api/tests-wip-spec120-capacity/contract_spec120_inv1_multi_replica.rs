//! SPEC-120 INV-1: cancel reaches a task owned by another replica within one heartbeat.
//!
//! NOTIFY is suppressed (`NoopCancelWake`). The owning replica discovers cancel
//! solely via `refresh_lease` → `LeaseVerdict::CancelRequested`.

use std::sync::Arc;
use std::time::Duration;

use edgequake_api::services::task_cancel::{apply_task_row_cancel_with_wake, NoopCancelWake};
use edgequake_tasks::{
    memory::MemoryTaskStorage, CancellationRegistry, LeaseVerdict, SharedTaskStorage, Task,
    TaskStatus, TaskType, DEFAULT_LEASE_HEARTBEAT_SECS,
};
use uuid::Uuid;

const _: () = assert!(
    DEFAULT_LEASE_HEARTBEAT_SECS <= 35,
    "heartbeat budget for INV-1 is at most 35s"
);

#[tokio::test]
async fn inv1_cancel_without_notify_stops_via_heartbeat() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    // Replica A and B share durable storage; each has its own registry cache.
    let registry_a = CancellationRegistry::new();
    let registry_b = CancellationRegistry::new();

    let task = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "inv1-doc" }),
    );
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.unwrap();

    // Replica B claims and registers a local token (NOTIFY path will be skipped).
    let claimed = storage
        .claim_next("replica-b", Duration::from_secs(120))
        .await
        .unwrap()
        .expect("task must be claimable");
    let lease_token = claimed.lease_token.expect("claimed task has lease token");
    let _token_b = registry_b.register(&track_id).await;
    assert!(!registry_b.is_cancelled(&track_id).await);

    // Replica A accepts cancel with NOTIFY suppressed.
    let applied =
        apply_task_row_cancel_with_wake(&storage, &registry_a, &track_id, &NoopCancelWake)
            .await
            .unwrap();
    assert!(applied.cancelling);
    assert!(applied.cancel_requested_at.is_some());

    // Replica B has not yet seen the intent via registry (NOTIFY suppressed).
    assert!(
        !registry_b.is_cancelled(&track_id).await,
        "NOTIFY suppressed: local registry B must not be woken yet"
    );

    // Within one heartbeat, lease refresh on B discovers durable cancel.
    let verdict = storage
        .refresh_lease(
            &track_id,
            "replica-b",
            lease_token,
            Duration::from_secs(120),
        )
        .await
        .unwrap();
    assert_eq!(verdict, LeaseVerdict::CancelRequested);
    let stored = storage.get_task(&track_id).await.unwrap().unwrap();
    assert_eq!(stored.status, TaskStatus::Cancelling);
    assert!(stored.cancel_requested_at.is_some());
}

#[tokio::test]
async fn inv1_expected_stop_by_is_one_heartbeat_from_request() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    let registry = CancellationRegistry::new();
    let mut task = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({}),
    );
    task.mark_processing();
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.unwrap();

    let applied = apply_task_row_cancel_with_wake(&storage, &registry, &track_id, &NoopCancelWake)
        .await
        .unwrap();

    let at = applied.cancel_requested_at.expect("intent timestamp");
    let stop_by = applied.expected_stop_by.expect("soft deadline");
    let delta = (stop_by - at).num_seconds();
    assert_eq!(delta, DEFAULT_LEASE_HEARTBEAT_SECS as i64);
}
