//! Document/workspace task purge — SPEC-027 IMP-029 (SRP extract from storage_helpers).
//!
//! Cancel is a control-plane verb for **in-flight** work (`Pending` /
//! `Processing`) only. Finished attempts (`Failed`, `Indexed`, `Cancelled`)
//! stay in `tasks` with their original status, error, and timestamps so a
//! later reprocess can enqueue a new `track_id` without rewriting history
//! (issue #386 / BR0903). Physical removal of old terminals is
//! [`edgequake_tasks::TaskStorage::prune_terminal_tasks`], not admit-cleanup.
//!
//! `tasks_history` is a RANGE partition of `tasks` (migration 104), not an
//! archive destination. `DELETE FROM tasks` would also erase that partition.

use uuid::Uuid;

use edgequake_tasks::{Pagination, TaskFilter};

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

/// Cancel and delete an **in-flight** task ([`edgequake_tasks::TaskStatus::is_inflight`]).
///
/// Finished rows are left unchanged and this returns `false` so callers'
/// `deleted_count` only reflects actual cancel+delete. `Failed → Cancelled`
/// is a legal state-machine edge for operator cancel; lifecycle purge must
/// not take it, because that overwrites the real failure before delete.
async fn cancel_and_delete_task(state: &AppState, task: &edgequake_tasks::Task) -> bool {
    if !task.status.is_inflight() {
        tracing::debug!(
            track_id = %task.track_id,
            status = ?task.status,
            "Skipping lifecycle purge of finished task (audit row retained)"
        );
        return false;
    }

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

/// Cancel in-flight persisted tasks associated with a single document.
///
/// Finished attempts for the document are retained in `tasks` (issue #386).
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

/// Like [`purge_persisted_tasks_for_document`] but never cancels `keep_track_id`.
///
/// Used by document deletion so the running `TaskType::Deletion` row survives
/// cascade cleanup (DRY with [`purge_workspace_tasks_except`] for wipe).
/// Only in-flight siblings are cancelled; Failed/Indexed/Cancelled stay.
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

/// Cancel in-flight persisted tasks belonging to a workspace.
///
/// Finished attempts stay in `tasks` for audit. Workspace **delete** still
/// drops leftover rows via FK `ON DELETE CASCADE` when the workspace row
/// itself is removed.
pub async fn purge_workspace_tasks(state: &AppState, workspace_id: Uuid) -> usize {
    purge_workspace_tasks_except(state, workspace_id, "").await
}

/// Cancel in-flight workspace tasks except `keep_track_id`.
///
/// Used by durable workspace wipe so the wipe task does not cancel itself.
/// Wipe cancels ingestion that is still running; it must not erase Indexed
/// or Failed history while the workspace lives (issue #386).
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
            obj.insert("deletion_track_id".into(), serde_json::json!(&deletion_id));
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

        let removed =
            purge_persisted_tasks_for_document_except(&state, doc_id, None, None, &deletion_id)
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

    /// #386 reproduction: a Failed attempt with a real persist error must
    /// survive reprocess-admit purge. Today that row is the only copy of the
    /// execution record (`tasks_history` is a partition, not an archive).
    const COLLISION_ERROR: &str = r#"duplicate key value violates unique constraint "idx_entity_embeddings_legacy_vector_id""#;

    fn new_doc_task(tenant: Uuid, workspace: Uuid, doc_id: &str) -> Task {
        Task::new(
            tenant,
            workspace,
            TaskType::PdfProcessing,
            serde_json::json!({ "document_id": doc_id }),
        )
    }

    async fn persist(state: &AppState, task: &Task) {
        state.tasks.storage.create_task(task).await.unwrap();
    }

    async fn get(state: &AppState, track_id: &str) -> Option<Task> {
        state.tasks.storage.get_task(track_id).await.unwrap()
    }

    #[tokio::test]
    async fn purge_preserves_failed_task_status_and_error() {
        let state = AppState::test_state();
        let doc_id = "purge-failed-audit-doc";
        let mut failed = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        failed.mark_failed(COLLISION_ERROR.to_string());
        let failed_id = failed.track_id.clone();
        persist(&state, &failed).await;

        let removed = super::purge_persisted_tasks_for_document(&state, doc_id, None, None).await;
        assert_eq!(removed, 0, "finished Failed row is not in-flight");

        let kept = get(&state, &failed_id)
            .await
            .expect("Failed task must survive purge (#386)");
        assert_eq!(kept.status, TaskStatus::Failed);
        assert_eq!(kept.error_message.as_deref(), Some(COLLISION_ERROR));
        assert_eq!(kept.track_id, failed_id);
    }

    #[tokio::test]
    async fn purge_preserves_indexed_task() {
        let state = AppState::test_state();
        let doc_id = "purge-indexed-audit-doc";
        let mut indexed = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        indexed.mark_processing();
        assert!(indexed.mark_success(serde_json::json!({"ok": true})));
        let indexed_id = indexed.track_id.clone();
        persist(&state, &indexed).await;

        let removed = super::purge_persisted_tasks_for_document(&state, doc_id, None, None).await;
        assert_eq!(removed, 0);

        let kept = get(&state, &indexed_id)
            .await
            .expect("Indexed task must survive purge (#386)");
        assert_eq!(kept.status, TaskStatus::Indexed);
    }

    #[tokio::test]
    async fn purge_preserves_cancelled_task() {
        let state = AppState::test_state();
        let doc_id = "purge-cancelled-audit-doc";
        let mut cancelled = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        cancelled.mark_cancelled();
        let cancelled_id = cancelled.track_id.clone();
        persist(&state, &cancelled).await;

        let removed = super::purge_persisted_tasks_for_document(&state, doc_id, None, None).await;
        assert_eq!(removed, 0);

        let kept = get(&state, &cancelled_id)
            .await
            .expect("Cancelled task must survive purge (#386)");
        assert_eq!(kept.status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn purge_still_cancels_pending_and_processing() {
        let state = AppState::test_state();
        let doc_id = "purge-inflight-doc";

        let pending = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        let pending_id = pending.track_id.clone();
        persist(&state, &pending).await;

        let mut processing = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        processing.mark_processing();
        let processing_id = processing.track_id.clone();
        persist(&state, &processing).await;

        let removed = super::purge_persisted_tasks_for_document(&state, doc_id, None, None).await;
        assert_eq!(removed, 2, "both in-flight rows must be cancelled+deleted");
        assert!(get(&state, &pending_id).await.is_none());
        assert!(get(&state, &processing_id).await.is_none());
    }

    #[tokio::test]
    async fn purge_mixed_doc_keeps_failed_removes_processing() {
        let state = AppState::test_state();
        let doc_id = "purge-mixed-doc";

        let mut failed = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        failed.mark_failed(COLLISION_ERROR.to_string());
        let failed_id = failed.track_id.clone();
        persist(&state, &failed).await;

        let mut processing = new_doc_task(Uuid::nil(), Uuid::nil(), doc_id);
        processing.mark_processing();
        let processing_id = processing.track_id.clone();
        persist(&state, &processing).await;

        let removed = super::purge_persisted_tasks_for_document(&state, doc_id, None, None).await;
        assert_eq!(removed, 1, "only the in-flight sibling is purged");
        assert!(get(&state, &processing_id).await.is_none());

        let kept = get(&state, &failed_id)
            .await
            .expect("Failed sibling must survive");
        assert_eq!(kept.status, TaskStatus::Failed);
        assert_eq!(kept.error_message.as_deref(), Some(COLLISION_ERROR));
    }

    #[tokio::test]
    async fn workspace_purge_preserves_terminal_cancels_inflight() {
        let state = AppState::test_state();
        let tenant = Uuid::nil();
        let workspace = Uuid::new_v4();
        let keep_wipe = "wipe-keep-self";

        let mut failed = new_doc_task(tenant, workspace, "ws-failed-doc");
        failed.mark_failed(COLLISION_ERROR.to_string());
        let failed_id = failed.track_id.clone();
        persist(&state, &failed).await;

        let mut indexed = new_doc_task(tenant, workspace, "ws-indexed-doc");
        indexed.mark_processing();
        assert!(indexed.mark_success(serde_json::json!({"ok": true})));
        let indexed_id = indexed.track_id.clone();
        persist(&state, &indexed).await;

        let pending = new_doc_task(tenant, workspace, "ws-pending-doc");
        let pending_id = pending.track_id.clone();
        persist(&state, &pending).await;

        let mut wipe = Task::new(
            tenant,
            workspace,
            TaskType::WorkspaceWipe,
            serde_json::json!({ "wipe_track_id": keep_wipe }),
        );
        wipe.status = TaskStatus::Processing;
        persist(&state, &wipe).await;

        let removed = super::purge_workspace_tasks_except(&state, workspace, keep_wipe).await;
        assert_eq!(removed, 1, "only the pending ingest is in-flight");
        assert!(get(&state, &pending_id).await.is_none());

        let kept_failed = get(&state, &failed_id)
            .await
            .expect("workspace wipe must not erase Failed history");
        assert_eq!(kept_failed.status, TaskStatus::Failed);
        assert_eq!(kept_failed.error_message.as_deref(), Some(COLLISION_ERROR));

        let kept_indexed = get(&state, &indexed_id)
            .await
            .expect("workspace wipe must not erase Indexed history");
        assert_eq!(kept_indexed.status, TaskStatus::Indexed);
        assert!(
            get(&state, &wipe.track_id).await.is_some(),
            "wipe keep-self row must survive"
        );
    }
}
