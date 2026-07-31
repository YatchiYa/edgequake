//! Contract: SPEC-120 P3 transparent operation resource.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{
    create_test_app_with_workers, extract_json, TEST_TENANT_ID, TEST_USER_ID, TEST_WORKSPACE_ID,
};
use edgequake_tasks::{Task, TaskType};
use tower::ServiceExt;
use uuid::Uuid;

fn operation_request(method: &str, uri: String) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Tenant-ID", TEST_TENANT_ID)
        .header("X-Workspace-ID", TEST_WORKSPACE_ID)
        .header("X-User-ID", TEST_USER_ID)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn cancel_operation_returns_accepted_transparency_shape() {
    let workers = create_test_app_with_workers().await;
    let task = Task::new(
        Uuid::parse_str(TEST_TENANT_ID).unwrap(),
        Uuid::parse_str(TEST_WORKSPACE_ID).unwrap(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "spec120-operation-cancel" }),
    );
    let id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(operation_request(
            "POST",
            format!("/api/v1/operations/{id}/cancel"),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = extract_json(response).await;
    assert_eq!(body["state"], "cancelled");
    assert!(body["cancel_requested_at"].as_str().is_some());
    assert!(body["expected_stop_by"].as_str().is_some());
}

#[tokio::test]
async fn cancel_processing_operation_returns_cancelling() {
    let workers = create_test_app_with_workers().await;
    let mut task = Task::new(
        Uuid::parse_str(TEST_TENANT_ID).unwrap(),
        Uuid::parse_str(TEST_WORKSPACE_ID).unwrap(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "spec120-operation-draining" }),
    );
    task.mark_processing();
    let id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(operation_request(
            "POST",
            format!("/api/v1/operations/{id}/cancel"),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = extract_json(response).await;
    assert_eq!(body["state"], "cancelling");
    assert!(body["cancel_requested_at"].as_str().is_some());
    assert!(body["expected_stop_by"].as_str().is_some());
}

#[tokio::test]
async fn operation_get_exposes_transparency_fields_and_events_stub() {
    let workers = create_test_app_with_workers().await;
    let task = Task::new(
        Uuid::parse_str(TEST_TENANT_ID).unwrap(),
        Uuid::parse_str(TEST_WORKSPACE_ID).unwrap(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "spec120-operation-get" }),
    );
    let id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(operation_request("GET", format!("/api/v1/operations/{id}")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    assert_eq!(body["track_id"], id);
    assert_eq!(body["document_id"], "spec120-operation-get");
    assert!(
        body["parent_task_id"].is_null(),
        "parent_task_id must be present (null when unbound)"
    );
    assert!(
        body["job_id"].is_null(),
        "job_id must be present (null when unbound)"
    );
    assert!(body.get("presentation").is_some());
    assert!(body["presentation"].get("badge").is_some());
    assert!(body.get("cancel_requested_at").is_some());
    assert!(body.get("available_at").is_some());
    assert!(body.get("superseded_by").is_some());
    // Optional relational join — absent when no documents row (still OK).
    assert!(body.get("document").is_none() || body["document"].is_object());

    let response = workers
        .app()
        .clone()
        .oneshot(operation_request(
            "GET",
            format!("/api/v1/operations/{id}/events"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(extract_json(response).await, serde_json::json!([]));
}

#[tokio::test]
async fn cancel_fenced_deletion_returns_423() {
    let workers = create_test_app_with_workers().await;
    let mut task = Task::new(
        Uuid::parse_str(TEST_TENANT_ID).unwrap(),
        Uuid::parse_str(TEST_WORKSPACE_ID).unwrap(),
        TaskType::Deletion,
        serde_json::json!({
            "document_id": "spec120-fenced-delete",
            "fence_raised": true
        }),
    );
    task.mark_processing();
    let id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(operation_request(
            "POST",
            format!("/api/v1/operations/{id}/cancel"),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::LOCKED);
    let body = extract_json(response).await;
    assert_eq!(body["code"], "operation_fenced");
}

#[tokio::test]
#[cfg(feature = "postgres")]
async fn operation_get_joins_relational_document_projection() {
    use edgequake_api::{create_router, AppState};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    let Ok(base_url) = std::env::var("DATABASE_URL") else {
        eprintln!("Skipping operation document join: DATABASE_URL unset");
        return;
    };
    let url = crate::common::test_db::isolated_test_url(&base_url);
    let Ok(pool) = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&url)
        .await
    else {
        eprintln!("Skipping operation document join: PostgreSQL unavailable");
        return;
    };

    let tenant = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();
    let document = Uuid::new_v4();
    let track = format!("op-proj-{}", Uuid::new_v4());

    // Best-effort seed (tenants/workspaces may already exist in shared test DB).
    let _ = sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(tenant)
    .bind(format!("op-proj-{tenant}"))
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug)
         VALUES ($1, $2, $3, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("op-proj-ws-{workspace}"))
    .execute(&pool)
    .await;

    sqlx::query(
        r#"INSERT INTO documents
           (id, tenant_id, workspace_id, title, content, status, track_id, entity_count, metadata)
           VALUES ($1, $2, $3, 'op proj doc', '', 'processing', $4, 7,
             '{"current_stage":"queued","stage_message":"Waiting for tenant fair-share","stage_progress":0.0}')
           ON CONFLICT (id) DO UPDATE SET
             status = EXCLUDED.status,
             track_id = EXCLUDED.track_id,
             entity_count = EXCLUDED.entity_count,
             metadata = EXCLUDED.metadata"#,
    )
    .bind(document)
    .bind(tenant)
    .bind(workspace)
    .bind(&track)
    .execute(&pool)
    .await
    .expect("seed documents row");

    let state = AppState::test_state_with_pg_pool(pool.clone());
    let mut task = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({ "document_id": document.to_string() }),
    );
    task.track_id = track.clone();
    state.tasks.storage.create_task(&task).await.unwrap();

    let app = create_router(state);
    let response = app
        .oneshot(operation_request(
            "GET",
            format!("/api/v1/operations/{track}"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    assert_eq!(body["document_id"], document.to_string());
    assert_eq!(body["document"]["id"], document.to_string());
    assert_eq!(body["document"]["title"], "op proj doc");
    assert_eq!(body["document"]["status"], "processing");
    assert_eq!(body["document"]["current_stage"], "queued");
    assert_eq!(
        body["document"]["stage_message"],
        "Waiting for tenant fair-share"
    );
    assert_eq!(body["document"]["entity_count"], 7);
    assert_eq!(body["document"]["track_id"], track);
    assert!(body.get("presentation").is_some());

    let _ = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(document)
        .execute(&pool)
        .await;
}
