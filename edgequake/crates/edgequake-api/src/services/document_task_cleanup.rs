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

fn should_keep_task(task: &edgequake_tasks::Task, keep_track_id: &str) -> bool {
    if keep_track_id.is_empty() {
        return false;
    }
    if task.track_id == keep_track_id {
        return true;
    }
    // Align payload correlation ids (deletion_track_id / wipe_track_id) with durable track_id.
    if let Some(id) = task
        .task_data
        .get("deletion_track_id")
        .and_then(|v| v.as_str())
    {
        if id == keep_track_id {
            return true;
        }
    }
    if let Some(id) = task.task_data.get("wipe_track_id").and_then(|v| v.as_str()) {
        if id == keep_track_id {
            return true;
        }
    }
    false
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
///
/// Prefer [`purge_persisted_tasks_for_document_except`] when a lifecycle task
/// (Deletion) is running so it does not cancel itself.
pub async fn purge_persisted_tasks_for_document(
    state: &AppState,
    document_id: &str,
    track_id_opt: Option<&str>,
    workspace_id_opt: Option<&str>,
) -> usize {
    purge_persisted_tasks_for_document_except(
        state,
        document_id,
        track_id_opt,
        workspace_id_opt,
        "",
    )
    .await
}

/// Like [`purge_persisted_tasks_for_document`] but never cancels/deletes `keep_track_id`.
///
/// Used by document deletion so the running `TaskType::Deletion` row survives
/// cascade cleanup (DRY with [`purge_workspace_tasks_except`] for wipe).
pub async fn purge_persisted_tasks_for_document_except(
    state: &AppState,
    document_id: &str,
    track_id_opt: Option<&str>,
    workspace_id_opt: Option<&str>,
    keep_track_id: &str,
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
        if should_keep_task(&task, keep_track_id) {
            continue;
        }
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
        if should_keep_task(&task, keep_track_id) {
            continue;
        }
        if cancel_and_delete_task(state, &task).await {
            deleted += 1;
        }
    }

    deleted
}

#[cfg(test)]
mod tests {
    use super::{purge_persisted_tasks_for_document_except, should_keep_task};
    use crate::state::AppState;
    use edgequake_tasks::{Task, TaskStatus, TaskType};
    use uuid::Uuid;

    #[test]
    fn keep_matches_durable_track_id() {
        let task = Task::new(
            Uuid::nil(),
            Uuid::nil(),
            TaskType::Deletion,
            serde_json::json!({"document_id": "doc-1", "deletion_track_id": "other"}),
        );
        assert!(should_keep_task(&task, &task.track_id));
        assert!(!should_keep_task(&task, "deletion-not-this"));
    }

    #[test]
    fn keep_matches_payload_deletion_track_id() {
        let task = Task::new(
            Uuid::nil(),
            Uuid::nil(),
            TaskType::Deletion,
            serde_json::json!({
                "document_id": "doc-1",
                "deletion_track_id": "corr-1"
            }),
        );
        assert!(should_keep_task(&task, "corr-1"));
    }

    #[tokio::test]
    async fn purge_except_keeps_deletion_row_and_allows_indexed_persist() {
        let state = AppState::test_state();
        let tenant = Uuid::nil();
        let workspace = Uuid::nil();
        let doc_id = "purge-keep-self-doc";

        let mut deletion = Task::new(
            tenant,
            workspace,
            TaskType::Deletion,
            serde_json::json!({ "document_id": doc_id }),
        );
        let deletion_id = deletion.track_id.clone();
        if let Some(obj) = deletion.task_data.as_object_mut() {
            obj.insert(
                "deletion_track_id".into(),
                serde_json::json!(&deletion_id),
            );
        }
        deletion.status = TaskStatus::Processing;
        state.tasks.storage.create_task(&deletion).await.unwrap();

        let ingest = Task::new(
            tenant,
            workspace,
            TaskType::PdfProcessing,
            serde_json::json!({ "document_id": doc_id }),
        );
        let ingest_id = ingest.track_id.clone();
        state.tasks.storage.create_task(&ingest).await.unwrap();

        let removed = purge_persisted_tasks_for_document_except(
            &state,
            doc_id,
            None,
            None,
            &deletion_id,
        )
        .await;
        assert!(removed >= 1, "ingest task for doc should be purged");

        let kept = state
            .tasks
            .storage
            .get_task(&deletion_id)
            .await
            .unwrap()
            .expect("running deletion row must survive purge");
        assert_eq!(kept.track_id, deletion_id);

        let gone = state.tasks.storage.get_task(&ingest_id).await.unwrap();
        assert!(gone.is_none(), "ingest task for same doc must be purged");

        let mut to_index = kept;
        to_index.status = TaskStatus::Indexed;
        state
            .tasks
            .storage
            .update_task(&to_index)
            .await
            .expect("Indexed persist must succeed after keep-self purge");
        let indexed = state
            .tasks
            .storage
            .get_task(&deletion_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(indexed.status, TaskStatus::Indexed);
    }
}
