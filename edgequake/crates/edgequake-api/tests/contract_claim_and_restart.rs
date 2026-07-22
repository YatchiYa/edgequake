//! Contract: SPEC-057 P1 claim / lease restart durability.
//!
//! - Pending survives without channel wake → claim_next returns it
//! - Second claim is None while lease held
//! - Cancelled Pending is never claimed (restart-after-cancel)

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{
    create_test_app_with_workers, extract_json, TEST_TENANT_ID, TEST_USER_ID, TEST_WORKSPACE_ID,
};
use edgequake_tasks::{Task, TaskStatus, TaskType};
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn claim_next_returns_pending_without_channel_wake() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        serde_json::json!({ "document_id": "claim-no-wake" }),
    );
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();
    // Intentionally no queue.send — claim poll / claim_next is SSOT.

    let claimed = workers
        .task_storage
        .claim_next("contract-worker-a", Duration::from_secs(120))
        .await
        .unwrap()
        .expect("Pending must be claimable without channel wake");
    assert_eq!(claimed.track_id, track_id);
    assert_eq!(claimed.status, TaskStatus::Processing);
    assert_eq!(claimed.lease_owner.as_deref(), Some("contract-worker-a"));
    assert!(claimed.lease_token.is_some());

    let second = workers
        .task_storage
        .claim_next("contract-worker-b", Duration::from_secs(120))
        .await
        .unwrap();
    assert!(second.is_none(), "held lease must prevent a second claim");
}

#[tokio::test]
async fn cancelled_pending_never_claimed() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let mut task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        serde_json::json!({ "document_id": "claim-cancelled" }),
    );
    task.mark_cancelled();
    workers.task_storage.create_task(&task).await.unwrap();

    let claimed = workers
        .task_storage
        .claim_next("contract-worker-c", Duration::from_secs(120))
        .await
        .unwrap();
    assert!(
        claimed.is_none(),
        "Cancelled tasks must never be claimed after restart"
    );
}

#[tokio::test]
async fn cancel_pending_then_claim_simulates_restart() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        serde_json::json!({ "document_id": "cancel-then-claim" }),
    );
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tasks/{track_id}/cancel"))
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .header("X-User-ID", TEST_USER_ID)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    assert_eq!(body["status"], "cancelled");

    // Restart simulation: process-local cancel intent may be gone; DB status is SSOT.
    let claimed = workers
        .task_storage
        .claim_next("contract-worker-restart", Duration::from_secs(120))
        .await
        .unwrap();
    assert!(
        claimed.as_ref().map(|t| t.track_id.as_str()) != Some(track_id.as_str()),
        "Cancelled Pending must never be returned by claim_next after restart"
    );
    if let Some(other) = claimed {
        assert_ne!(other.track_id, track_id);
    }

    let stored = workers
        .task_storage
        .get_task(&track_id)
        .await
        .unwrap()
        .expect("task row");
    assert_eq!(stored.status, TaskStatus::Cancelled);
}

#[tokio::test]
async fn refresh_lease_and_release_claim_cas() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        serde_json::json!({}),
    );
    workers.task_storage.create_task(&task).await.unwrap();

    let claimed = workers
        .task_storage
        .claim_next("owner", Duration::from_secs(120))
        .await
        .unwrap()
        .unwrap();
    let token = claimed.lease_token.expect("lease token");

    assert!(workers
        .task_storage
        .refresh_lease(&claimed.track_id, "owner", token, Duration::from_secs(120))
        .await
        .unwrap());
    assert!(!workers
        .task_storage
        .refresh_lease(
            &claimed.track_id,
            "intruder",
            token,
            Duration::from_secs(120)
        )
        .await
        .unwrap());

    assert!(workers
        .task_storage
        .release_claim(&claimed.track_id, "owner", token)
        .await
        .unwrap());
    let pending = workers
        .task_storage
        .get_task(&claimed.track_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(pending.status, TaskStatus::Pending);
    assert!(pending.lease_owner.is_none());
}
