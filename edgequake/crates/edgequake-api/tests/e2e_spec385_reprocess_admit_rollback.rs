//! Issue #385 — early reprocess admit must roll back when retract fails closed.
//!
//! SPEC-119 `retract_document_indexes_checked` used to `?` out of the whole
//! batch after writing `processing`/`cleaning`, leaving documents stuck with
//! no task. These tests inject a source-discovery timeout on the live memory
//! graph and assert: restore prior `failed` status, isolate the rest of the
//! batch, never `processing` without a task.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::services::extract_document_id_from_task;
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_storage::kv_keys;
use edgequake_storage::MemoryGraphStorage;
use edgequake_tasks::storage::{Pagination, TaskFilter};
use edgequake_tasks::TaskStatus;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

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

async fn seed_failed_document(state: &AppState, doc_id: &str, title: &str) {
    let metadata = json!({
        "id": doc_id,
        "title": title,
        "status": "failed",
        "error_message": "prior persist failure",
        "track_id": format!("insert-old-{doc_id}"),
        "current_stage": "extracting",
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

async fn post_reprocess(state: AppState, body: Value) -> (StatusCode, Value) {
    let app = Server::new(test_config(), state).build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents/reprocess")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let parsed = json_body(response).await;
    (status, parsed)
}

async fn doc_metadata(state: &AppState, doc_id: &str) -> Value {
    state
        .storage
        .kv_storage
        .get_by_id(&kv_keys::doc_metadata(doc_id))
        .await
        .unwrap()
        .expect("document metadata must exist")
}

fn is_inflight_status(status: &str) -> bool {
    matches!(
        status,
        "pending" | "queued" | "processing" | "waiting" | "cleaning"
    )
}

async fn live_tasks_for_doc(state: &AppState, doc_id: &str) -> Vec<String> {
    let listed = state
        .tasks
        .storage
        .list_tasks(
            TaskFilter {
                tenant_id: Some(tenant_uuid()),
                workspace_id: Some(workspace_uuid()),
                status: None,
                task_type: None,
            },
            Pagination {
                page: 1,
                page_size: 200,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    listed
        .tasks
        .into_iter()
        .filter(|t| extract_document_id_from_task(t).as_deref() == Some(doc_id))
        .filter(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::Processing))
        .map(|t| t.track_id)
        .collect()
}

async fn assert_no_processing_without_task(state: &AppState, doc_ids: &[&str]) {
    for doc_id in doc_ids {
        let meta = doc_metadata(state, doc_id).await;
        let status = meta.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if is_inflight_status(status) {
            let live = live_tasks_for_doc(state, doc_id).await;
            assert!(
                !live.is_empty(),
                "document {doc_id} is {status} with no live task (issue #385/#384 invariant)"
            );
        }
    }
}

#[tokio::test]
async fn e2e_retract_timeout_rolls_back_and_continues_batch() {
    let graph = Arc::new(MemoryGraphStorage::new("spec385-batch"));
    let state = AppState::test_state_with_graph(Arc::clone(&graph));

    let doc_a = "spec385-batch-doc-a";
    let doc_b = "spec385-batch-doc-b";
    seed_failed_document(&state, doc_a, "spec385 a").await;
    seed_failed_document(&state, doc_b, "spec385 b").await;
    // Arm after AppState construction — QueryEngine wiring can consume a
    // one-shot if it is set before `test_state_with_graph`.
    graph.fail_next_find_edges_by_source_prefixes();

    let (status, body) = post_reprocess(state.clone(), json!({ "max_documents": 10 })).await;

    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "batch reprocess must not 5xx on one retract timeout, got {status}: {body}"
    );
    assert_eq!(
        body["requeued"].as_u64().unwrap_or(0),
        1,
        "exactly one sibling must enqueue: {body}"
    );
    let reasons = body["skip_reasons"].as_object().expect("skip_reasons");
    assert_eq!(
        reasons
            .get("graph_cleanup_failed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        1,
        "the timed-out document must be reported as graph_cleanup_failed: {body}"
    );

    let meta_a = doc_metadata(&state, doc_a).await;
    let meta_b = doc_metadata(&state, doc_b).await;
    let status_a = meta_a.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let status_b = meta_b.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let live_a = live_tasks_for_doc(&state, doc_a).await;
    let live_b = live_tasks_for_doc(&state, doc_b).await;

    let a_failed_no_task = status_a == "failed" && live_a.is_empty();
    let b_failed_no_task = status_b == "failed" && live_b.is_empty();
    assert!(
        a_failed_no_task ^ b_failed_no_task,
        "exactly one document must roll back to failed with no live task; a={status_a}/{live_a:?} b={status_b}/{live_b:?}"
    );

    let (rolled, enqueued_meta, enqueued_live) = if a_failed_no_task {
        (&meta_a, &meta_b, &live_b)
    } else {
        (&meta_b, &meta_a, &live_a)
    };
    let rolled_stage = rolled
        .get("current_stage")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_ne!(
        rolled_stage, "cleaning",
        "rolled-back document must not stay on cleaning: {rolled}"
    );
    let rolled_err = rolled
        .get("error_message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        rolled_err.contains("Graph cleanup timed out"),
        "SPEC-119 product copy must overlay the prior error: {rolled}"
    );
    assert!(
        !enqueued_live.is_empty(),
        "sibling must have a live task after enqueue: {enqueued_meta}"
    );

    assert_no_processing_without_task(&state, &[doc_a, doc_b]).await;
}

#[tokio::test]
async fn e2e_single_doc_retract_timeout_restores_failed_no_5xx() {
    let graph = Arc::new(MemoryGraphStorage::new("spec385-single"));
    let state = AppState::test_state_with_graph(Arc::clone(&graph));

    let doc_id = "spec385-single-doc";
    seed_failed_document(&state, doc_id, "spec385 single").await;
    graph.fail_next_find_edges_by_source_prefixes();

    let (status, body) = post_reprocess(
        state.clone(),
        json!({
            "document_id": doc_id,
            "max_documents": 1
        }),
    )
    .await;

    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "targeted reprocess must not 5xx on retract timeout, got {status}: {body}"
    );
    assert_eq!(body["requeued"].as_u64().unwrap_or(0), 0, "body={body}");
    let reasons = body["skip_reasons"].as_object().expect("skip_reasons");
    assert_eq!(
        reasons
            .get("graph_cleanup_failed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        1,
        "expected graph_cleanup_failed: {body}"
    );

    let meta = doc_metadata(&state, doc_id).await;
    assert_eq!(meta.get("status").and_then(|v| v.as_str()), Some("failed"));
    assert_ne!(
        meta.get("current_stage").and_then(|v| v.as_str()),
        Some("cleaning")
    );
    let err = meta
        .get("error_message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        err.contains("Graph cleanup timed out"),
        "timeout copy must be visible: {meta}"
    );
    assert!(
        live_tasks_for_doc(&state, doc_id).await.is_empty(),
        "no task must be enqueued after fail-closed retract"
    );
    assert_no_processing_without_task(&state, &[doc_id]).await;
}
