//! Issue #384 / #385 — in-flight document status implies a live Task.
//!
//! Shared so bulk reprocess, HTTP reprocess, and SPEC-054 reconcile e2e
//! assert the same invariant (DRY).

#![allow(dead_code)]

use edgequake_api::document_metadata::is_active_processing_status;
use edgequake_api::services::extract_document_id_from_task;
use edgequake_api::AppState;
use edgequake_storage::kv_keys;
use edgequake_tasks::storage::{Pagination, TaskFilter};
use uuid::Uuid;

/// Waiting / cleaning are UX stages; pipeline in-flight uses
/// [`is_active_processing_status`].
pub fn is_inflight_document_status(status: &str) -> bool {
    matches!(
        status.to_lowercase().as_str(),
        "queued" | "waiting" | "cleaning"
    ) || is_active_processing_status(status)
}

pub async fn live_task_ids_for_doc(
    state: &AppState,
    tenant_id: Uuid,
    workspace_id: Uuid,
    doc_id: &str,
) -> Vec<String> {
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
        .filter(|t| t.status.is_inflight())
        .map(|t| t.track_id)
        .collect()
}

/// Fail if any listed document advertises in-flight work with no live task.
pub async fn assert_no_inflight_without_live_task(
    state: &AppState,
    tenant_id: Uuid,
    workspace_id: Uuid,
    doc_ids: &[&str],
) {
    for doc_id in doc_ids {
        let meta = state
            .storage
            .kv_storage
            .get_by_id(&kv_keys::doc_metadata(doc_id))
            .await
            .unwrap()
            .unwrap_or_else(|| panic!("document metadata must exist for {doc_id}"));
        let status = meta.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if is_inflight_document_status(status) {
            let live = live_task_ids_for_doc(state, tenant_id, workspace_id, doc_id).await;
            assert!(
                !live.is_empty(),
                "document {doc_id} is {status} with no live task (issue #384/#385 invariant)"
            );
        }
    }
}
