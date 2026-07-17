//! Unified cancel entry for HTTP / WebSocket (SPEC-057 DRY).
//!
//! Owns: task-row cancel + doc KV sync + Convert∪Insert pdf_id chain.

use std::sync::Arc;

use edgequake_storage::traits::KVStorage;
use edgequake_tasks::{CancellationRegistry, SharedTaskStorage};

use super::task_cancel::{
    apply_cancel_pdf_pipeline_tasks, apply_task_row_cancel, TaskCancelApplyResult,
};
use super::task_document_sync::sync_doc_cancelled_for_task;

/// Cancel a track_id, sync linked document KV, and cancel sibling Convert/Insert
/// tasks for the same `pdf_id` when present.
pub async fn cancel_track_with_doc_and_pdf_chain(
    storage: &SharedTaskStorage,
    registry: &CancellationRegistry,
    kv: Arc<dyn KVStorage>,
    track_id: &str,
) -> Result<TaskCancelApplyResult, String> {
    let applied = apply_task_row_cancel(storage, registry, track_id).await?;

    if applied.conflict_indexed {
        return Ok(applied);
    }

    if applied.cancelled {
        if let Some(ref task) = applied.task {
            if let Err(e) =
                sync_doc_cancelled_for_task(Arc::clone(&kv), task, "Task cancelled by user").await
            {
                tracing::warn!(
                    track_id = %track_id,
                    error = %e,
                    "cancel facade: doc KV sync failed"
                );
            }

            if let Some(pdf_id) = task.pdf_id() {
                match apply_cancel_pdf_pipeline_tasks(storage, registry, pdf_id, task.workspace_id)
                    .await
                {
                    Ok(linked) => {
                        for linked_applied in linked {
                            if let Some(ref linked_task) = linked_applied.task {
                                let _ = sync_doc_cancelled_for_task(
                                    Arc::clone(&kv),
                                    linked_task,
                                    "Task cancelled by user",
                                )
                                .await;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            track_id = %track_id,
                            pdf_id = %pdf_id,
                            error = %e,
                            "cancel facade: linked PDF pipeline cancel failed"
                        );
                    }
                }
            }
        }
    }

    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::MemoryKVStorage;
    use edgequake_tasks::{memory::MemoryTaskStorage, Task, TaskStatus, TaskType};
    use uuid::Uuid;

    #[tokio::test]
    async fn cancel_convert_also_cancels_linked_insert() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let registry = CancellationRegistry::new();
        let kv: Arc<dyn KVStorage> = Arc::new(MemoryKVStorage::new("cancel-facade"));
        let tenant = Uuid::new_v4();
        let workspace = Uuid::new_v4();
        let pdf_id = Uuid::new_v4();

        let convert = Task::new(
            tenant,
            workspace,
            TaskType::PdfProcessing,
            serde_json::json!({ "pdf_id": pdf_id.to_string() }),
        );
        let convert_track = convert.track_id.clone();
        storage.create_task(&convert).await.unwrap();

        let insert = Task::new(
            tenant,
            workspace,
            TaskType::Insert,
            serde_json::json!({
                "text": "x",
                "file_source": "a.pdf",
                "workspace_id": workspace.to_string(),
                "metadata": { "pdf_id": pdf_id.to_string() },
            }),
        );
        let insert_track = insert.track_id.clone();
        storage.create_task(&insert).await.unwrap();

        let applied = cancel_track_with_doc_and_pdf_chain(&storage, &registry, kv, &convert_track)
            .await
            .unwrap();
        assert!(applied.cancelled);

        assert_eq!(
            storage
                .get_task(&convert_track)
                .await
                .unwrap()
                .unwrap()
                .status,
            TaskStatus::Cancelled
        );
        assert_eq!(
            storage
                .get_task(&insert_track)
                .await
                .unwrap()
                .unwrap()
                .status,
            TaskStatus::Cancelled
        );
    }
}
