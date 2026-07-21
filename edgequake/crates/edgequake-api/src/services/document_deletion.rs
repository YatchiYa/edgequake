//! Authoritative document cascade delete (DRY for handler worker + bulk).
//!
//! First principles:
//! - HTTP admits `status=deleting` and returns 202; this service runs async.
//! - Batch graph/vector ops (see `document_graph_cascade`) — no N+1 Cypher.
//! - Idempotent: deleting absent rows is a no-op.
//! - Fail closed with `delete_failed` status (never leave permanent `deleting`).

use uuid::Uuid;

use edgequake_audit::{AuditEventType, AuditResult};
use edgequake_core::MetricsTriggerType;
use edgequake_tasks::DeletionTaskData;

use crate::error::{ApiError, ApiResult};
use crate::handlers::websocket_types::DeletionPhaseKind;
use crate::middleware::TenantContext;
use crate::services::document_metadata_scan::metadata_key_for_document;
use crate::services::document_task_cleanup::purge_persisted_tasks_for_document;
use crate::services::document_vector_storage::get_workspace_vector_storage_for_delete;
use crate::services::{
    cascade_remove_document_sources, record_compliance_event, CascadeStats, ContentHasher,
    DocumentSourceScope,
};
use crate::state::AppState;

/// Result of a completed cascade delete.
#[derive(Debug, Clone, Default)]
pub struct DocumentDeletionResult {
    pub chunks_deleted: usize,
    pub entities_removed: usize,
    pub entities_updated: usize,
    pub relationships_removed: usize,
    pub relationships_updated: usize,
    pub embeddings_deleted: usize,
    pub persisted_tasks_removed: usize,
    pub partial_failure: bool,
    pub partial_failure_reason: Option<String>,
}

/// Reset a stuck/failed deleting document to a recoverable terminal status.
pub async fn reset_deleting_status(
    state: &AppState,
    document_id: &str,
    key_prefix: &str,
    reason: &str,
    deletion_track_id: Option<&str>,
) {
    let metadata_key = metadata_key_for_document(key_prefix);
    if let Ok(Some(mut metadata)) = state.storage.kv_storage.get_by_id(&metadata_key).await {
        if let Some(obj) = metadata.as_object_mut() {
            let current = obj
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if current == "deleting" || current == "delete_failed" {
                obj.insert("status".to_string(), serde_json::json!("delete_failed"));
                obj.insert(
                    "current_stage".to_string(),
                    serde_json::json!("delete_failed"),
                );
                obj.insert("stage_message".to_string(), serde_json::json!(reason));
                obj.insert("error_message".to_string(), serde_json::json!(reason));
                let _ = crate::services::upsert_metadata_kv_with_index(
                    state.storage.kv_storage.as_ref(),
                    &metadata_key,
                    metadata,
                )
                .await;
            }
        }
    }

    if let Some(track_id) = deletion_track_id {
        state
            .tasks
            .progress_broadcaster
            .deletion_failed(document_id, track_id, reason);
    }
}

/// Run the authoritative cascade for a document (vectors → graph → KV → relational).
pub async fn perform_document_deletion(
    state: &AppState,
    data: &DeletionTaskData,
    tenant_ctx: &TenantContext,
) -> ApiResult<DocumentDeletionResult> {
    let document_id = data.document_id.clone();
    let actual_key_prefix = data.key_prefix.clone();
    let key_id_mismatch = actual_key_prefix != document_id;
    let metadata_key = data
        .metadata_key
        .clone()
        .unwrap_or_else(|| metadata_key_for_document(&actual_key_prefix));
    let has_metadata = state
        .storage
        .kv_storage
        .get_by_id(&metadata_key)
        .await
        .ok()
        .flatten()
        .is_some();
    let content_key = format!("{}-content", actual_key_prefix);
    let has_content = data.has_content
        || state
            .storage
            .kv_storage
            .get_by_id(&content_key)
            .await
            .ok()
            .flatten()
            .is_some();
    let chunk_ids = if data.chunk_ids.is_empty() {
        let chunk_prefix = format!("{}-chunk-", actual_key_prefix);
        state
            .storage
            .kv_storage
            .keys_with_prefix(&chunk_prefix)
            .await
            .unwrap_or_default()
    } else {
        data.chunk_ids.clone()
    };
    let workspace_id_for_storage = data.workspace_id.clone();
    let deletion_track_id = data.deletion_track_id.clone();
    let document_status = data
        .document_status
        .clone()
        .unwrap_or_else(|| "deleting".to_string());

    state
        .tasks
        .progress_broadcaster
        .deletion_started(&document_id, &deletion_track_id);

    if matches!(
        document_status.as_str(),
        "pending" | "processing" | "deleting"
    ) {
        if let Some(track_id) = data.ingest_track_id.as_deref() {
            state.tasks.progress_broadcaster.deletion_phase(
                &document_id,
                &deletion_track_id,
                DeletionPhaseKind::CancellingTask,
                0,
                1,
            );
            let cancelled = state.tasks.cancellation_registry.cancel(track_id).await;
            tracing::info!(
                document_id = %document_id,
                track_id = %track_id,
                status = %document_status,
                cancelled,
                "Cancelled in-flight task before cascade delete"
            );
        }
    }

    let persisted_tasks_removed = purge_persisted_tasks_for_document(
        state,
        &document_id,
        data.ingest_track_id.as_deref(),
        Some(&workspace_id_for_storage),
    )
    .await;

    let workspace_vector_storage =
        get_workspace_vector_storage_for_delete(state, &workspace_id_for_storage).await;

    let chunks_deleted = chunk_ids.len();
    let mut embeddings_deleted = 0usize;
    let mut partial_failure = false;
    let mut partial_failure_reason: Option<String> = None;

    state.tasks.progress_broadcaster.deletion_phase(
        &document_id,
        &deletion_track_id,
        DeletionPhaseKind::RemovingVectors,
        0,
        chunk_ids.len() as u32,
    );

    // Prefer document-scoped wipe (chunks + any doc-tagged vectors), then
    // fall back to explicit chunk id list for legacy rows.
    match workspace_vector_storage
        .delete_by_document(&document_id)
        .await
    {
        Ok(n) => {
            embeddings_deleted += n;
        }
        Err(e) => {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "delete_by_document failed; falling back to chunk id delete"
            );
            if !chunk_ids.is_empty() {
                if let Err(e2) = workspace_vector_storage.delete(&chunk_ids).await {
                    tracing::warn!(
                        document_id = %document_id,
                        error = %e2,
                        "Failed to delete chunk embeddings, continuing with graph cleanup"
                    );
                } else {
                    embeddings_deleted += chunk_ids.len();
                }
            }
        }
    }
    if key_id_mismatch {
        let _ = workspace_vector_storage
            .delete_by_document(&actual_key_prefix)
            .await;
    }

    let scope =
        DocumentSourceScope::with_key_prefix(document_id.clone(), actual_key_prefix.clone());

    state.tasks.progress_broadcaster.deletion_phase(
        &document_id,
        &deletion_track_id,
        DeletionPhaseKind::RemovingGraph,
        0,
        0,
    );

    let cascade_stats = match cascade_remove_document_sources(
        &state.storage.graph_storage,
        Some(&workspace_vector_storage),
        Some(tenant_ctx),
        &scope,
    )
    .await
    {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!(
                document_id = %document_id,
                error = %e,
                "Graph cascade delete failed (non-fatal) — proceeding with KV/vector/relational cleanup"
            );
            partial_failure = true;
            partial_failure_reason = Some(format!("Graph cascade error: {}", e));
            CascadeStats::default()
        }
    };
    let entities_removed = cascade_stats.entities_removed;
    let entities_updated = cascade_stats.entities_updated;
    let relationships_removed = cascade_stats.relationships_removed;
    let relationships_updated = cascade_stats.relationships_updated;
    embeddings_deleted += cascade_stats.embeddings_deleted;

    let mut keys_to_delete = chunk_ids.clone();
    if has_metadata {
        keys_to_delete.push(metadata_key.clone());
        keys_to_delete.push(edgequake_storage::kv_keys::workspace_doc_index(
            &workspace_id_for_storage,
            &actual_key_prefix,
        ));
    }
    if has_content {
        keys_to_delete.push(content_key);
    }

    let actual_doc_prefix = format!("{}-", actual_key_prefix);
    let all_prefix_keys: Vec<String> = state
        .storage
        .kv_storage
        .keys_with_prefix(&actual_doc_prefix)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|k| !keys_to_delete.contains(k))
        .collect();
    keys_to_delete.extend(all_prefix_keys);

    if key_id_mismatch {
        let json_doc_prefix = format!("{}-", document_id);
        let alt_prefix_keys: Vec<String> = state
            .storage
            .kv_storage
            .keys_with_prefix(&json_doc_prefix)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|k| !keys_to_delete.contains(k))
            .collect();
        keys_to_delete.extend(alt_prefix_keys);
    }

    if let Some(content_hash) = data.content_hash.as_ref() {
        let hash_key = ContentHasher::workspace_hash_key(&workspace_id_for_storage, content_hash);
        keys_to_delete.push(hash_key);
    }

    state.tasks.progress_broadcaster.deletion_phase(
        &document_id,
        &deletion_track_id,
        DeletionPhaseKind::RemovingKv,
        0,
        keys_to_delete.len() as u32,
    );

    state
        .storage
        .kv_storage
        .delete(&keys_to_delete)
        .await
        .map_err(ApiError::from)?;

    #[cfg(feature = "postgres")]
    {
        let mm_storage = state.storage.mm_asset_storage.as_deref();
        let workspace_uuid = uuid::Uuid::parse_str(&workspace_id_for_storage).ok();
        match crate::services::delete_document_mm_assets(mm_storage, &document_id, workspace_uuid)
            .await
        {
            Ok(n) if n > 0 => {
                tracing::debug!(
                    document_id = %document_id,
                    deleted = n,
                    "Deleted document mm-assets"
                );
            }
            Err(e) => {
                tracing::warn!(
                    document_id = %document_id,
                    error = %e,
                    "Failed to delete document mm-assets (continuing cascade)"
                );
            }
            _ => {}
        }
        if key_id_mismatch {
            let _ = crate::services::delete_document_mm_assets(
                mm_storage,
                &actual_key_prefix,
                workspace_uuid,
            )
            .await;
        }
    }

    #[cfg(feature = "postgres")]
    {
        if let Some(ref pdf_storage) = state.storage.pdf_storage {
            if let Some(ref pid) = data.pdf_id {
                if let Ok(pdf_uuid) = Uuid::parse_str(pid) {
                    if let Err(e) = pdf_storage.delete_pdf(&pdf_uuid).await {
                        tracing::warn!(
                            pdf_id = %pid,
                            document_id = %document_id,
                            error = %e,
                            "Failed to delete pdf_documents row (may already be gone)"
                        );
                    }
                }
            }

            let doc_ids_to_try: Vec<&str> = if key_id_mismatch {
                vec![&actual_key_prefix, &document_id]
            } else {
                vec![&document_id]
            };
            for doc_id_str in &doc_ids_to_try {
                if let Ok(doc_uuid) = Uuid::parse_str(doc_id_str) {
                    match pdf_storage.delete_document_record(&doc_uuid).await {
                        Ok(_) => break,
                        Err(e) => {
                            tracing::warn!(
                                document_id = %doc_id_str,
                                error = %e,
                                "Failed to delete documents table row (may not exist)"
                            );
                        }
                    }
                }
            }
        }
    }

    tracing::info!(
        document_id = %document_id,
        chunks = chunks_deleted,
        embeddings_deleted = embeddings_deleted,
        entities_removed = entities_removed,
        entities_updated = entities_updated,
        relationships_removed = relationships_removed,
        relationships_updated = relationships_updated,
        persisted_tasks_removed = persisted_tasks_removed,
        "Document cascade delete complete"
    );

    if let Ok(workspace_uuid) = Uuid::parse_str(&workspace_id_for_storage) {
        if let Err(e) = state
            .workspace_service
            .record_metrics_snapshot(workspace_uuid, MetricsTriggerType::Event)
            .await
        {
            tracing::warn!(
                workspace_id = %workspace_id_for_storage,
                error = %e,
                "Failed to record post-deletion metrics snapshot"
            );
        }
    }

    let tenant_for_audit = tenant_ctx
        .tenant_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    record_compliance_event(
        state,
        tenant_for_audit,
        AuditEventType::Authorization,
        "delete_document",
        AuditResult::Success,
        tenant_ctx.workspace_id.clone(),
        tenant_ctx.user_id.clone(),
        Some(("document".to_string(), document_id.clone())),
    );

    state.tasks.progress_broadcaster.deletion_phase(
        &document_id,
        &deletion_track_id,
        DeletionPhaseKind::Finalizing,
        0,
        1,
    );

    state.tasks.progress_broadcaster.deletion_completed(
        &document_id,
        &deletion_track_id,
        chunks_deleted,
        entities_removed,
        relationships_removed,
        embeddings_deleted,
        partial_failure,
        partial_failure_reason.clone(),
    );

    Ok(DocumentDeletionResult {
        chunks_deleted,
        entities_removed,
        entities_updated,
        relationships_removed,
        relationships_updated,
        embeddings_deleted,
        persisted_tasks_removed,
        partial_failure,
        partial_failure_reason,
    })
}

/// Boot/crash recovery: docs left in `deleting` with no active Deletion task
/// are re-enqueued (idempotent cascade) so they never stay stuck forever.
pub async fn reconcile_stuck_deleting_documents(state: &AppState, max: usize) -> usize {
    use crate::services::document_metadata_scan::{
        document_id_from_metadata_key, load_all_document_metadata_entries,
    };
    use edgequake_tasks::{Task, TaskType};

    let Ok(entries) = load_all_document_metadata_entries(state.storage.kv_storage.as_ref()).await
    else {
        return 0;
    };

    let mut requeued = 0usize;
    for (key, meta) in entries {
        if requeued >= max {
            break;
        }
        let status = meta
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if status != "deleting" {
            continue;
        }
        let document_id = meta
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| document_id_from_metadata_key(&key))
            .unwrap_or_else(|| key.trim_end_matches("-metadata").to_string());
        let key_prefix = document_id_from_metadata_key(&key).unwrap_or_else(|| document_id.clone());
        let workspace_id = meta
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        let workspace_uuid = Uuid::parse_str(&workspace_id).ok();
        if find_active_deletion_track_id(state, &document_id, workspace_uuid)
            .await
            .is_some()
        {
            continue;
        }

        let deletion_track_id = Uuid::new_v4().to_string();
        let tenant_id = meta
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        let data = DeletionTaskData {
            document_id: document_id.clone(),
            key_prefix: key_prefix.clone(),
            workspace_id: workspace_id.clone(),
            tenant_id: tenant_id.clone(),
            deletion_track_id: deletion_track_id.clone(),
            metadata_key: Some(key),
            chunk_ids: Vec::new(),
            has_content: false,
            content_hash: meta
                .get("content_hash")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            pdf_id: meta
                .get("pdf_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ingest_track_id: meta
                .get("track_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            document_status: Some("deleting".to_string()),
        };
        let task = Task::new(
            Uuid::parse_str(&tenant_id).unwrap_or_else(|_| Uuid::nil()),
            Uuid::parse_str(&workspace_id).unwrap_or_else(|_| Uuid::nil()),
            TaskType::Deletion,
            serde_json::to_value(&data).unwrap_or_default(),
        );
        match state.enqueue_task(task).await {
            Ok(()) => {
                tracing::info!(
                    document_id = %document_id,
                    track_id = %deletion_track_id,
                    "Re-enqueued stuck deleting document"
                );
                requeued += 1;
            }
            Err(e) => {
                tracing::warn!(
                    document_id = %document_id,
                    error = %e,
                    "Failed to re-enqueue stuck deleting document"
                );
            }
        }
    }
    requeued
}

/// Find a pending/processing Deletion task for the same document (enqueue dedup).
pub async fn find_active_deletion_track_id(
    state: &AppState,
    document_id: &str,
    workspace_id: Option<Uuid>,
) -> Option<String> {
    use edgequake_tasks::{Pagination, TaskFilter, TaskStatus, TaskType};

    for status in [TaskStatus::Pending, TaskStatus::Processing] {
        let list = state
            .tasks
            .storage
            .list_tasks(
                TaskFilter {
                    workspace_id,
                    status: Some(status),
                    task_type: Some(TaskType::Deletion),
                    ..Default::default()
                },
                Pagination {
                    page: 1,
                    page_size: 100,
                    ..Default::default()
                },
            )
            .await
            .ok()?;
        for task in list.tasks {
            if let Ok(data) = serde_json::from_value::<DeletionTaskData>(task.task_data.clone()) {
                if data.document_id == document_id || data.key_prefix == document_id {
                    return Some(data.deletion_track_id);
                }
            }
        }
    }
    None
}
