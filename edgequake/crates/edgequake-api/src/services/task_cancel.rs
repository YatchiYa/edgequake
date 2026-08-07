//! Shared task-cancel application (DRY / SRP).
//!
//! All cancel entry points (HTTP task cancel, PDF cancel, pipeline cancel,
//! WebSocket) must:
//! 1. Record a cancel intent + signal any in-flight token
//! 2. Persist `TaskStatus::Cancelled` when the task row is cancellable
//! 3. Sync linked document KV via `sync_doc_cancelled_for_task` (SPEC-057 P0)
//!
//! This module owns the task-row + registry half. Document KV sync lives in
//! `task_document_sync` and is called by every cancel entry point.

use edgequake_tasks::{CancellationRegistry, SharedTaskStorage, Task, TaskStatus};

/// Result of applying cancel to a single track_id's task row + registry.
#[derive(Debug, Clone)]
pub struct TaskCancelApplyResult {
    pub track_id: String,
    /// True when an in-flight CancellationToken was signalled.
    pub was_running: bool,
    /// True when we transitioned the task row to Cancelled (or it already was).
    pub cancelled: bool,
    /// True when the task is Indexed and cannot be cancelled.
    pub conflict_indexed: bool,
    /// Updated task snapshot when a row existed.
    pub task: Option<Task>,
}

/// Signal cancel intent/token and persist Cancelled on the task row.
///
/// Idempotent for already-Cancelled tasks. Indexed tasks are left untouched
/// (`conflict_indexed = true`) and do **not** receive a cancel intent.
/// Missing task rows still record cancel intent so a later dequeue/park drops
/// the work (fairness-park / channel race).
pub async fn apply_task_row_cancel(
    storage: &SharedTaskStorage,
    registry: &CancellationRegistry,
    track_id: &str,
) -> Result<TaskCancelApplyResult, String> {
    let existing = storage
        .get_task(track_id)
        .await
        .map_err(|e| format!("Failed to get task: {e}"))?;

    if let Some(task) = existing.as_ref() {
        if task.status == TaskStatus::Indexed {
            return Ok(TaskCancelApplyResult {
                track_id: track_id.to_string(),
                was_running: false,
                cancelled: false,
                conflict_indexed: true,
                task: existing,
            });
        }
    }

    let was_running = registry.cancel(track_id).await;

    let Some(mut task) = existing else {
        // Row already purged (Clear All / delete) — cancel intent recorded; done.
        return Ok(TaskCancelApplyResult {
            track_id: track_id.to_string(),
            was_running,
            cancelled: true,
            conflict_indexed: false,
            task: None,
        });
    };

    if task.status != TaskStatus::Cancelled {
        task.mark_cancelled();
        match storage.update_task(&task).await {
            Ok(()) => {}
            // Concurrent purge already removed the row — cancel intent + registry
            // signal still applied; desired end state is "gone or cancelled".
            Err(edgequake_tasks::TaskError::TaskNotFound(_)) => {
                return Ok(TaskCancelApplyResult {
                    track_id: track_id.to_string(),
                    was_running,
                    cancelled: true,
                    conflict_indexed: false,
                    task: None,
                });
            }
            Err(e) => {
                return Err(format!("Failed to persist cancelled task: {e}"));
            }
        }
    }

    Ok(TaskCancelApplyResult {
        track_id: track_id.to_string(),
        was_running,
        cancelled: true,
        conflict_indexed: false,
        task: Some(task),
    })
}

/// Cancel every currently registered in-flight task (pipeline-wide stop).
pub async fn apply_cancel_all_active(
    storage: &SharedTaskStorage,
    registry: &CancellationRegistry,
) -> Result<Vec<TaskCancelApplyResult>, String> {
    let ids = registry.cancel_all_active().await;
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        results.push(apply_task_row_cancel(storage, registry, &id).await?);
    }
    Ok(results)
}

/// Cancel all active Convert + Insert tasks for a PDF (SPEC-057 P2 cancel chain).
pub async fn apply_cancel_pdf_pipeline_tasks(
    storage: &SharedTaskStorage,
    registry: &CancellationRegistry,
    pdf_id: uuid::Uuid,
    workspace_id: uuid::Uuid,
) -> Result<Vec<TaskCancelApplyResult>, String> {
    let mut results = Vec::new();
    // Bound iterations: Convert + Insert at most (plus races).
    for _ in 0..8 {
        let Some(task) = storage
            .find_active_pdf_processing_task(pdf_id, workspace_id)
            .await
            .map_err(|e| format!("Failed to find PDF pipeline task: {e}"))?
        else {
            break;
        };
        let applied = apply_task_row_cancel(storage, registry, &task.track_id).await?;
        let done = !applied.cancelled || applied.conflict_indexed;
        results.push(applied);
        if done {
            break;
        }
    }
    Ok(results)
}

/// True when a pipeline/processing error string represents user cancel.
///
/// DRY: delegates to `edgequake_tasks::is_cancel_failure_message` (SPEC-057).
pub fn is_cancel_error_message(message: &str) -> bool {
    edgequake_tasks::is_cancel_failure_message(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_tasks::{memory::MemoryTaskStorage, TaskType};
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn apply_cancel_marks_pending_and_records_intent() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let registry = CancellationRegistry::new();
        let task = Task::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();

        let result = apply_task_row_cancel(&storage, &registry, &track_id)
            .await
            .unwrap();
        assert!(result.cancelled);
        assert!(!result.conflict_indexed);
        assert!(registry.has_cancel_intent(&track_id).await);

        let stored = storage.get_task(&track_id).await.unwrap().unwrap();
        assert_eq!(stored.status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn apply_cancel_ok_when_row_already_purged() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let registry = CancellationRegistry::new();
        let track_id = format!("insert-{}", Uuid::new_v4());
        let result = apply_task_row_cancel(&storage, &registry, &track_id)
            .await
            .unwrap();
        assert!(
            result.cancelled,
            "missing row after purge is terminal cancel"
        );
        assert!(result.task.is_none());
        assert!(registry.has_cancel_intent(&track_id).await);
    }

    #[tokio::test]
    async fn apply_cancel_idempotent_after_purge_following_cancel() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let registry = CancellationRegistry::new();
        let task = Task::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();
        let first = apply_task_row_cancel(&storage, &registry, &track_id)
            .await
            .unwrap();
        assert!(first.cancelled);
        storage.delete_task(&track_id).await.unwrap();
        let second = apply_task_row_cancel(&storage, &registry, &track_id)
            .await
            .unwrap();
        assert!(second.cancelled);
        assert!(second.task.is_none());
    }

    #[tokio::test]
    async fn apply_cancel_idempotent_for_already_cancelled() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let registry = CancellationRegistry::new();
        let mut task = Task::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        task.mark_cancelled();
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();

        let result = apply_task_row_cancel(&storage, &registry, &track_id)
            .await
            .unwrap();
        assert!(result.cancelled);
        assert!(!result.conflict_indexed);
    }

    #[tokio::test]
    async fn apply_cancel_conflict_on_indexed() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let registry = CancellationRegistry::new();
        let mut task = Task::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        // SPEC-091 QW0: an Indexed task must walk the legal path (Complete
        // requires Processing — the state machine refuses Pending → Indexed).
        task.mark_processing();
        task.mark_success(serde_json::json!({"ok": true}));
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();

        let result = apply_task_row_cancel(&storage, &registry, &track_id)
            .await
            .unwrap();
        assert!(result.conflict_indexed);
        assert!(!result.cancelled);
        let stored = storage.get_task(&track_id).await.unwrap().unwrap();
        assert_eq!(stored.status, TaskStatus::Indexed);
    }

    #[test]
    fn cancel_error_message_detection() {
        assert!(is_cancel_error_message("Task cancelled during embed"));
        assert!(is_cancel_error_message("Cancelled by user"));
        assert!(is_cancel_error_message(
            "Cancelled during vision PDF conversion"
        ));
        assert!(!is_cancel_error_message("Network error"));
    }
}
