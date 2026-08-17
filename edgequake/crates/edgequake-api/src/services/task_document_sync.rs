//! Task ↔ document KV sync helpers (SPEC-045 SRE-I01).
//!
//! Keeps document metadata terminal state aligned when tasks fail outside the
//! worker processor (e.g. periodic orphan heartbeat detection in `main.rs`).
<<<<<<< HEAD
=======
//!
//! After KV upsert, also touches `public.documents.status` when a sidecar pool
//! is registered so list reads (column SSOT) cannot resurrect mid-pipeline
//! zombies after cancel/fail.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

use std::sync::Arc;

use edgequake_storage::traits::KVStorage;
use edgequake_tasks::Task;
use serde_json::json;
<<<<<<< HEAD
=======
use uuid::Uuid;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

use crate::document_metadata::is_terminal_failure_status;

/// Extract document ID from task payload (PDF or text insert paths).
pub fn extract_document_id_from_task(task: &Task) -> Option<String> {
    task.task_data
        .get("existing_document_id")
        .and_then(|v| v.as_str())
        .or_else(|| task.task_data.get("document_id").and_then(|v| v.as_str()))
        .or_else(|| {
            task.task_data
                .get("metadata")
                .and_then(|m| m.get("document_id"))
                .and_then(|v| v.as_str())
        })
<<<<<<< HEAD
        .map(str::to_string)
=======
        .or_else(|| {
            task.metadata
                .as_ref()
                .and_then(|m| m.get("document_id"))
                .and_then(|v| v.as_str())
        })
        .or_else(|| {
            task.metadata
                .as_ref()
                .and_then(|m| m.get("existing_document_id"))
                .and_then(|v| v.as_str())
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Resolve document id: task payload → `documents.track_id` / metadata track_id.
pub async fn resolve_document_id_for_task(kv: &dyn KVStorage, task: &Task) -> Option<String> {
    if let Some(id) = extract_document_id_from_task(task) {
        return Some(id);
    }
    if let Some(id) = document_id_by_track_id(&task.track_id).await {
        return Some(id);
    }
    // Last resort: workspace-scoped metadata scan for matching track_id / pdf_id.
    find_document_id_in_kv_by_correlation(kv, task).await
}

#[cfg(feature = "postgres")]
async fn document_id_by_track_id(track_id: &str) -> Option<String> {
    let pool = crate::services::relational_sidecar_store::sidecar_pool()?;
    match sqlx::query_scalar::<_, String>(
        "SELECT id::text FROM public.documents \
         WHERE track_id = $1 OR metadata->>'track_id' = $1 \
         LIMIT 1",
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::debug!(
                track_id = %track_id,
                error = %e,
                "document_id_by_track_id lookup failed"
            );
            None
        }
    }
}

#[cfg(not(feature = "postgres"))]
async fn document_id_by_track_id(_track_id: &str) -> Option<String> {
    None
}

async fn find_document_id_in_kv_by_correlation(kv: &dyn KVStorage, task: &Task) -> Option<String> {
    let pdf_id = task.pdf_id().map(|u| u.to_string());
    let entries = crate::services::document_metadata_scan::load_all_document_metadata_entries(kv)
        .await
        .ok()?;
    for (_key, value) in entries {
        let Some(obj) = value.as_object() else {
            continue;
        };
        let track_match = obj
            .get("track_id")
            .and_then(|v| v.as_str())
            .is_some_and(|t| t == task.track_id);
        let pdf_match = pdf_id.as_ref().is_some_and(|p| {
            obj.get("pdf_id")
                .and_then(|v| v.as_str())
                .is_some_and(|v| v == p)
        });
        if !(track_match || pdf_match) {
            continue;
        }
        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
            let id = id.trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Best-effort `public.documents.status` touch (list column SSOT).
pub async fn touch_relational_document_status_best_effort(document_id: &str, status: &str) {
    #[cfg(feature = "postgres")]
    {
        let Some(pool) = crate::services::relational_sidecar_store::sidecar_pool() else {
            return;
        };
        let Ok(doc_uuid) = Uuid::parse_str(document_id) else {
            return;
        };
        let pg_status = if status == "completed" {
            "indexed"
        } else {
            status
        };
        if let Err(e) =
            sqlx::query("UPDATE public.documents SET status = $2, updated_at = NOW() WHERE id = $1")
                .bind(doc_uuid)
                .bind(pg_status)
                .execute(pool)
                .await
        {
            tracing::warn!(
                document_id = %document_id,
                status = %status,
                error = %e,
                "touch_relational_document_status_best_effort failed (non-fatal)"
            );
        }
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (document_id, status);
    }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

/// After task cancel: sync linked document KV to `cancelled` + failure_class (SPEC-057 P0).
///
/// No-op when the task has no document id, metadata is missing, or the doc is
/// already terminal-cancelled. Used by HTTP/WS/PDF/pipeline cancel paths.
pub async fn sync_doc_cancelled_for_task(
    kv: Arc<dyn KVStorage>,
    task: &Task,
    message: &str,
) -> Result<bool, String> {
<<<<<<< HEAD
    let Some(document_id) = extract_document_id_from_task(task) else {
=======
    let Some(document_id) = resolve_document_id_for_task(kv.as_ref(), task).await else {
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        return Ok(false);
    };
    sync_doc_cancelled_by_document_id(kv, &document_id, message).await
}

/// Sync a document metadata row to cancelled by document id.
pub async fn sync_doc_cancelled_by_document_id(
    kv: Arc<dyn KVStorage>,
    document_id: &str,
    message: &str,
) -> Result<bool, String> {
<<<<<<< HEAD
    let metadata_key = crate::services::resolve_document_metadata_key(document_id, &kv).await;
    let existing = kv
        .get_by_id(&metadata_key)
        .await
        .map_err(|e| e.to_string())?;

    let Some(mut obj) = existing.and_then(|v| v.as_object().cloned()) else {
=======
    // IMP-075-10: one RT staging+final (not resolve key then re-get).
    let Some((metadata_key, existing)) =
        crate::services::load_staging_first_metadata(kv.as_ref(), document_id).await?
    else {
        return Ok(false);
    };

    let Some(mut obj) = existing.as_object().cloned() else {
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        return Ok(false);
    };

    let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if status.eq_ignore_ascii_case("cancelled") {
<<<<<<< HEAD
=======
        // Still touch relational in case KV was cancelled but column lagged.
        touch_relational_document_status_best_effort(document_id, "cancelled").await;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        return Ok(false);
    }

    crate::services::apply_doc_cancelled_fields(&mut obj, message);
    crate::services::upsert_metadata_kv_with_index(kv.as_ref(), &metadata_key, json!(obj))
        .await
        .map_err(|e| e.to_string())?;
<<<<<<< HEAD
=======
    touch_relational_document_status_best_effort(document_id, "cancelled").await;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

    tracing::info!(
        document_id = %document_id,
        "Synced document metadata to cancelled after task cancel"
    );
    Ok(true)
}

<<<<<<< HEAD
=======
/// Mark a mid-pipeline orphan document failed (no live Pending/Processing task).
pub async fn sync_doc_failed_no_active_task(
    kv: Arc<dyn KVStorage>,
    document_id: &str,
    message: &str,
) -> Result<bool, String> {
    let Some((metadata_key, existing)) =
        crate::services::load_staging_first_metadata(kv.as_ref(), document_id).await?
    else {
        return Ok(false);
    };

    let Some(mut obj) = existing.as_object().cloned() else {
        return Ok(false);
    };

    let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if is_terminal_failure_status(status)
        || crate::document_metadata::is_terminal_success_status(status)
    {
        return Ok(false);
    }

    crate::services::apply_doc_failed_fields(&mut obj, message);
    crate::services::upsert_metadata_kv_with_index(kv.as_ref(), &metadata_key, json!(obj))
        .await
        .map_err(|e| e.to_string())?;
    touch_relational_document_status_best_effort(document_id, "failed").await;

    tracing::warn!(
        document_id = %document_id,
        "Marked document failed — pipeline interrupted with no active task"
    );
    Ok(true)
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
/// Mark document metadata `failed` when a task dies from heartbeat loss.
pub async fn sync_document_failed_on_orphan_heartbeat(
    kv: Arc<dyn KVStorage>,
    task: &Task,
    error_msg: &str,
) -> Result<(), String> {
<<<<<<< HEAD
    let Some(document_id) = extract_document_id_from_task(task) else {
        return Ok(());
    };

    let metadata_key = crate::services::resolve_document_metadata_key(&document_id, &kv).await;

    let existing = kv
        .get_by_id(&metadata_key)
        .await
        .map_err(|e| e.to_string())?;

    let Some(mut obj) = existing.and_then(|v| v.as_object().cloned()) else {
=======
    let Some(document_id) = resolve_document_id_for_task(kv.as_ref(), task).await else {
        return Ok(());
    };

    // IMP-075-10: one RT staging+final (not resolve key then re-get).
    let Some((metadata_key, existing)) =
        crate::services::load_staging_first_metadata(kv.as_ref(), &document_id).await?
    else {
        return Ok(());
    };

    let Some(mut obj) = existing.as_object().cloned() else {
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        return Ok(());
    };

    let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if is_terminal_failure_status(status) {
        return Ok(());
    }

<<<<<<< HEAD
    let failure = crate::services::classify_ingestion_failure(error_msg);
    edgequake_observability::record_ingestion_failure(
        failure.as_str(),
        &task.workspace_id.to_string(),
    );

    obj.insert("status".to_string(), json!("failed"));
    obj.insert("current_stage".to_string(), json!("failed"));
    obj.insert("error_message".to_string(), json!(error_msg));
    obj.insert("failure_class".to_string(), json!(failure.as_str()));
=======
    let heartbeat_msg = format!("Task heartbeat lost — processing stopped. {error_msg}");
    crate::services::apply_doc_failed_fields(&mut obj, &heartbeat_msg);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    // ISSUE-304: structured Interrupted code for Reprocess routing (not message matching).
    if error_msg
        .to_ascii_lowercase()
        .contains("interrupted by server restart")
        || error_msg
            .to_ascii_lowercase()
            .contains("interrupted — use reprocess")
    {
        obj.insert(
            "failure_code".to_string(),
            json!(crate::services::FAILURE_CODE_SERVER_RESTART_INTERRUPTED),
        );
    }
<<<<<<< HEAD
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
=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

    crate::services::upsert_metadata_kv_with_index(kv.as_ref(), &metadata_key, json!(obj))
        .await
        .map_err(|e| e.to_string())?;
<<<<<<< HEAD
=======
    touch_relational_document_status_best_effort(&document_id, "failed").await;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

    tracing::warn!(
        task_id = %task.track_id,
        document_id = %document_id,
<<<<<<< HEAD
        failure_class = failure.as_str(),
=======
        failure_class = obj.get("failure_class").and_then(|v| v.as_str()).unwrap_or(""),
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
            lease_owner: None,
            lease_token: None,
            lease_expires_at: None,
<<<<<<< HEAD
=======
            fairness_hold_until: None,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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

<<<<<<< HEAD
=======
    #[test]
    fn extract_document_id_from_task_metadata_field() {
        let mut task = Task::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            TaskType::PdfProcessing,
            serde_json::json!({ "pdf_id": uuid::Uuid::new_v4().to_string() }),
        );
        task.metadata = Some(json!({ "document_id": "doc-from-meta" }));
        assert_eq!(
            extract_document_id_from_task(&task).as_deref(),
            Some("doc-from-meta")
        );
    }

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    #[tokio::test]
    async fn sync_doc_cancelled_for_task_sets_failure_class() {
        use edgequake_storage::kv_keys;
        use edgequake_storage::MemoryKVStorage;

        let kv: Arc<dyn KVStorage> = Arc::new(MemoryKVStorage::new("cancel-sync-test"));
        let doc_id = "cancel-sync-doc";
        let meta_key = kv_keys::doc_metadata(doc_id);
        crate::services::upsert_metadata_kv_with_index(
            kv.as_ref(),
            &meta_key,
            json!({
                "id": doc_id,
                "status": "processing",
                "workspace_id": "ws-1",
            }),
        )
        .await
        .unwrap();

        let task = Task::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            TaskType::Insert,
            json!({ "metadata": { "document_id": doc_id } }),
        );

        let updated = sync_doc_cancelled_for_task(Arc::clone(&kv), &task, "Task cancelled by user")
            .await
            .unwrap();
        assert!(updated);

        let stored = kv.get_by_id(&meta_key).await.unwrap().unwrap();
        assert_eq!(stored["status"], "cancelled");
        assert_eq!(stored["failure_class"], "cancelled");
        assert_eq!(stored["recommended_action"], "none");
    }
<<<<<<< HEAD
=======

    #[tokio::test]
    async fn sync_doc_failed_no_active_task_marks_failed() {
        use edgequake_storage::kv_keys;
        use edgequake_storage::MemoryKVStorage;

        let kv: Arc<dyn KVStorage> = Arc::new(MemoryKVStorage::new("orphan-fail-test"));
        let doc_id = "orphan-fail-doc";
        let meta_key = kv_keys::doc_metadata(doc_id);
        crate::services::upsert_metadata_kv_with_index(
            kv.as_ref(),
            &meta_key,
            json!({
                "id": doc_id,
                "status": "converting",
                "workspace_id": "ws-1",
            }),
        )
        .await
        .unwrap();

        let updated = sync_doc_failed_no_active_task(
            Arc::clone(&kv),
            doc_id,
            "Pipeline interrupted — no active task",
        )
        .await
        .unwrap();
        assert!(updated);
        let stored = kv.get_by_id(&meta_key).await.unwrap().unwrap();
        assert_eq!(stored["status"], "failed");
    }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}
