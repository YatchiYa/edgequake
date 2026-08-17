//! Durable workspace wipe-all phase machine (issue #309 / SPEC-050).
//!
//! Invariants:
//! - Cancel all workspace ingestion first, then clear graph/vectors once, then purge docs.
//! - Never run N× `find_*_by_source_prefixes` (AGE LIKE SeqScans → timeout/OOM).
//! - Graph/vector clear failures are retryable task failures (fail-closed).
//! - HTTP 202 only admits; this worker owns terminal success/failure.

use edgequake_tasks::{
    Task, TaskStatus, WipeActivePolicy, WorkspaceWipePhase, WorkspaceWipeTaskData,
};
<<<<<<< HEAD
use serde_json::Value;
=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::middleware::{resolve_workspace_uuid, TenantContext};
<<<<<<< HEAD
use crate::services::document_metadata_scan::{
    document_id_from_metadata_key, load_scoped_document_metadata_entries,
};
use crate::services::document_task_cleanup::purge_workspace_tasks_except;
use crate::state::AppState;

/// Bounded batch size for KV document purge (deterministic, not a wall-clock timeout).
const PURGE_BATCH_SIZE: usize = 50;

#[derive(Debug, Clone)]
struct WipeDoc {
    metadata_key: String,
    document_id: String,
    #[cfg_attr(not(feature = "postgres"), allow(dead_code))]
    pdf_id: Option<String>,
}

fn list_all_wipe_docs(scoped_entries: Vec<(String, Value)>) -> Vec<WipeDoc> {
    let mut docs = Vec::with_capacity(scoped_entries.len());
    for (metadata_key, metadata) in scoped_entries {
        let document_id = document_id_from_metadata_key(&metadata_key).unwrap_or_else(|| {
            metadata
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(&metadata_key)
                .to_string()
        });
        let pdf_id = metadata
            .get("pdf_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        docs.push(WipeDoc {
            metadata_key,
            document_id,
            pdf_id,
        });
    }
    docs
}

=======
use crate::services::document_metadata_scan::load_scoped_document_metadata_entries;
use crate::services::document_task_cleanup::purge_workspace_tasks_except;
use crate::state::AppState;

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
async fn persist_wipe_checkpoint(
    state: &AppState,
    task: &mut Task,
    data: &WorkspaceWipeTaskData,
) -> ApiResult<()> {
    task.task_data = serde_json::to_value(data)
        .map_err(|e| ApiError::Internal(format!("serialize WorkspaceWipeTaskData: {e}")))?;
    task.updated_at = chrono::Utc::now();
    state
        .tasks
        .storage
        .update_task(task)
        .await
        .map_err(|e| ApiError::Internal(format!("persist wipe checkpoint: {e}")))?;
    Ok(())
}

async fn cancel_inflight_except_wipe(
    state: &AppState,
    workspace_uuid: Uuid,
    wipe_track_id: &str,
) -> ApiResult<usize> {
    let purged = purge_workspace_tasks_except(state, workspace_uuid, wipe_track_id).await;
    Ok(purged)
}

async fn clear_graph_fail_closed(
    state: &AppState,
    workspace_uuid: Uuid,
) -> ApiResult<(usize, usize)> {
    state
        .storage
        .graph_storage
        .clear_workspace(&workspace_uuid)
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "workspace wipe graph clear failed (retryable): {e}"
            ))
        })
}

async fn clear_vectors_fail_closed(state: &AppState, workspace_uuid: Uuid) -> ApiResult<usize> {
<<<<<<< HEAD
    state
=======
    let legacy_n = state
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        .storage
        .vector_storage
        .clear_workspace(&workspace_uuid)
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "workspace wipe vector clear failed (retryable): {e}"
            ))
<<<<<<< HEAD
        })
}

async fn purge_one_document_kv(
    state: &AppState,
    doc: &WipeDoc,
    tenant_ctx: &TenantContext,
) -> ApiResult<usize> {
    let chunk_prefix = format!("{}-chunk-", doc.document_id);
    let chunk_ids = state
        .storage
        .kv_storage
        .keys_with_prefix(&chunk_prefix)
        .await
        .unwrap_or_default();
    let content_key = format!("{}-content", doc.document_id);

    if !chunk_ids.is_empty() {
        state
            .storage
            .kv_storage
            .delete(&chunk_ids)
            .await
            .map_err(|e| ApiError::Internal(format!("wipe delete chunks: {e}")))?;
    }
    state
        .storage
        .kv_storage
        .delete(std::slice::from_ref(&doc.metadata_key))
        .await
        .map_err(|e| ApiError::Internal(format!("wipe delete metadata: {e}")))?;
    let _ = state
        .storage
        .kv_storage
        .delete(std::slice::from_ref(&content_key))
        .await;

    #[cfg(feature = "postgres")]
    {
        let mm_storage = state.storage.mm_asset_storage.as_deref();
        let workspace_uuid = uuid::Uuid::parse_str(&tenant_ctx.workspace_id_or_default()).ok();
        if let Err(e) =
            crate::services::delete_document_mm_assets(mm_storage, &doc.document_id, workspace_uuid)
                .await
        {
            return Err(ApiError::Internal(format!(
                "workspace wipe mm-asset delete failed (retryable) doc={}: {e}",
                doc.document_id
            )));
        }
        if let Some(ref pdf_id_str) = doc.pdf_id {
            if let (Some(ref pdf_storage), Ok(pdf_uuid)) = (
                &state.storage.pdf_storage,
                uuid::Uuid::parse_str(pdf_id_str),
            ) {
                if let Err(e) = pdf_storage.delete_pdf(&pdf_uuid).await {
                    return Err(ApiError::Internal(format!(
                        "workspace wipe PDF delete failed (retryable) pdf_id={pdf_id_str}: {e}"
                    )));
                }
            }
        }
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = tenant_ctx;
    }

    Ok(chunk_ids.len())
=======
        })?;

    // SPEC-091: under typed authority, also clear typed SSOT (legacy clear is write-stopped).
    let mut typed_n = 0usize;
    #[cfg(feature = "postgres")]
    if edgequake_storage::legacy_vector_writes_stopped() {
        if let Some(ref pool) = state.pg_pool {
            let model = std::env::var("EDGEQUAKE_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string());
            let chunk_index = edgequake_storage::PgChunkEmbeddingIndex::new(pool.clone(), &model);
            let fleet = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), &model);
            let ws = edgequake_storage::traits::domain::WorkspaceId(workspace_uuid);
            use edgequake_storage::embedding_family::EmbeddingFamily;
            use edgequake_storage::traits::domain::{EmbeddingIndex, FleetEmbeddingIndex};
            typed_n += chunk_index.delete_for_workspace(ws).await.map_err(|e| {
                ApiError::Internal(format!("wipe typed chunk_embeddings failed: {e}"))
            })? as usize;
            for family in [
                EmbeddingFamily::Entity,
                EmbeddingFamily::Relationship,
                EmbeddingFamily::Report,
            ] {
                typed_n += fleet.delete_for_workspace(family, ws).await.map_err(|e| {
                    ApiError::Internal(format!(
                        "wipe typed fleet embeddings ({family:?}) failed: {e}"
                    ))
                })? as usize;
            }
        }
    }

    Ok(legacy_n.saturating_add(typed_n))
}

/// SPEC-091 RM1: set-based chunk delete for a workspace (O(1) SQL, not O(docs)).
#[cfg(feature = "postgres")]
async fn delete_chunks_for_workspace(
    pool: Option<&sqlx::PgPool>,
    workspace_uuid: Uuid,
) -> ApiResult<u64> {
    let Some(pool) = pool else {
        return Ok(0);
    };
    let result = sqlx::query(
        r#"
        DELETE FROM public.chunks c
        USING public.documents d
        WHERE c.document_id = d.id
          AND (
            d.workspace_id = $1
            OR (d.workspace_id IS NULL AND d.metadata->>'workspace_id' = $2)
          )
        "#,
    )
    .bind(workspace_uuid)
    .bind(workspace_uuid.to_string())
    .execute(pool)
    .await
    .map_err(|e| ApiError::Internal(format!("wipe typed chunks failed: {e}")))?;
    Ok(result.rows_affected())
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

/// Execute (or resume) a durable workspace wipe from the task checkpoint.
pub async fn run_workspace_wipe_phases(
    state: &AppState,
    task: &mut Task,
    mut data: WorkspaceWipeTaskData,
) -> ApiResult<WorkspaceWipeTaskData> {
    let tenant_ctx = TenantContext {
        tenant_id: Some(data.tenant_id.clone()),
        workspace_id: Some(data.workspace_id.clone()),
        user_id: None,
    };
    let workspace_uuid = resolve_workspace_uuid(Some(&data.workspace_id)).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "invalid workspace_id for wipe: {}",
            data.workspace_id
        ))
    })?;

    let _ = data.active_policy; // ForceCancelAll is the only policy today
    let wipe_track_id = data.wipe_track_id.clone();
    let planned_total = data.planned_delete_count.max(1);

    loop {
        match data.phase {
            WorkspaceWipePhase::Admitted | WorkspaceWipePhase::CancellingInflight => {
                data.phase = WorkspaceWipePhase::CancellingInflight;
                persist_wipe_checkpoint(state, task, &data).await?;

                state.tasks.progress_broadcaster.bulk_deletion_started(
                    planned_total,
                    Some(&wipe_track_id),
                    Some(&data.workspace_id),
                );

                let purged =
                    cancel_inflight_except_wipe(state, workspace_uuid, &wipe_track_id).await?;
                tracing::info!(
                    workspace_id = %workspace_uuid,
                    wipe_track_id = %wipe_track_id,
                    tasks_purged = purged,
                    "Workspace wipe cancelled inflight tasks"
                );
                data.phase = WorkspaceWipePhase::ClearingGraph;
                persist_wipe_checkpoint(state, task, &data).await?;
            }
            WorkspaceWipePhase::ClearingGraph => {
                let (nodes, edges) = clear_graph_fail_closed(state, workspace_uuid).await?;
                data.total_entities_removed = data.total_entities_removed.max(nodes);
                data.total_relationships_removed = data.total_relationships_removed.max(edges);
                tracing::info!(
                    workspace_id = %workspace_uuid,
                    nodes_cleared = nodes,
                    edges_cleared = edges,
                    "Workspace wipe cleared graph once"
                );
                data.phase = WorkspaceWipePhase::ClearingVectors;
                persist_wipe_checkpoint(state, task, &data).await?;
            }
            WorkspaceWipePhase::ClearingVectors => {
                let vectors = clear_vectors_fail_closed(state, workspace_uuid).await?;
                tracing::info!(
                    workspace_id = %workspace_uuid,
                    vectors_cleared = vectors,
                    "Workspace wipe cleared vectors once"
                );
<<<<<<< HEAD
                data.phase = WorkspaceWipePhase::PurgingDocumentKv;
                persist_wipe_checkpoint(state, task, &data).await?;
            }
            WorkspaceWipePhase::PurgingDocumentKv => {
                let scoped = load_scoped_document_metadata_entries(
                    state.storage.kv_storage.as_ref(),
                    &tenant_ctx,
                )
                .await?;
                let mut docs = list_all_wipe_docs(scoped);
                // Resume after cursor: skip keys <= cursor (lexicographic on metadata_key).
                if let Some(ref cursor) = data.cursor_metadata_key {
                    docs.retain(|d| d.metadata_key.as_str() > cursor.as_str());
                }
                docs.sort_by(|a, b| a.metadata_key.cmp(&b.metadata_key));

                if docs.is_empty() {
                    data.cursor_metadata_key = None;
                    data.phase = WorkspaceWipePhase::ClearingRelational;
                    persist_wipe_checkpoint(state, task, &data).await?;
                    continue;
                }

                let batch: Vec<_> = docs.into_iter().take(PURGE_BATCH_SIZE).collect();
                for doc in &batch {
                    let chunks = purge_one_document_kv(state, doc, &tenant_ctx).await?;
                    data.total_chunks_deleted += chunks;
                    data.deleted_count += 1;
                    if doc.pdf_id.is_some() {
                        data.total_pdfs_deleted += 1;
                    }
                    state
                        .tasks
                        .progress_broadcaster
                        .bulk_deletion_item_progress(
                            crate::handlers::websocket_types::BulkDeletionItemProgressArgs {
                                document_id: &doc.document_id,
                                completed: data.deleted_count,
                                total: planned_total.max(data.deleted_count),
                                entities_removed: data.total_entities_removed,
                                relationships_removed: data.total_relationships_removed,
                                wipe_track_id: Some(&wipe_track_id),
                                workspace_id: Some(&data.workspace_id),
                            },
                        );
                }
                data.cursor_metadata_key = batch.last().map(|d| d.metadata_key.clone());
                persist_wipe_checkpoint(state, task, &data).await?;
                // Stay in PurgingDocumentKv until a batch returns empty.
=======
                // SPEC-091 RM1: skip O(docs) KV purge — typed set-delete in ClearingRelational.
                data.cursor_metadata_key = None;
                data.phase = WorkspaceWipePhase::ClearingRelational;
                persist_wipe_checkpoint(state, task, &data).await?;
            }
            WorkspaceWipePhase::PurgingDocumentKv => {
                // Legacy phase retained for in-flight wipe task checkpoints only.
                // Post-125 / RM1: no per-doc KV loops — jump to set-based relational clear.
                tracing::info!(
                    workspace_id = %workspace_uuid,
                    "SPEC-091 RM1: skipping PurgingDocumentKv (typed set-delete)"
                );
                data.cursor_metadata_key = None;
                data.phase = WorkspaceWipePhase::ClearingRelational;
                persist_wipe_checkpoint(state, task, &data).await?;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            }
            WorkspaceWipePhase::ClearingRelational => {
                #[cfg(feature = "postgres")]
                {
<<<<<<< HEAD
=======
                    // RM-AC-05: O(families) set deletes — chunks cascade via FK from documents.
                    let chunks_deleted =
                        delete_chunks_for_workspace(state.pg_pool.as_ref(), workspace_uuid).await?;
                    data.total_chunks_deleted = data
                        .total_chunks_deleted
                        .saturating_add(chunks_deleted as usize);

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
                    let relational_deleted =
                        crate::document_read_model::delete_relational_documents_for_workspace(
                            state.pg_pool.as_ref(),
                            &tenant_ctx,
                        )
                        .await?;
<<<<<<< HEAD
                    if relational_deleted > 0 {
                        tracing::info!(
                            workspace_id = %workspace_uuid,
                            relational_deleted,
                            "Workspace wipe cleared relational document rows"
=======
                    data.deleted_count = data.deleted_count.max(relational_deleted as usize);
                    if relational_deleted > 0 || chunks_deleted > 0 {
                        tracing::info!(
                            workspace_id = %workspace_uuid,
                            relational_deleted,
                            chunks_deleted,
                            "Workspace wipe cleared typed document/chunk rows (set-based)"
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
                        );
                    }
                }
                data.phase = WorkspaceWipePhase::Completed;
                persist_wipe_checkpoint(state, task, &data).await?;
            }
            WorkspaceWipePhase::Completed => {
                state.tasks.progress_broadcaster.bulk_deletion_completed(
                    data.deleted_count,
                    data.skipped_document_ids.len(),
                    data.total_entities_removed,
                    data.total_relationships_removed,
                    Some(&wipe_track_id),
                    Some(&data.workspace_id),
                );
                state.tasks.wipe_admission.release(workspace_uuid);
                return Ok(data);
            }
        }
    }
}

/// Count documents that will be wiped (admit-time planned count).
pub async fn count_planned_wipe_documents(
    state: &AppState,
    tenant_ctx: &TenantContext,
) -> ApiResult<usize> {
<<<<<<< HEAD
=======
    #[cfg(feature = "postgres")]
    if let Some(pool) = state.pg_pool.as_ref() {
        if let Some(ws) = tenant_ctx
            .workspace_id
            .as_ref()
            .and_then(|w| Uuid::parse_str(w).ok())
        {
            let n: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM public.documents
                WHERE workspace_id = $1
                   OR (workspace_id IS NULL AND metadata->>'workspace_id' = $2)
                "#,
            )
            .bind(ws)
            .bind(ws.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| ApiError::Internal(format!("count wipe documents: {e}")))?;
            return Ok(n.max(0) as usize);
        }
    }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    let scoped =
        load_scoped_document_metadata_entries(state.storage.kv_storage.as_ref(), tenant_ctx)
            .await?;
    Ok(scoped.len())
}

/// Admit helper: build task data for a new wipe (ForceCancelAll — never skips active docs).
pub fn new_wipe_task_data(
    tenant_id: String,
    workspace_id: String,
    wipe_track_id: String,
    planned_delete_count: usize,
) -> WorkspaceWipeTaskData {
    WorkspaceWipeTaskData {
        tenant_id,
        workspace_id,
        wipe_track_id,
        phase: WorkspaceWipePhase::Admitted,
        deleted_count: 0,
        skipped_document_ids: Vec::new(),
        cursor_metadata_key: None,
        active_policy: WipeActivePolicy::ForceCancelAll,
        total_chunks_deleted: 0,
        total_entities_removed: 0,
        total_relationships_removed: 0,
        total_pdfs_deleted: 0,
        planned_delete_count,
    }
}

/// Broadcast terminal failure for a permanently failed wipe task.
pub fn broadcast_wipe_failed(state: &AppState, data: &WorkspaceWipeTaskData, error_message: &str) {
    if let Some(ws) = resolve_workspace_uuid(Some(&data.workspace_id)) {
        state.tasks.wipe_admission.release(ws);
    }
    state.tasks.progress_broadcaster.bulk_deletion_failed(
        &data.wipe_track_id,
        Some(&data.workspace_id),
        error_message,
        data.deleted_count,
    );
}

/// Mark wipe task status helper for tests / recovery.
pub fn wipe_is_active(status: TaskStatus) -> bool {
    matches!(status, TaskStatus::Pending | TaskStatus::Processing)
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< HEAD
    use serde_json::json;

    #[test]
    fn list_all_includes_active_processing() {
        let entries = vec![
            (
                "doc-1-metadata".to_string(),
                json!({"id": "doc-1", "status": "processing"}),
            ),
            (
                "doc-2-metadata".to_string(),
                json!({"id": "doc-2", "status": "completed"}),
            ),
        ];
        let docs = list_all_wipe_docs(entries);
        assert_eq!(docs.len(), 2);
    }
=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

    #[test]
    fn new_wipe_starts_admitted() {
        let data = new_wipe_task_data("default".into(), "ws".into(), "wipe-1".into(), 10);
        assert_eq!(data.phase, WorkspaceWipePhase::Admitted);
        assert_eq!(data.planned_delete_count, 10);
        assert!(matches!(
            data.active_policy,
            WipeActivePolicy::ForceCancelAll
        ));
    }
}
