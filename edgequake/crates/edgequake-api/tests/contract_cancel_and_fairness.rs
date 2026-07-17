//! Contract / E2E: task cancel intent + fairness park wiring.
//!
//! Validates SPEC-057 P0 cancel/fairness through the HTTP API:
//! - Pending task cancel → Cancelled status + cancel intent
//! - Cancelled is idempotent
//! - Indexed rejects cancel (409)
//! - Doc KV cancel writes `failure_class=cancelled`
//! - PDF cancel → `PdfProcessingStatus::Cancelled` (postgres feature)
//! - Worker-backed app exposes a shared tenant fairness limiter

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{
    create_test_app_with_workers, extract_json, TEST_TENANT_ID, TEST_USER_ID, TEST_WORKSPACE_ID,
};
use edgequake_storage::kv_keys;
use edgequake_tasks::{
    classify_ingestion_failure, is_cancel_failure_message, IngestionFailureClass, Task, TaskStatus,
    TaskType,
};
use serde_json::json;
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

#[tokio::test]
async fn e2e_cancel_task_writes_doc_kv_failure_class_cancelled() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        json!({ "document_id": "cancel-kv-doc" }),
    );
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let doc_id = "cancel-kv-doc";
    let meta_key = kv_keys::doc_metadata(doc_id);
    let meta = json!({
        "id": doc_id,
        "track_id": track_id,
        "status": "processing",
        "tenant_id": TEST_TENANT_ID,
        "workspace_id": TEST_WORKSPACE_ID,
    });
    edgequake_api::services::upsert_metadata_kv_with_index(
        workers.kv_storage.as_ref(),
        &meta_key,
        meta,
    )
    .await
    .expect("seed metadata");

    let response = post_cancel(workers.app(), &track_id).await;
    assert_eq!(response.status(), StatusCode::OK);

    let stored = workers
        .kv_storage
        .get_by_id(&meta_key)
        .await
        .unwrap()
        .expect("metadata after cancel");
    assert_eq!(stored["status"], "cancelled");
    assert_eq!(stored["failure_class"], "cancelled");
    assert_eq!(stored["recommended_action"], "none");
}

#[test]
fn vision_cancel_message_classifies_as_cancelled() {
    let msg = "Cancelled during vision PDF conversion";
    assert!(is_cancel_failure_message(msg));
    assert_eq!(
        classify_ingestion_failure(msg),
        IngestionFailureClass::Cancelled
    );
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn e2e_pdf_cancel_sets_pdf_status_cancelled_not_failed() {
    use edgequake_storage::{CreatePdfRequest, PdfProcessingStatus};

    let workers = create_test_app_with_workers().await;
    let pdf_storage = workers
        .pdf_storage
        .as_ref()
        .expect("memory pdf storage wired under postgres feature");

    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();
    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n";
    let pdf_id = pdf_storage
        .create_pdf(CreatePdfRequest {
            workspace_id,
            filename: "cancel-contract.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            file_size_bytes: pdf_bytes.len() as i64,
            sha256_checksum: format!("cancel-contract-{}", Uuid::new_v4()),
            page_count: Some(1),
            pdf_data: pdf_bytes.to_vec(),
            vision_model: None,
        })
        .await
        .expect("create pdf");

    pdf_storage
        .update_pdf_status(&pdf_id, PdfProcessingStatus::Processing)
        .await
        .unwrap();

    // Doc id in task.existing_document_id → sync_doc_cancelled_for_task on cancel.
    let doc_id = format!("pdf-cancel-doc-{}", Uuid::new_v4());
    let meta_key = kv_keys::doc_metadata(&doc_id);
    let meta = json!({
        "id": doc_id,
        "status": "processing",
        "tenant_id": TEST_TENANT_ID,
        "workspace_id": TEST_WORKSPACE_ID,
    });
    edgequake_api::services::upsert_metadata_kv_with_index(
        workers.kv_storage.as_ref(),
        &meta_key,
        meta,
    )
    .await
    .expect("seed metadata");

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::PdfProcessing,
        json!({
            "pdf_id": pdf_id.to_string(),
            "existing_document_id": doc_id,
        }),
    );
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = workers
        .app()
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/pdf/{pdf_id}/cancel"))
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
    assert_eq!(body["success"], true);

    let pdf = pdf_storage
        .get_pdf(&pdf_id)
        .await
        .unwrap()
        .expect("pdf row");
    assert_eq!(
        pdf.processing_status,
        PdfProcessingStatus::Cancelled,
        "SPEC-057: PDF cancel must not map to Failed"
    );

    let stored_task = workers
        .task_storage
        .get_task(&track_id)
        .await
        .unwrap()
        .expect("task");
    assert_eq!(stored_task.status, TaskStatus::Cancelled);

    let stored_doc = workers
        .kv_storage
        .get_by_id(&meta_key)
        .await
        .unwrap()
        .expect("doc metadata after PDF cancel");
    assert_eq!(
        stored_doc["status"], "cancelled",
        "SPEC-057 P0: PDF cancel must sync doc KV to cancelled"
    );
    assert_eq!(stored_doc["failure_class"], "cancelled");
}

/// SPEC-057 P2: cancel Convert also cancels Pending Insert for the same pdf_id.
#[tokio::test]
async fn e2e_cancel_convert_cancels_linked_pending_insert() {
    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();
    let pdf_id = Uuid::new_v4();

    let convert = Task::new(
        tenant_id,
        workspace_id,
        TaskType::PdfProcessing,
        json!({ "pdf_id": pdf_id.to_string() }),
    );
    let convert_track = convert.track_id.clone();
    workers.task_storage.create_task(&convert).await.unwrap();

    let insert = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        json!({
            "text": "from convert",
            "file_source": "x.pdf",
            "workspace_id": workspace_id.to_string(),
            "metadata": { "pdf_id": pdf_id.to_string() },
        }),
    );
    let insert_track = insert.track_id.clone();
    workers.task_storage.create_task(&insert).await.unwrap();

    let response = post_cancel(workers.app(), &convert_track).await;
    assert_eq!(response.status(), StatusCode::OK);

    let convert_row = workers
        .task_storage
        .get_task(&convert_track)
        .await
        .unwrap()
        .unwrap();
    let insert_row = workers
        .task_storage
        .get_task(&insert_track)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(convert_row.status, TaskStatus::Cancelled);
    assert_eq!(
        insert_row.status,
        TaskStatus::Cancelled,
        "P2 cancel chain must cancel linked Insert"
    );
}

/// SPEC-057 P1: after cancel, claim_next (restart simulation) never returns the track.
#[tokio::test]
async fn e2e_cancel_pending_never_claimed_after_restart_sim() {
    use std::time::Duration;

    let workers = create_test_app_with_workers().await;
    let tenant_id = Uuid::parse_str(TEST_TENANT_ID).unwrap();
    let workspace_id = Uuid::parse_str(TEST_WORKSPACE_ID).unwrap();

    let task = Task::new(
        tenant_id,
        workspace_id,
        TaskType::Insert,
        json!({ "document_id": "cancel-restart-sim" }),
    );
    let track_id = task.track_id.clone();
    workers.task_storage.create_task(&task).await.unwrap();

    let response = post_cancel(workers.app(), &track_id).await;
    assert_eq!(response.status(), StatusCode::OK);

    let claimed = workers
        .task_storage
        .claim_next("restart-sim-worker", Duration::from_secs(120))
        .await
        .unwrap();
    assert!(
        claimed.as_ref().map(|t| t.track_id.as_str()) != Some(track_id.as_str()),
        "claim_next must not return Cancelled track after restart"
    );
}
