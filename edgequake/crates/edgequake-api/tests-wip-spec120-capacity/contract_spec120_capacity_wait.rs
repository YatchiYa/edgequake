//! SPEC-120 / queue honesty — capacity_wait on pipeline status SSOT.
//!
//! Under provider-pool park (1 processing + fairness-held pendings stamped with
//! ProviderModel), `GET /pipeline/status` must report `capacity_wait=true` so the
//! UI never shows "Workers are idle". Tenant fair-share is opt-in only.

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

fn pipeline_status_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/pipeline/status?tenant_id={TEST_TENANT_ID}&workspace_id={TEST_WORKSPACE_ID}"
        ))
        .header("X-Tenant-ID", TEST_TENANT_ID)
        .header("X-Workspace-ID", TEST_WORKSPACE_ID)
        .header("X-User-ID", TEST_USER_ID)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn pipeline_status_reports_capacity_wait_under_fairness_hold() {
    let workers = create_test_app_with_workers().await;
    let tenant = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let mut processing = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({ "document_id": "capacity-active" }),
    );
    processing.mark_processing();
    // Live lease so lane/presentation treats this as in-flight work.
    processing.lease_expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    processing.lease_owner = Some("worker-test".to_string());
    workers.task_storage.create_task(&processing).await.unwrap();

    let mut held_a = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({ "document_id": "capacity-held-a" }),
    );
    held_a.status = TaskStatus::Pending;
    workers.task_storage.create_task(&held_a).await.unwrap();
    workers
        .task_storage
        .mark_fairness_hold(&held_a.track_id, Duration::from_secs(60))
        .await
        .unwrap();
    let layer = edgequake_tasks::CapacityLayer::ProviderModel {
        provider: "ollama".to_string(),
        model: Some("gemma3".to_string()),
        in_use: 1,
        max: 1,
    };
    edgequake_tasks::stamp_capacity_block(workers.task_storage.as_ref(), &held_a.track_id, &layer)
        .await
        .unwrap();

    let mut held_b = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({ "document_id": "capacity-held-b" }),
    );
    held_b.status = TaskStatus::Pending;
    workers.task_storage.create_task(&held_b).await.unwrap();
    workers
        .task_storage
        .mark_fairness_hold(&held_b.track_id, Duration::from_secs(60))
        .await
        .unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(pipeline_status_request())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;

    assert_eq!(body["processing_tasks"].as_u64(), Some(1));
    assert!(
        body["pending_tasks"].as_u64().unwrap_or(0) >= 2,
        "pending+held waiters must appear in pending_tasks: {body}"
    );
    assert!(
        body["held_or_fairness_held_tasks"].as_u64().unwrap_or(0) >= 2,
        "fairness-held waiters must be counted: {body}"
    );
    assert_eq!(
        body["capacity_wait"].as_bool(),
        Some(true),
        "capacity_wait must be true when processing + held waiters coexist: {body}"
    );
    let reason = body["capacity_wait_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("ollama") || reason.contains("capacity"),
        "capacity_wait_reason must name provider capacity (not tenant fair-share): {body}"
    );
    assert!(
        !reason.contains("tenant fair-share"),
        "default product capacity wait must not claim tenant fair-share: {body}"
    );
    assert_eq!(body["is_busy"].as_bool(), Some(true));

    // Task GET / list presentation must surface named provider capacity badge.
    let task_resp = workers
        .app()
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/tasks/{}?tenant_id={TEST_TENANT_ID}&workspace_id={TEST_WORKSPACE_ID}",
                    held_a.track_id
                ))
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .header("X-User-ID", TEST_USER_ID)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_resp.status(), StatusCode::OK);
    let task_body = extract_json(task_resp).await;
    let badge = task_body["presentation"]["badge"].as_str().unwrap_or("");
    assert!(
        badge.contains("ollama") || badge.to_lowercase().contains("capacity"),
        "held task presentation badge must name provider capacity: {task_body}"
    );
    assert!(
        task_body["capacity_wait_reason"]
            .as_str()
            .unwrap_or("")
            .contains("ollama")
            || task_body["capacity_wait_reason"]
                .as_str()
                .unwrap_or("")
                .contains("capacity"),
        "task capacity_wait_reason must match provider SSOT: {task_body}"
    );
}

/// Opt-in path: when operators re-enable `MAX_TASKS_PER_TENANT`, tenant fair-share
/// stamps and presentation remain honest (regression for SaaS isolation).
#[tokio::test]
async fn pipeline_status_reports_tenant_fair_share_when_opt_in_stamped() {
    let workers = create_test_app_with_workers().await;
    let tenant = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let mut processing = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({ "document_id": "tenant-cap-active" }),
    );
    processing.mark_processing();
    processing.lease_expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    processing.lease_owner = Some("worker-test".to_string());
    workers.task_storage.create_task(&processing).await.unwrap();

    let mut held = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({ "document_id": "tenant-cap-held" }),
    );
    held.status = TaskStatus::Pending;
    workers.task_storage.create_task(&held).await.unwrap();
    workers
        .task_storage
        .mark_fairness_hold(&held.track_id, Duration::from_secs(60))
        .await
        .unwrap();
    let layer = edgequake_tasks::CapacityLayer::TenantFairShare { in_use: 1, max: 1 };
    edgequake_tasks::stamp_capacity_block(workers.task_storage.as_ref(), &held.track_id, &layer)
        .await
        .unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(pipeline_status_request())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    assert_eq!(body["capacity_wait"].as_bool(), Some(true));
    let reason = body["capacity_wait_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("tenant fair-share"),
        "opt-in tenant lane must still surface fair-share wait copy: {body}"
    );
}

#[test]
fn enhanced_pipeline_status_dto_exposes_capacity_fields() {
    let types = include_str!("../src/handlers/pipeline_types.rs");
    assert!(
        types.contains("held_or_fairness_held_tasks"),
        "EnhancedPipelineStatusResponse must expose held_or_fairness_held_tasks"
    );
    assert!(
        types.contains("claimable_pending_tasks"),
        "EnhancedPipelineStatusResponse must expose claimable_pending_tasks"
    );
    assert!(
        types.contains("capacity_wait"),
        "EnhancedPipelineStatusResponse must expose capacity_wait"
    );
    assert!(
        types.contains("capacity_wait_reason"),
        "EnhancedPipelineStatusResponse must expose capacity_wait_reason"
    );
}

#[test]
fn openapi_snapshot_includes_capacity_wait_fields() {
    let snapshot = include_str!("../../../../edgequake_webui/openapi/openapi.snapshot.json");
    assert!(
        snapshot.contains("\"capacity_wait\""),
        "OpenAPI snapshot must include capacity_wait"
    );
    assert!(
        snapshot.contains("\"capacity_wait_reason\""),
        "OpenAPI snapshot must include capacity_wait_reason"
    );
    assert!(
        snapshot.contains("\"held_or_fairness_held_tasks\""),
        "OpenAPI snapshot must include held_or_fairness_held_tasks"
    );
    assert!(
        snapshot.contains("\"claimable_pending_tasks\""),
        "OpenAPI snapshot must include claimable_pending_tasks"
    );
    // StatisticsInfo on /tasks also carries capacity fields for pipeline fallback.
    assert!(
        snapshot.contains("\"held_or_fairness_held\""),
        "OpenAPI snapshot must include StatisticsInfo.held_or_fairness_held"
    );
}

#[test]
fn health_soft_degrades_on_queue_pressure_timeout() {
    let health = include_str!("../src/handlers/health.rs");
    assert!(
        health.contains("soft-degrading"),
        "health must soft-degrade when queue pressure counts time out"
    );
    assert!(
        health.contains("get_queue_pressure_counts"),
        "health must use lightweight get_queue_pressure_counts"
    );
    assert!(
        health.contains("\"unknown\""),
        "soft-degrade must emit pressure=unknown rather than dropping operational"
    );
}
