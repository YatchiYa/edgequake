//! Contract / E2E: task cancel intent + fairness park wiring.
//!
//! Validates the P0–P3 remediation end-to-end through the HTTP API:
//! - Pending task cancel → Cancelled status + cancel intent
//! - Cancelled is idempotent
//! - Indexed rejects cancel (409)
//! - Worker-backed app exposes a shared tenant fairness limiter

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{
    create_test_app_with_workers, extract_json, TEST_TENANT_ID, TEST_USER_ID, TEST_WORKSPACE_ID,
};
use edgequake_tasks::{Task, TaskStatus, TaskType};
use tower::ServiceExt;
use uuid::Uuid;

async fn post_cancel(app: &axum::Router, track_id: &str) -> axum::response::Response {
    app.clone()
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
        .unwrap()
}

async fn get_task(app: &axum::Router, track_id: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tasks/{track_id}"))
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .header("X-User-ID", TEST_USER_ID)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn e2e_cancel_pending_task_persists_cancelled_and_intent() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        serde_json::json!({ "document_id": "cancel-contract-doc" }),
    );
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = post_cancel(workers.app(), &track_id).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    assert_eq!(body["status"], "cancelled");
    assert_eq!(body["track_id"], track_id);

    let stored = workers
        .task_storage
        .get_task(&track_id)
        .await
        .unwrap()
        .expect("task row");
    assert_eq!(stored.status, TaskStatus::Cancelled);
    assert!(
        workers
            .cancellation_registry
            .has_cancel_intent(&track_id)
            .await,
        "cancel intent must be recorded so parked/queued copies are skipped"
    );

    // Idempotent re-cancel
    let again = post_cancel(workers.app(), &track_id).await;
    assert_eq!(again.status(), StatusCode::OK);
    let again_body = extract_json(again).await;
    assert_eq!(again_body["status"], "cancelled");
}

#[tokio::test]
async fn e2e_cancel_indexed_task_conflicts() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let mut task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        serde_json::json!({}),
    );
    task.mark_success(serde_json::json!({ "ok": true }));
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = post_cancel(workers.app(), &track_id).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert!(
        !workers
            .cancellation_registry
            .has_cancel_intent(&track_id)
            .await,
        "Indexed tasks must not receive cancel intent"
    );

    let get = get_task(workers.app(), &track_id).await;
    assert_eq!(get.status(), StatusCode::OK);
    let body = extract_json(get).await;
    assert_eq!(body["status"], "indexed");
}

#[tokio::test]
async fn e2e_worker_app_wires_tenant_fairness_limiter() {
    let workers = create_test_app_with_workers().await;
    // Queue metrics should report the shared limiter (max_tasks_per_tenant > 0).
    let response = workers
        .app()
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/pipeline/queue-metrics")
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    let max = body["max_tasks_per_tenant"]
        .as_u64()
        .expect("max_tasks_per_tenant present");
    assert!(
        max > 0,
        "test worker pool must expose fairness limiter (got {max})"
    );
    assert!(body["tenant_park_waiters"].as_u64().is_some());
    assert!(body["cancel_intent_count"].as_u64().is_some());
}
