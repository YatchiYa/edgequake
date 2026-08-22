//! Issue #384 — bulk reprocess must not write in-flight status without a live task.
//!
//! First principles: Task is work; document status is a projection. Workspace
//! bulk ops used to `mark_document_pending` before enqueue, so `no_content` and
//! `create_task` failure left documents pending forever.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::services::extract_document_id_from_task;
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_core::types::CreateWorkspaceRequest;
use edgequake_core::Tenant;
use edgequake_storage::kv_keys;
use edgequake_tasks::memory::MemoryTaskStorage;
use edgequake_tasks::storage::{Pagination, TaskFilter};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

#[path = "common/inflight_task_invariant.rs"]
mod inflight_task_invariant;

fn test_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: false,
    }
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(json!({}))
}

async fn seed_completed_doc(state: &AppState, doc_id: &str, tenant_id: Uuid, workspace_id: Uuid) {
    let metadata = json!({
        "id": doc_id,
        "title": "spec384 completed",
        "status": "completed",
        "tenant_id": tenant_id.to_string(),
        "workspace_id": workspace_id.to_string(),
        "source_type": "text",
        "chunk_count": 1,
    });
    edgequake_api::services::upsert_metadata_kv_with_index(
        state.storage.kv_storage.as_ref(),
        &kv_keys::doc_metadata(doc_id),
        metadata,
    )
    .await
    .expect("seed metadata");
}

async fn seed_content(state: &AppState, doc_id: &str) {
    state
        .storage
        .kv_storage
        .upsert(&[(
            format!("{doc_id}-content"),
            json!({ "content": "Body for reprocess." }),
        )])
        .await
        .unwrap();
}

async fn create_tenant_workspace(state: &AppState) -> (Uuid, Uuid) {
    let tenant = Tenant::new("spec384", format!("spec384-{}", Uuid::new_v4()));
    let created = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .expect("tenant");
    let ws = state
        .workspace_service
        .create_workspace(
            created.tenant_id,
            CreateWorkspaceRequest {
                name: "spec384 ws".to_string(),
                slug: Some(format!("spec384-{}", Uuid::new_v4())),
                ..Default::default()
            },
        )
        .await
        .expect("workspace");
    (created.tenant_id, ws.workspace_id)
}

async fn post_reprocess_all(
    state: AppState,
    tenant_id: Uuid,
    workspace_id: Uuid,
    body: Value,
) -> (StatusCode, Value) {
    let app = Server::new(test_config(), state).build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/reprocess-documents"
                ))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", tenant_id.to_string())
                .header("X-Workspace-ID", workspace_id.to_string())
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let parsed = json_body(response).await;
    (status, parsed)
}

async fn doc_status(state: &AppState, doc_id: &str) -> String {
    state
        .storage
        .kv_storage
        .get_by_id(&kv_keys::doc_metadata(doc_id))
        .await
        .unwrap()
        .expect("metadata")
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

async fn doc_track_id(state: &AppState, doc_id: &str) -> String {
    state
        .storage
        .kv_storage
        .get_by_id(&kv_keys::doc_metadata(doc_id))
        .await
        .unwrap()
        .expect("metadata")
        .get("track_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

#[tokio::test]
async fn e2e_bulk_reprocess_with_content_binds_task_track_id() {
    let state = AppState::test_state();
    let (tenant_id, workspace_id) = create_tenant_workspace(&state).await;
    let doc_id = "spec384-with-content";
    seed_completed_doc(&state, doc_id, tenant_id, workspace_id).await;
    seed_content(&state, doc_id).await;

    let (status, body) = post_reprocess_all(
        state.clone(),
        tenant_id,
        workspace_id,
        json!({ "include_completed": true, "max_documents": 10 }),
    )
    .await;

    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "got {status}: {body}"
    );
    assert_eq!(
        body["documents_queued"].as_u64().unwrap_or(0),
        1,
        "queued: {body}"
    );

    let listed = state
        .tasks
        .storage
        .list_tasks(
            TaskFilter {
                tenant_id: Some(tenant_id),
                workspace_id: Some(workspace_id),
                ..Default::default()
            },
            Pagination {
                page: 1,
                page_size: 50,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let live: Vec<_> = listed
        .tasks
        .iter()
        .filter(|t| extract_document_id_from_task(t).as_deref() == Some(doc_id))
        .filter(|t| t.status.is_inflight())
        .collect();
    assert_eq!(live.len(), 1, "exactly one live task: {listed:?}");
    let task_id = &live[0].track_id;
    assert_eq!(
        doc_track_id(&state, doc_id).await,
        *task_id,
        "KV track_id must be the worker task id, not the batch reprocess_* id"
    );
    assert!(
        !task_id.starts_with("reprocess_"),
        "task id must not be the batch id: {task_id}"
    );
    inflight_task_invariant::assert_no_inflight_without_live_task(
        &state,
        tenant_id,
        workspace_id,
        &[doc_id],
    )
    .await;
}

#[tokio::test]
async fn e2e_bulk_reprocess_no_content_leaves_completed() {
    let state = AppState::test_state();
    let (tenant_id, workspace_id) = create_tenant_workspace(&state).await;
    let doc_id = "spec384-no-content";
    seed_completed_doc(&state, doc_id, tenant_id, workspace_id).await;

    let (status, body) = post_reprocess_all(
        state.clone(),
        tenant_id,
        workspace_id,
        json!({ "include_completed": true, "max_documents": 10 }),
    )
    .await;

    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "got {status}: {body}"
    );
    assert_eq!(body["documents_queued"].as_u64().unwrap_or(0), 0, "{body}");
    assert_eq!(
        body["skip_reasons"]["no_content"].as_u64().unwrap_or(0),
        1,
        "tonight's retry repro: no_content must be reported, not silent pending: {body}"
    );
    assert_eq!(doc_status(&state, doc_id).await, "completed");
    inflight_task_invariant::assert_no_inflight_without_live_task(
        &state,
        tenant_id,
        workspace_id,
        &[doc_id],
    )
    .await;
}

#[tokio::test]
async fn e2e_bulk_reprocess_enqueue_failure_leaves_completed() {
    let tasks = Arc::new(MemoryTaskStorage::new());
    let state = AppState::test_state_with_memory_tasks(Arc::clone(&tasks));
    let (tenant_id, workspace_id) = create_tenant_workspace(&state).await;
    let doc_id = "spec384-enqueue-fail";
    seed_completed_doc(&state, doc_id, tenant_id, workspace_id).await;
    seed_content(&state, doc_id).await;
    tasks.fail_next_create_task();

    let (status, body) = post_reprocess_all(
        state.clone(),
        tenant_id,
        workspace_id,
        json!({ "include_completed": true, "max_documents": 10 }),
    )
    .await;

    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "got {status}: {body}"
    );
    assert_eq!(body["documents_queued"].as_u64().unwrap_or(0), 0, "{body}");
    assert_eq!(
        body["skip_reasons"]["task_enqueue_failed"]
            .as_u64()
            .unwrap_or(0),
        1,
        "enqueue failure must be counted: {body}"
    );
    assert_eq!(doc_status(&state, doc_id).await, "completed");
    inflight_task_invariant::assert_no_inflight_without_live_task(
        &state,
        tenant_id,
        workspace_id,
        &[doc_id],
    )
    .await;
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn e2e_pdf_leftover_enqueue_failure_does_not_abort_kv_batch() {
    use edgequake_storage::{CreatePdfRequest, PdfProcessingStatus};

    let tasks = Arc::new(MemoryTaskStorage::new());
    let state = AppState::test_state_with_memory_tasks(Arc::clone(&tasks));
    let (tenant_id, workspace_id) = create_tenant_workspace(&state).await;

    let kv_doc = "spec384-kv-sibling";
    let metadata = json!({
        "id": kv_doc,
        "title": "kv sibling",
        "status": "failed",
        "error_message": "prior",
        "tenant_id": tenant_id.to_string(),
        "workspace_id": workspace_id.to_string(),
        "source_type": "text",
    });
    edgequake_api::services::upsert_metadata_kv_with_index(
        state.storage.kv_storage.as_ref(),
        &kv_keys::doc_metadata(kv_doc),
        metadata,
    )
    .await
    .unwrap();
    seed_content(&state, kv_doc).await;

    let pdf_storage = state.storage.pdf_storage.as_ref().expect("pdf_storage");
    let pdf_id = pdf_storage
        .create_pdf(CreatePdfRequest {
            workspace_id,
            filename: "stuck.pdf".into(),
            content_type: "application/pdf".into(),
            file_size_bytes: 16,
            sha256_checksum: format!("spec384-{}", Uuid::new_v4()),
            page_count: Some(1),
            pdf_data: b"%PDF-1.4\n%%EOF\n".to_vec(),
            vision_model: None,
        })
        .await
        .expect("create pdf");
    pdf_storage
        .update_pdf_status(&pdf_id, PdfProcessingStatus::Failed)
        .await
        .expect("mark failed");

    tasks.fail_next_pdf_processing_create();

    let app = Server::new(test_config(), state.clone()).build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents/reprocess")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", tenant_id.to_string())
                .header("X-Workspace-ID", workspace_id.to_string())
                .body(Body::from(
                    serde_json::to_string(&json!({ "max_documents": 10 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;
    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "PDF enqueue fail must not 5xx the batch, got {status}: {body}"
    );

    inflight_task_invariant::assert_no_inflight_without_live_task(
        &state,
        tenant_id,
        workspace_id,
        &[kv_doc],
    )
    .await;

    let pdf = pdf_storage.get_pdf(&pdf_id).await.unwrap().expect("pdf");
    assert_eq!(
        pdf.processing_status,
        PdfProcessingStatus::Failed,
        "PDF must remain Failed when enqueue fails"
    );
}
