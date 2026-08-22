//! Issue #386 — reprocess/purge must not destroy finished task history.
//!
//! A later reprocess admit used to `mark_cancelled()` + `delete_task()` every
//! document-referenced row, including already-`Failed` attempts. The execution
//! record vanished from `tasks` (and from the `tasks_history` partition, which
//! is not an archive API). These tests pin: leave the Failed row, enqueue a
//! new attempt.
//!
//! A Processing sibling is required so "Failed survived" cannot pass merely
//! because purge never listed the row.
//!
//! Memory-mode (`AppState::test_state()`), CI-safe.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::services::document_task_cleanup::purge_persisted_tasks_for_document;
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_storage::kv_keys;
use edgequake_tasks::{Task, TaskStatus, TaskType};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

const COLLISION_ERROR: &str =
    r#"duplicate key value violates unique constraint "idx_entity_embeddings_legacy_vector_id""#;

const TEST_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";
const TEST_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000002";

fn test_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: false,
    }
}

fn tenant_uuid() -> Uuid {
    Uuid::parse_str(TEST_TENANT_ID).unwrap()
}

fn workspace_uuid() -> Uuid {
    Uuid::parse_str(TEST_WORKSPACE_ID).unwrap()
}

fn ingest_task(doc_id: &str) -> Task {
    Task::new(
        tenant_uuid(),
        workspace_uuid(),
        TaskType::Insert,
        json!({ "document_id": doc_id }),
    )
}

fn failed_ingest_task(doc_id: &str) -> Task {
    let mut task = ingest_task(doc_id);
    task.mark_failed(COLLISION_ERROR.to_string());
    task
}

async fn seed_failed_document(state: &AppState, doc_id: &str, track_id: &str) {
    let metadata = json!({
        "id": doc_id,
        "title": "spec386 collision doc",
        "status": "failed",
        "error_message": COLLISION_ERROR,
        "track_id": track_id,
        "tenant_id": TEST_TENANT_ID,
        "workspace_id": TEST_WORKSPACE_ID,
    });
    edgequake_api::services::upsert_metadata_kv_with_index(
        state.storage.kv_storage.as_ref(),
        &kv_keys::doc_metadata(doc_id),
        metadata,
    )
    .await
    .expect("seed metadata + wsdoc index");

    state
        .storage
        .kv_storage
        .upsert(&[(
            format!("{doc_id}-content"),
            json!({ "content": "Body that survived the failed persist." }),
        )])
        .await
        .unwrap();
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(json!({}))
}

/// Direct contract the reprocess handler calls: purge leaves Failed + error.
#[tokio::test]
async fn e2e_purge_then_new_enqueue_keeps_failed_history() {
    let state = AppState::test_state();
    let doc_id = "spec386-purge-enqueue-doc";

    let failed = failed_ingest_task(doc_id);
    let failed_id = failed.track_id.clone();
    state.tasks.storage.create_task(&failed).await.unwrap();

    let mut inflight = ingest_task(doc_id);
    inflight.mark_processing();
    let inflight_id = inflight.track_id.clone();
    state.tasks.storage.create_task(&inflight).await.unwrap();

    seed_failed_document(&state, doc_id, &failed_id).await;

    let purged =
        purge_persisted_tasks_for_document(&state, doc_id, None, Some(TEST_WORKSPACE_ID)).await;
    assert_eq!(purged, 1, "only the in-flight sibling must be cancelled");
    assert!(
        state
            .tasks
            .storage
            .get_task(&inflight_id)
            .await
            .unwrap()
            .is_none(),
        "purge must have matched this document and removed Processing"
    );

    let kept = state
        .tasks
        .storage
        .get_task(&failed_id)
        .await
        .unwrap()
        .expect("Failed track_id must remain queryable after purge");
    assert_eq!(kept.status, TaskStatus::Failed);
    assert_eq!(kept.error_message.as_deref(), Some(COLLISION_ERROR));

    let replacement = ingest_task(doc_id);
    let replacement_id = replacement.track_id.clone();
    state.tasks.storage.create_task(&replacement).await.unwrap();

    assert_ne!(failed_id, replacement_id);
    assert!(state
        .tasks
        .storage
        .get_task(&failed_id)
        .await
        .unwrap()
        .is_some());
    let new_row = state
        .tasks
        .storage
        .get_task(&replacement_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_row.status, TaskStatus::Pending);
}

/// HTTP reprocess admit must not erase the prior Failed attempt.
#[tokio::test]
async fn e2e_http_reprocess_keeps_failed_task_row() {
    let state = AppState::test_state();
    let doc_id = "spec386-http-reprocess-doc";

    let failed = failed_ingest_task(doc_id);
    let failed_id = failed.track_id.clone();
    state.tasks.storage.create_task(&failed).await.unwrap();

    let mut inflight = ingest_task(doc_id);
    inflight.mark_processing();
    let inflight_id = inflight.track_id.clone();
    state.tasks.storage.create_task(&inflight).await.unwrap();

    seed_failed_document(&state, doc_id, &failed_id).await;

    let app = Server::new(test_config(), state.clone()).build_router();
    let request = json!({
        "document_id": doc_id,
        "force": true,
        "max_documents": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents/reprocess")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = json_body(response).await;
    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "reprocess should succeed, got {status}: {body}"
    );
    let requeued = body["requeued"].as_u64().unwrap_or(0);
    let new_id = body["document_task_ids"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v["task_id"].as_str())
        .map(str::to_string);
    assert!(
        requeued >= 1 || new_id.is_some(),
        "document must be requeued so purge actually ran: {body}"
    );
    assert!(
        state
            .tasks
            .storage
            .get_task(&inflight_id)
            .await
            .unwrap()
            .is_none(),
        "reprocess purge must have run and cancelled the Processing sibling"
    );

    let kept = state
        .tasks
        .storage
        .get_task(&failed_id)
        .await
        .unwrap()
        .expect("HTTP reprocess must not delete the Failed execution record (#386)");
    assert_eq!(kept.status, TaskStatus::Failed);
    assert_eq!(kept.error_message.as_deref(), Some(COLLISION_ERROR));

    let new_id = new_id.expect("reprocess must return the replacement task_id");
    assert_ne!(
        new_id, failed_id,
        "reprocess must enqueue a new attempt, not reuse the Failed track_id"
    );
    let replacement = state
        .tasks
        .storage
        .get_task(&new_id)
        .await
        .unwrap()
        .expect("replacement task must exist alongside the Failed audit row");
    assert!(
        replacement.status.is_inflight(),
        "new attempt must be Pending/Processing, got {:?}",
        replacement.status
    );
}
