//! Document/workspace task purge — SPEC-027 IMP-029 (SRP extract from storage_helpers).

use uuid::Uuid;

use edgequake_tasks::{Pagination, TaskFilter, TaskStatus};

use crate::state::AppState;

fn parse_explicit_workspace_uuid(workspace_id: Option<&str>) -> Option<Uuid> {
    match workspace_id.map(str::trim) {
        None | Some("") | Some("default") => None,
        Some(value) => Uuid::parse_str(value).ok(),
    }
}

fn task_references_document(task: &edgequake_tasks::Task, document_id: &str) -> bool {
    task.task_data
        .get("existing_document_id")
        .and_then(|v| v.as_str())
        == Some(document_id)
        || task.task_data.get("document_id").and_then(|v| v.as_str()) == Some(document_id)
        || task
            .task_data
            .get("metadata")
            .and_then(|v| v.get("document_id"))
            .and_then(|v| v.as_str())
            == Some(document_id)
}

async fn cancel_and_delete_task(state: &AppState, task: &edgequake_tasks::Task) -> bool {
    if matches!(task.status, TaskStatus::Pending | TaskStatus::Processing) {
        let cancelled = state
            .tasks
            .cancellation_registry
            .cancel(&task.track_id)
            .await;
        tracing::info!(
            track_id = %task.track_id,
            cancelled,
            "Cancelled in-flight task during lifecycle cleanup"
        );
    }

    state
        .tasks
        .pipeline_state
        .remove_pdf_progress(&task.track_id)
        .await;

    if let Ok(Some(mut persisted_task)) = state.tasks.storage.get_task(&task.track_id).await {
        persisted_task.mark_cancelled();
        let _ = state.tasks.storage.update_task(&persisted_task).await;
    }

    state
        .tasks
        .storage
        .delete_task(&task.track_id)
        .await
        .is_ok()
}

/// Remove persisted tasks associated with a single document.
pub async fn purge_persisted_tasks_for_document(
    state: &AppState,
    document_id: &str,
    track_id_opt: Option<&str>,
    workspace_id_opt: Option<&str>,
) -> usize {
    let pagination = Pagination {
        page: 1,
        page_size: 10_000,
        ..Default::default()
    };
    let filter = TaskFilter {
        workspace_id: parse_explicit_workspace_uuid(workspace_id_opt),
        ..Default::default()
    };

    let Ok(task_list) = state.tasks.storage.list_tasks(filter, pagination).await else {
        return 0;
    };

    let mut deleted_count = 0usize;

    for task in task_list.tasks {
        let matches_track = track_id_opt
            .map(|track_id| task.track_id == track_id)
            .unwrap_or(false);
        if !matches_track && !task_references_document(&task, document_id) {
            continue;
        }

        if cancel_and_delete_task(state, &task).await {
            deleted_count += 1;
        }
    }

    deleted_count
}

/// Remove all persisted tasks belonging to a workspace.
pub async fn purge_workspace_tasks(state: &AppState, workspace_id: Uuid) -> usize {
    purge_workspace_tasks_except(state, workspace_id, "").await
}

/// Remove all persisted tasks for a workspace except `keep_track_id`.
///
/// Used by durable workspace wipe so the wipe task does not cancel itself.
pub async fn purge_workspace_tasks_except(
    state: &AppState,
    workspace_id: Uuid,
    keep_track_id: &str,
) -> usize {
    let pagination = Pagination {
        page: 1,
        page_size: 10_000,
        ..Default::default()
    };
    let filter = TaskFilter {
        workspace_id: Some(workspace_id),
        ..Default::default()
    };

    let Ok(task_list) = state.tasks.storage.list_tasks(filter, pagination).await else {
        return 0;
    };

    let mut deleted = 0usize;
    for task in task_list.tasks {
        if !keep_track_id.is_empty() && task.track_id == keep_track_id {
            continue;
        }
        // Also keep by wipe payload correlation id when task.track_id differs.
        if !keep_track_id.is_empty() {
            if let Some(wid) = task.task_data.get("wipe_track_id").and_then(|v| v.as_str()) {
                if wid == keep_track_id {
                    continue;
                }
            }
        }
        if cancel_and_delete_task(state, &task).await {
            deleted += 1;
        }
    }

    deleted
}
