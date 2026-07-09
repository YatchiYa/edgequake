//! Task ↔ document KV sync helpers (SPEC-045 SRE-I01).
//!
//! Keeps document metadata terminal state aligned when tasks fail outside the
//! worker processor (e.g. periodic orphan heartbeat detection in `main.rs`).

use std::sync::Arc;

use edgequake_storage::traits::KVStorage;
use edgequake_tasks::Task;
use serde_json::json;

use crate::document_metadata::is_terminal_failure_status;

/// Extract document ID from task payload (PDF or text insert paths).
pub fn extract_document_id_from_task(task: &Task) -> Option<String> {
    task.task_data
        .get("existing_document_id")
        .and_then(|v| v.as_str())
        .or_else(|| {
            task.task_data
                .get("metadata")
                .and_then(|m| m.get("document_id"))
                .and_then(|v| v.as_str())
        })
        .map(str::to_string)
}

/// Mark document metadata `failed` when a task dies from heartbeat loss.
pub async fn sync_document_failed_on_orphan_heartbeat(
    kv: Arc<dyn KVStorage>,
    task: &Task,
    error_msg: &str,
) -> Result<(), String> {
    let Some(document_id) = extract_document_id_from_task(task) else {
        return Ok(());
    };

    let metadata_key = crate::services::resolve_document_metadata_key(&document_id, &kv).await;

    let existing = kv
        .get_by_id(&metadata_key)
        .await
        .map_err(|e| e.to_string())?;

    let Some(mut obj) = existing.and_then(|v| v.as_object().cloned()) else {
        return Ok(());
    };

    let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if is_terminal_failure_status(status) {
        return Ok(());
    }

    let failure = crate::services::classify_ingestion_failure(error_msg);
    edgequake_observability::record_ingestion_failure(
        failure.as_str(),
        &task.workspace_id.to_string(),
    );

    obj.insert("status".to_string(), json!("failed"));
    obj.insert("current_stage".to_string(), json!("failed"));
    obj.insert("error_message".to_string(), json!(error_msg));
    obj.insert("failure_class".to_string(), json!(failure.as_str()));
    obj.insert(
        "recommended_action".to_string(),
        json!(failure.recommended_action()),
    );
    obj.insert(
        "stage_message".to_string(),
        json!(format!(
            "Task heartbeat lost — processing stopped. {}",
            error_msg
        )),
    );
    obj.insert(
        "updated_at".to_string(),
        json!(chrono::Utc::now().to_rfc3339()),
    );

    crate::services::upsert_metadata_kv_with_index(kv.as_ref(), &metadata_key, json!(obj))
        .await
        .map_err(|e| e.to_string())?;

    tracing::warn!(
        task_id = %task.track_id,
        document_id = %document_id,
        failure_class = failure.as_str(),
        "Periodic orphan check: synced document metadata to failed"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_tasks::{Task, TaskType};

    #[test]
    fn spec045_extract_document_id_from_pdf_task() {
        let task = Task {
            track_id: "t1".to_string(),
            tenant_id: uuid::Uuid::new_v4(),
            workspace_id: uuid::Uuid::new_v4(),
            task_type: TaskType::PdfProcessing,
            status: edgequake_tasks::TaskStatus::Processing,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
            consecutive_timeout_failures: 0,
            circuit_breaker_tripped: false,
            task_data: serde_json::json!({ "existing_document_id": "doc-abc" }),
            metadata: None,
            progress: None,
            result: None,
        };
        assert_eq!(
            extract_document_id_from_task(&task).as_deref(),
            Some("doc-abc")
        );
    }

    #[test]
    fn spec045_extract_document_id_from_insert_metadata() {
        let task = Task::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({
                "metadata": { "document_id": "doc-xyz" }
            }),
        );
        assert_eq!(
            extract_document_id_from_task(&task).as_deref(),
            Some("doc-xyz")
        );
    }
}
