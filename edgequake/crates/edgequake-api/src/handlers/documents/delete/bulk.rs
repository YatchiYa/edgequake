//! Bulk deletion handler for all documents.
//!
//! Admits durable `TaskType::WorkspaceWipe` (HTTP 202) with `wipe_track_id`.
//! The worker cancels inflight work, clears graph/vectors once, then purges docs.
//!
//! First principles (issue #309):
//! - Wipe-all must not run N× per-document AGE source-prefix cascades.
//! - HTTP 202 means admitted, never completed.
//!
//! @implements SPEC-050: Real-time bulk deletion progress via WebSocket broadcast.

use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    Json,
};
use edgequake_tasks::{Task, TaskType};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::handlers::documents_types::*;
use crate::middleware::{resolve_workspace_uuid, TenantContext};
use crate::services::{
    count_planned_wipe_documents, find_active_workspace_wipe_track_id, new_wipe_task_data,
};
use crate::state::AppState;

/// Delete all documents in the workspace (bulk wipe).
///
/// Returns **202 Accepted** with `wipe_track_id` after durable enqueue.
/// Terminal counts arrive via WebSocket `BulkDeletionCompleted` / `BulkDeletionFailed`
/// or `GET /api/v1/tasks/{wipe_track_id}`.
#[utoipa::path(
    delete,
    path = "/api/v1/documents",
    tag = "Documents",
    responses(
        (status = 202, description = "Bulk wipe accepted; track via wipe_track_id / WebSocket", body = DeleteAllDocumentsResponse),
        (status = 400, description = "Missing confirm header when required"),
        (status = 409, description = "Workspace wipe already in flight"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn delete_all_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    tenant_ctx: TenantContext,
) -> ApiResult<(StatusCode, HeaderMap, Json<DeleteAllDocumentsResponse>)> {
    if state.security.require_delete_all_confirm {
        let confirmed = headers
            .get("x-edgequake-confirm")
            .and_then(|value| value.to_str().ok())
            == Some("delete-all-documents");
        if !confirmed {
            tracing::warn!(
                workspace_id = ?tenant_ctx.workspace_id,
                "Bulk delete rejected — missing X-EdgeQuake-Confirm header (SPEC-027 IMP-018)"
            );
            return Err(ApiError::BadRequest(
                "Bulk delete requires header X-EdgeQuake-Confirm: delete-all-documents".into(),
            ));
        }
    } else if headers.get("x-edgequake-confirm").is_none() {
        tracing::warn!(
            workspace_id = ?tenant_ctx.workspace_id,
            "Bulk delete without confirm header — set EDGEQUAKE_REQUIRE_DELETE_ALL_CONFIRM=true to enforce"
        );
    }

    tracing::info!(workspace_id = ?tenant_ctx.workspace_id, "Bulk delete documents requested");

    let workspace_id_str = tenant_ctx.workspace_id_or_default();
    let workspace_uuid = resolve_workspace_uuid(Some(&workspace_id_str))
        .ok_or_else(|| ApiError::BadRequest(format!("invalid workspace_id: {workspace_id_str}")))?;

    if let Some(existing) = find_active_workspace_wipe_track_id(&state, workspace_uuid).await {
        tracing::info!(
            workspace_id = %workspace_uuid,
            wipe_track_id = %existing,
            "Workspace wipe already in flight — returning existing track"
        );
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = HeaderValue::from_str(&format!("/api/v1/tasks/{existing}")) {
            resp_headers.insert(header::LOCATION, loc);
        }
        return Ok((
            StatusCode::ACCEPTED,
            resp_headers,
            Json(admit_wipe_response(existing, 0)),
        ));
    }

    let planned_delete = count_planned_wipe_documents(&state, &tenant_ctx).await?;

    let tenant_id_str = tenant_ctx
        .tenant_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let tenant_uuid = Uuid::parse_str(&tenant_id_str).unwrap_or_else(|_| Uuid::nil());

    // Pre-allocate track id for process-local admission before durable enqueue.
    let provisional_track = format!("workspace_wipe-{}", Uuid::new_v4());
    if let Some(existing) = state
        .tasks
        .wipe_admission
        .try_register(workspace_uuid, &provisional_track)
    {
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = HeaderValue::from_str(&format!("/api/v1/tasks/{existing}")) {
            resp_headers.insert(header::LOCATION, loc);
        }
        return Ok((
            StatusCode::ACCEPTED,
            resp_headers,
            Json(admit_wipe_response(existing, 0)),
        ));
    }

    let mut task = Task::new(
        tenant_uuid,
        workspace_uuid,
        TaskType::WorkspaceWipe,
        serde_json::json!({}),
    );
    // Align wipe_track_id with durable task.track_id for Location / WS / poll.
    let wipe_track_id = task.track_id.clone();
    // Atomically replace provisional slot with the real track id — never release
    // first (that opens a race where upload/reprocess can admit mid-wipe).
    if let Some(existing) = state.tasks.wipe_admission.replace_track_id(
        workspace_uuid,
        &provisional_track,
        &wipe_track_id,
    ) {
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = HeaderValue::from_str(&format!("/api/v1/tasks/{existing}")) {
            resp_headers.insert(header::LOCATION, loc);
        }
        return Ok((
            StatusCode::ACCEPTED,
            resp_headers,
            Json(admit_wipe_response(existing, 0)),
        ));
    }

    let data = new_wipe_task_data(
        tenant_id_str,
        workspace_id_str,
        wipe_track_id.clone(),
        planned_delete,
    );
    task.task_data = serde_json::to_value(&data).map_err(|e| {
        state.tasks.wipe_admission.release(workspace_uuid);
        ApiError::Internal(format!("Failed to serialize WorkspaceWipeTaskData: {e}"))
    })?;

    if let Err(e) = state.enqueue_task(task).await {
        state.tasks.wipe_admission.release(workspace_uuid);
        return Err(e);
    }

    tracing::info!(
        workspace_id = %workspace_uuid,
        wipe_track_id = %wipe_track_id,
        planned_delete,
        "Admitted durable WorkspaceWipe task"
    );

    let mut resp_headers = HeaderMap::new();
    if let Ok(loc) = HeaderValue::from_str(&format!("/api/v1/tasks/{wipe_track_id}")) {
        resp_headers.insert(header::LOCATION, loc);
    }

    Ok((
        StatusCode::ACCEPTED,
        resp_headers,
        Json(admit_wipe_response(wipe_track_id, planned_delete)),
    ))
}

fn admit_wipe_response(wipe_track_id: String, planned_delete: usize) -> DeleteAllDocumentsResponse {
    DeleteAllDocumentsResponse {
        accepted: true,
        wipe_track_id: Some(wipe_track_id),
        deleted_count: planned_delete,
        planned_delete_count: Some(planned_delete),
        total_chunks_deleted: 0,
        total_entities_removed: 0,
        total_relationships_removed: 0,
        total_pdfs_deleted: 0,
        skipped_count: 0,
        skipped_documents: Vec::new(),
    }
}
