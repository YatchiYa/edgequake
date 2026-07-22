//! Authoritative document cascade delete (DRY for handler worker + bulk).
//!
//! First principles:
//! - HTTP admits `status=deleting` and returns 202; this service runs async.
//! - Order: graph discover/mutate/post-proof → vectors → KV/PDF/relational.
//! - Never wipe metadata before graph provenance is proved absent.
//! - Fail closed with `delete_failed` status (never leave permanent `deleting`).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use edgequake_audit::{AuditEventType, AuditResult};
use edgequake_core::MetricsTriggerType;
use edgequake_tasks::DeletionTaskData;

use crate::error::{ApiError, ApiResult};
use crate::handlers::websocket_types::DeletionPhaseKind;
use crate::middleware::TenantContext;
use crate::services::document_graph_cascade::{find_document_edges, find_document_nodes};
use crate::services::document_metadata_scan::metadata_key_for_document;
use crate::services::document_task_cleanup::purge_persisted_tasks_for_document_except;
use crate::services::document_vector_storage::get_workspace_vector_storage_for_delete;
use crate::services::{
    cascade_remove_document_sources_with_progress, record_compliance_event, ContentHasher,
    DocumentSourceScope,
};
use crate::state::AppState;

const DELETION_CHECKPOINT_KEY: &str = "deletion_checkpoint";
const CHECKPOINT_GRAPH_DONE: &str = "graph_done";
const CHECKPOINT_VECTORS_DONE: &str = "vectors_done";

async fn read_deletion_checkpoint(state: &AppState, metadata_key: &str) -> Option<String> {
    let meta = state
        .storage
        .kv_storage
        .get_by_id(metadata_key)
        .await
        .ok()
        .flatten()?;
    meta.get(DELETION_CHECKPOINT_KEY)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

async fn write_deletion_checkpoint(
    state: &AppState,
    metadata_key: &str,
    has_metadata: bool,
    phase: &str,
) {
    if !has_metadata {
        return;
    }
    if let Ok(Some(mut metadata)) = state.storage.kv_storage.get_by_id(metadata_key).await {
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert(
                DELETION_CHECKPOINT_KEY.to_string(),
                serde_json::json!(phase),
            );
            let _ = crate::services::upsert_metadata_kv_with_index(
                state.storage.kv_storage.as_ref(),
                metadata_key,
                metadata,
            )
            .await;
        }
    }
}

/// Diagnostic unscoped probe: source-owned rows exist but scoped discovery is empty.
async fn scoped_discovery_has_filter_mismatch(
    state: &AppState,
    tenant_ctx: &TenantContext,
    scope: &DocumentSourceScope,
) -> ApiResult<bool> {
    let scoped_nodes =
        find_document_nodes(&state.storage.graph_storage, Some(tenant_ctx), scope).await?;
    let scoped_edges =
        find_document_edges(&state.storage.graph_storage, Some(tenant_ctx), scope).await?;
    if !scoped_nodes.is_empty() || !scoped_edges.is_empty() {
        return Ok(false);
    }
    let unscoped_nodes = find_document_nodes(&state.storage.graph_storage, None, scope).await?;
    let unscoped_edges = find_document_edges(&state.storage.graph_storage, None, scope).await?;
    Ok(!unscoped_nodes.is_empty() || !unscoped_edges.is_empty())
}

async fn post_proof_source_absent(
    state: &AppState,
    tenant_ctx: &TenantContext,
    scope: &DocumentSourceScope,
) -> ApiResult<()> {
    let nodes = find_document_nodes(&state.storage.graph_storage, Some(tenant_ctx), scope).await?;
    let edges = find_document_edges(&state.storage.graph_storage, Some(tenant_ctx), scope).await?;
    if nodes.is_empty() && edges.is_empty() {
        return Ok(());
    }
    Err(ApiError::Internal(format!(
        "Post-proof failed: {} nodes and {} edges still reference document sources",
        nodes.len(),
        edges.len()
    )))
}

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

/// Run the authoritative cascade for a document (graph → vectors → KV → relational).
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

    // Keep the running deletion task itself (DRY with wipe keep-self). Matching
    // on document_id alone used to cancel+delete this row mid-cascade.
    let persisted_tasks_removed = purge_persisted_tasks_for_document_except(
        state,
        &document_id,
        data.ingest_track_id.as_deref(),
        Some(&workspace_id_for_storage),
        &deletion_track_id,
    )
    .await;

    let workspace_vector_storage =
        get_workspace_vector_storage_for_delete(state, &workspace_id_for_storage).await;

    let chunks_deleted = chunk_ids.len();
    let mut embeddings_deleted = 0usize;
    let partial_failure = false;
    let partial_failure_reason: Option<String> = None;

    let scope =
        DocumentSourceScope::with_key_prefix(document_id.clone(), actual_key_prefix.clone());
    let checkpoint = read_deletion_checkpoint(state, &metadata_key).await;
    let graph_already_done = checkpoint.as_deref() == Some(CHECKPOINT_GRAPH_DONE)
        || checkpoint.as_deref() == Some(CHECKPOINT_VECTORS_DONE);

    let mut entities_removed = 0usize;
    let mut entities_updated = 0usize;
    let mut relationships_removed = 0usize;
    let mut relationships_updated = 0usize;

    if !graph_already_done {
        state.tasks.progress_broadcaster.deletion_phase(
            &document_id,
            &deletion_track_id,
            DeletionPhaseKind::RemovingGraph,
            0,
            0,
        );

        // Zero-match guard: scoped empty but unscoped hits ⇒ filter mismatch, fail closed.
        match scoped_discovery_has_filter_mismatch(state, tenant_ctx, &scope).await {
            Ok(true) => {
                let reason = "Graph discovery filter mismatch: source-owned rows exist but scoped discovery returned zero — retaining metadata as delete_failed";
                tracing::error!(document_id = %document_id, "{reason}");
                reset_deleting_status(
                    state,
                    &document_id,
                    &actual_key_prefix,
                    reason,
                    Some(deletion_track_id.as_str()),
                )
                .await;
                return Err(ApiError::Internal(reason.into()));
            }
            Err(e) => {
                let reason = format!("Graph discovery error: {e}");
                reset_deleting_status(
                    state,
                    &document_id,
                    &actual_key_prefix,
                    &reason,
                    Some(deletion_track_id.as_str()),
                )
                .await;
                return Err(e);
            }
            Ok(false) => {}
        }

        // ISSUE-305: fail closed — never wipe KV/docs if graph cascade cannot run.
        // SPEC-069: progress ticks + periodic heartbeats during long shared upserts.
        let cascade_stats = {
            let mut last_err: Option<ApiError> = None;
            let mut stats = None;
            let last_processed = Arc::new(AtomicU32::new(0));
            let last_total = Arc::new(AtomicU32::new(0));
            for attempt in 1u8..=2 {
                let hb_doc = document_id.clone();
                let hb_track = deletion_track_id.clone();
                let hb_proc = Arc::clone(&last_processed);
                let hb_tot = Arc::clone(&last_total);
                let hb_broadcast = state.tasks.progress_broadcaster.clone();
                let heartbeat = tokio::spawn(async move {
                    let mut tick = tokio::time::interval(Duration::from_secs(3));
                    tick.tick().await; // skip immediate first fire
                    loop {
                        tick.tick().await;
                        hb_broadcast.deletion_phase(
                            &hb_doc,
                            &hb_track,
                            DeletionPhaseKind::RemovingGraph,
                            hb_proc.load(Ordering::Relaxed),
                            hb_tot.load(Ordering::Relaxed),
                        );
                    }
                });

                let broadcast = state.tasks.progress_broadcaster.clone();
                let doc_id = document_id.clone();
                let track = deletion_track_id.clone();
                let proc_ref = Arc::clone(&last_processed);
                let tot_ref = Arc::clone(&last_total);
                let result = cascade_remove_document_sources_with_progress(
                    &state.storage.graph_storage,
                    Some(&workspace_vector_storage),
                    Some(tenant_ctx),
                    &scope,
                    |processed, total| {
                        proc_ref.store(processed, Ordering::Relaxed);
                        tot_ref.store(total, Ordering::Relaxed);
                        broadcast.deletion_phase(
                            &doc_id,
                            &track,
                            DeletionPhaseKind::RemovingGraph,
                            processed,
                            total,
                        );
                    },
                )
                .await;
                heartbeat.abort();

                match result {
                    Ok(s) => {
                        stats = Some(s);
                        break;
                    }
                    Err(e) => {
                        tracing::warn!(
                            document_id = %document_id,
                            attempt,
                            error = %e,
                            "Graph cascade delete failed"
                        );
                        last_err = Some(e);
                    }
                }
            }
            match stats {
                Some(s) => s,
                None => {
                    let reason = format!(
                        "Graph cascade error: {}",
                        last_err
                            .as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    );
                    tracing::error!(
                        document_id = %document_id,
                        %reason,
                        "Graph cascade delete failed after retry — aborting KV wipe (fail-closed)"
                    );
                    reset_deleting_status(
                        state,
                        &document_id,
                        &actual_key_prefix,
                        &reason,
                        Some(deletion_track_id.as_str()),
                    )
                    .await;
                    return Err(last_err.unwrap_or_else(|| {
                        ApiError::Internal("Graph cascade delete failed".into())
                    }));
                }
            }
        };

        if let Err(e) = post_proof_source_absent(state, tenant_ctx, &scope).await {
            let reason = e.to_string();
            tracing::error!(
                document_id = %document_id,
                %reason,
                "Post-proof residual provenance — aborting vector/KV wipe"
            );
            reset_deleting_status(
                state,
                &document_id,
                &actual_key_prefix,
                &reason,
                Some(deletion_track_id.as_str()),
            )
            .await;
            return Err(e);
        }

        entities_removed = cascade_stats.entities_removed;
        entities_updated = cascade_stats.entities_updated;
        relationships_removed = cascade_stats.relationships_removed;
        relationships_updated = cascade_stats.relationships_updated;
        embeddings_deleted += cascade_stats.embeddings_deleted;
        write_deletion_checkpoint(state, &metadata_key, has_metadata, CHECKPOINT_GRAPH_DONE).await;
    } else {
        // Resume after graph: still re-prove before vectors/KV.
        if let Err(e) = post_proof_source_absent(state, tenant_ctx, &scope).await {
            let reason = e.to_string();
            reset_deleting_status(
                state,
                &document_id,
                &actual_key_prefix,
                &reason,
                Some(deletion_track_id.as_str()),
            )
            .await;
            return Err(e);
        }
    }

    let vectors_already_done = checkpoint.as_deref() == Some(CHECKPOINT_VECTORS_DONE);
    if !vectors_already_done {
        state.tasks.progress_broadcaster.deletion_phase(
            &document_id,
            &deletion_track_id,
            DeletionPhaseKind::RemovingVectors,
            0,
            chunk_ids.len() as u32,
        );

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
                        let reason = format!("Vector delete failed: {e2}");
                        reset_deleting_status(
                            state,
                            &document_id,
                            &actual_key_prefix,
                            &reason,
                            Some(deletion_track_id.as_str()),
                        )
                        .await;
                        return Err(ApiError::Internal(reason));
                    }
                    embeddings_deleted += chunk_ids.len();
                }
            }
        }
        if key_id_mismatch {
            let _ = workspace_vector_storage
                .delete_by_document(&actual_key_prefix)
                .await;
        }
        write_deletion_checkpoint(state, &metadata_key, has_metadata, CHECKPOINT_VECTORS_DONE)
            .await;
    }

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
            deletion_track_id: String::new(),
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
        let mut task = Task::new(
            Uuid::parse_str(&tenant_id).unwrap_or_else(|_| Uuid::nil()),
            Uuid::parse_str(&workspace_id).unwrap_or_else(|_| Uuid::nil()),
            TaskType::Deletion,
            serde_json::to_value(&data).unwrap_or_default(),
        );
        let deletion_track_id = task.track_id.clone();
        if let Some(obj) = task.task_data.as_object_mut() {
            obj.insert(
                "deletion_track_id".to_string(),
                serde_json::json!(&deletion_track_id),
            );
        }
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
