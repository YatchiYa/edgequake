use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::services::ContentHasher;
use crate::state::AppState;
use edgequake_core::MetricsTriggerType;
#[cfg(feature = "postgres")]
use edgequake_storage::ListPdfFilter;

use crate::handlers::documents_types::*;
use super::storage_helpers::{extract_source_docs, get_workspace_vector_storage_strict};
#[allow(unused_imports)]
use super::storage_helpers::get_workspace_vector_storage_with_fallback;

/// Delete a document by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/documents/{document_id}",
    tag = "Documents",
    params(
        ("document_id" = String, Path, description = "Document ID to delete")
    ),
    responses(
        (status = 200, description = "Document deleted", body = DeleteDocumentResponse),
        (status = 404, description = "Document not found")
    )
)]
pub async fn delete_document(
    State(state): State<AppState>,
    axum::extract::Path(document_id): axum::extract::Path<String>,
) -> ApiResult<Json<DeleteDocumentResponse>> {
    let keys = state.kv_storage.keys().await?;

    // Find chunks belonging to this document
    let chunk_prefix = format!("{}-chunk-", document_id);
    let chunk_ids: Vec<String> = keys
        .iter()
        .filter(|k| k.starts_with(&chunk_prefix))
        .cloned()
        .collect();

    // Also check for metadata and content keys
    let metadata_key = format!("{}-metadata", document_id);
    let content_key = format!("{}-content", document_id);
    let has_metadata = keys.contains(&metadata_key);
    let has_content = keys.contains(&content_key);

    // Document must have either chunks, metadata, or content
    if chunk_ids.is_empty() && !has_metadata && !has_content {
        return Err(ApiError::NotFound(format!(
            "Document {} not found",
            document_id
        )));
    }

    // SPEC-033: Get workspace_id from document metadata for vector storage isolation
    // OODA-02: Also check document status for safe deletion
    // OODA-90: Extract content_hash for hash key cleanup
    let (workspace_id_for_storage, document_status, content_hash_opt) = if has_metadata {
        if let Ok(Some(metadata)) = state.kv_storage.get_by_id(&metadata_key).await {
            let workspace = metadata
                .get("workspace_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "default".to_string());
            let status = metadata
                .get("status")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            // OODA-90: Extract content hash for duplicate detection key cleanup
            let content_hash = metadata
                .get("content_hash")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (workspace, status, content_hash)
        } else {
            ("default".to_string(), "unknown".to_string(), None)
        }
    } else {
        ("default".to_string(), "unknown".to_string(), None)
    };

    // OODA-02: Safety check - prevent deletion of documents that are still being processed
    // WHY: Deleting a document while it's being processed can cause:
    //   1. Race condition: Background task writes data while deletion removes it
    //   2. Orphaned data: Entities/edges created AFTER deletion check starts
    //   3. Partial deletion: Some entities exist, others don't
    //
    // Status lifecycle (FIX-5: Added partial_failure):
    //   "pending"         → Cannot delete (queued for processing)
    //   "processing"      → Cannot delete (actively being processed)
    //   "completed"       → Can delete (processing finished successfully with entities)
    //   "processed"       → Can delete (legacy status, same as completed)
    //   "partial_failure" → Can delete (processed but 0 entities extracted - FIX-5)
    //   "failed"          → Can delete (processing failed, cleanup partial data)
    //   "unknown"         → Can delete (legacy documents without status)
    match document_status.as_str() {
        "pending" => {
            tracing::warn!(
                document_id = %document_id,
                status = %document_status,
                "Rejecting deletion of pending document"
            );
            return Err(ApiError::Conflict(format!(
                "Cannot delete document '{}' with status 'pending'. \
                 The document is queued for processing. \
                 Please wait for processing to complete or cancel the task.",
                document_id
            )));
        }
        "processing" => {
            tracing::warn!(
                document_id = %document_id,
                status = %document_status,
                "Rejecting deletion of processing document"
            );
            return Err(ApiError::Conflict(format!(
                "Cannot delete document '{}' with status 'processing'. \
                 The document is currently being processed. \
                 Please wait for processing to complete or cancel the task.",
                document_id
            )));
        }
        "completed" | "processed" | "partial_failure" | "failed" | "cancelled" | "unknown" => {
            // OK to delete
            // OODA-13: Added "cancelled" status to explicitly allow deletion after task cancellation
            tracing::debug!(
                document_id = %document_id,
                status = %document_status,
                "Document status allows deletion"
            );
        }
        other => {
            // Unknown status - allow deletion with warning
            tracing::warn!(
                document_id = %document_id,
                status = %other,
                "Unknown document status, allowing deletion"
            );
        }
    }

    // SPEC-028: Collect chunk IDs for vector storage deletion
    // Clone chunk_ids before workspace_vector_storage operations
    let keys_to_delete_for_vectors: Vec<String> = chunk_ids.clone();

    // SPEC-033: Get workspace-specific vector storage for deletion
    // WHY-OODA223: STRICT mode - fail loudly if workspace storage unavailable
    // to ensure we delete from the correct workspace table, not a fallback
    let workspace_vector_storage =
        get_workspace_vector_storage_strict(&state, &workspace_id_for_storage).await?;

    let chunks_deleted = chunk_ids.len();
    let mut entities_removed = 0usize;
    let mut entities_updated = 0usize;
    let mut relationships_removed = 0usize;
    let mut relationships_updated = 0usize;
    let mut embeddings_deleted = 0usize;

    // SPEC-028: Delete chunk embeddings from vector storage first
    // WHY: Chunks are stored with IDs like "doc-xxx-chunk-0", delete them
    let chunk_embedding_ids: Vec<String> = keys_to_delete_for_vectors.clone();
    if !chunk_embedding_ids.is_empty() {
        if let Err(e) = workspace_vector_storage.delete(&chunk_embedding_ids).await {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "Failed to delete chunk embeddings, continuing with graph cleanup"
            );
        } else {
            embeddings_deleted += chunk_embedding_ids.len();
            tracing::debug!(
                document_id = %document_id,
                count = chunk_embedding_ids.len(),
                "Deleted chunk embeddings"
            );
        }
    }

    // Cascade delete: Process graph entities - remove document sources
    let all_nodes = state.graph_storage.get_all_nodes().await?;
    for node in all_nodes {
        let sources = extract_source_docs(&node.properties);
        if sources.is_empty() {
            continue;
        }

        // Filter out sources that belong to this document
        let remaining_sources: Vec<String> = sources
            .iter()
            .filter(|s| {
                !s.starts_with(&chunk_prefix) && *s != &document_id && !s.starts_with(&document_id)
            })
            .cloned()
            .collect();

        if remaining_sources.is_empty() {
            // No sources left - delete the entity entirely

            // WHY-OODA01: DO NOT delete edges here!
            // Edges have their own source_ids tracking and will be processed
            // independently in the edge processing loop below (line ~1500).
            // Deleting them here would cause data loss if the edge has other
            // source documents that are not being deleted.
            //
            // Example bug scenario (fixed):
            //   Document A: "Alice works at Google"
            //   Document B: "Alice graduated from MIT"
            //   DELETE Document A:
            //     - ALICE entity sources: [doc_a, doc_b] → [doc_b] (update)
            //     - GOOGLE entity sources: [doc_a] → [] (delete entity)
            //     - OLD BUG: Deleted ALL edges from GOOGLE, including MIT edge!
            //     - FIXED: Edges are processed separately based on their own sources

            // Delete the node (backend may cascade edges, but we handle explicitly below)
            state.graph_storage.delete_node(&node.id).await?;
            // SPEC-033: Use workspace-specific vector storage for entity deletion
            let _ = workspace_vector_storage.delete_entity(&node.id).await;
            entities_removed += 1;
        } else if remaining_sources.len() < sources.len() {
            // Some sources were removed - update the entity
            let mut updated_props = node.properties.clone();
            // Use source_ids (JSON array) format for updates
            updated_props.insert(
                "source_ids".to_string(),
                serde_json::json!(remaining_sources),
            );
            state
                .graph_storage
                .upsert_node(&node.id, updated_props)
                .await?;
            entities_updated += 1;
        }
    }

    // Process graph edges - remove document sources
    // WHY-OODA01: We must also check for orphaned edges (edges connecting to deleted nodes)
    // This handles the case where a node was deleted above but edges still reference it.
    let all_edges = state.graph_storage.get_all_edges().await?;

    // Get current node IDs for orphan detection
    let existing_nodes = state.graph_storage.get_all_nodes().await?;
    let existing_node_ids: std::collections::HashSet<String> =
        existing_nodes.iter().map(|n| n.id.clone()).collect();

    for edge in all_edges {
        // Check if edge is orphaned (connects to deleted node)
        let is_orphaned =
            !existing_node_ids.contains(&edge.source) || !existing_node_ids.contains(&edge.target);

        if is_orphaned {
            // Edge connects to a deleted node - delete it
            state
                .graph_storage
                .delete_edge(&edge.source, &edge.target)
                .await?;
            relationships_removed += 1;
            tracing::debug!(
                source = %edge.source,
                target = %edge.target,
                "Deleted orphaned edge (connects to deleted node)"
            );
            continue;
        }

        let sources = extract_source_docs(&edge.properties);
        if sources.is_empty() {
            continue;
        }

        // Filter out sources that belong to this document
        let remaining_sources: Vec<String> = sources
            .iter()
            .filter(|s| {
                !s.starts_with(&chunk_prefix) && *s != &document_id && !s.starts_with(&document_id)
            })
            .cloned()
            .collect();

        if remaining_sources.is_empty() {
            // No sources left - delete the relationship
            state
                .graph_storage
                .delete_edge(&edge.source, &edge.target)
                .await?;
            relationships_removed += 1;
        } else if remaining_sources.len() < sources.len() {
            // Some sources were removed - update the relationship
            let mut updated_props = edge.properties.clone();
            // Use source_ids (JSON array) format for updates
            updated_props.insert(
                "source_ids".to_string(),
                serde_json::json!(remaining_sources),
            );
            state
                .graph_storage
                .upsert_edge(&edge.source, &edge.target, updated_props)
                .await?;
            relationships_updated += 1;
        }
    }

    // Collect all keys to delete from KV storage
    let mut keys_to_delete = keys_to_delete_for_vectors;
    if has_metadata {
        keys_to_delete.push(metadata_key);
    }
    if has_content {
        keys_to_delete.push(content_key);
    }

    // OODA-90: Delete workspace-scoped hash key to allow re-upload of same content
    // WHY: If we don't delete the hash key, the duplicate detection will still
    // block uploads of the same content even after the document is deleted.
    if let Some(content_hash) = content_hash_opt {
        let hash_key = ContentHasher::workspace_hash_key(&workspace_id_for_storage, &content_hash);
        keys_to_delete.push(hash_key.clone());
        tracing::debug!(
            hash_key = %hash_key,
            document_id = %document_id,
            "Adding hash key to deletion list for duplicate detection cleanup"
        );
    }

    // Delete all document data from KV storage
    state.kv_storage.delete(&keys_to_delete).await?;

    tracing::info!(
        document_id = %document_id,
        chunks = chunks_deleted,
        embeddings_deleted = embeddings_deleted,
        entities_removed = entities_removed,
        entities_updated = entities_updated,
        relationships_removed = relationships_removed,
        relationships_updated = relationships_updated,
        "Document cascade delete complete"
    );

    // OODA-21: Record metrics snapshot for trend analysis after deletion
    // Best-effort: log error but don't fail the deletion
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
        } else {
            tracing::debug!(
                workspace_id = %workspace_id_for_storage,
                "Recorded post-deletion metrics snapshot"
            );
        }
    }

    Ok(Json(DeleteDocumentResponse {
        document_id,
        deleted: true,
        chunks_deleted,
        entities_affected: entities_removed + entities_updated,
        relationships_affected: relationships_removed + relationships_updated,
    }))
}

/// Delete all documents in the system (bulk deletion).
///
/// This endpoint allows users to clear all documents from the system.
/// Documents that are actively being processed (pending/processing status)
/// will be skipped to prevent data corruption.
///
/// WHY: Frontend "Clear All" button needs this endpoint to remove stuck
/// or failed documents in bulk rather than deleting one by one.
#[utoipa::path(
    delete,
    path = "/api/v1/documents",
    tag = "Documents",
    responses(
        (status = 200, description = "Documents deleted", body = DeleteAllDocumentsResponse),
        (status = 500, description = "Internal error")
    )
)]
pub async fn delete_all_documents(
    State(state): State<AppState>,
) -> ApiResult<Json<DeleteAllDocumentsResponse>> {
    tracing::info!("Bulk delete all documents requested");

    let keys = state.kv_storage.keys().await?;

    // Find all document metadata keys to identify unique documents
    let metadata_keys: Vec<String> = keys
        .iter()
        .filter(|k| k.ends_with("-metadata"))
        .cloned()
        .collect();

    let mut deleted_count = 0usize;
    let mut total_chunks_deleted = 0usize;
    let mut total_entities_removed = 0usize;
    let mut total_relationships_removed = 0usize;
    let mut skipped_count = 0usize;
    let mut skipped_documents = Vec::new();

    // Define stuck threshold: documents processing for > 1 hour are considered stuck
    let stuck_threshold_secs = 3600; // 1 hour

    for metadata_key in &metadata_keys {
        // Extract document_id from metadata key (format: {document_id}-metadata)
        let document_id = metadata_key.trim_end_matches("-metadata").to_string();

        // Get document status and metadata to check if safe to delete
        let (status, updated_at_opt, stage_progress_opt) =
            if let Ok(Some(metadata)) = state.kv_storage.get_by_id(metadata_key).await {
                let status = metadata
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let updated_at = metadata
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));
                let stage_progress = metadata.get("stage_progress").and_then(|v| v.as_f64());
                (status, updated_at, stage_progress)
            } else {
                ("unknown".to_string(), None, None)
            };

        // Skip documents that are actively being processed (unless stuck)
        // A document is considered stuck if:
        //   - Status is "processing" or "pending"
        //   - AND updated_at is more than stuck_threshold_secs ago
        //   - AND stage_progress is 1.0 (100%) or close to it
        let is_stuck = if status == "pending" || status == "processing" {
            if let Some(updated_at) = updated_at_opt {
                let age_secs = (Utc::now() - updated_at).num_seconds();
                let high_progress = stage_progress_opt.map(|p| p >= 0.99).unwrap_or(false);
                age_secs > stuck_threshold_secs && high_progress
            } else {
                false
            }
        } else {
            false
        };

        if (status == "pending" || status == "processing") && !is_stuck {
            tracing::debug!(
                document_id = %document_id,
                status = %status,
                "Skipping bulk delete of document with active processing"
            );
            skipped_count += 1;
            skipped_documents.push(document_id.clone());
            continue;
        }

        if is_stuck {
            tracing::info!(
                document_id = %document_id,
                status = %status,
                "Deleting stuck document (>1 hour at 100% progress)"
            );
        }

        // Attempt to delete this document
        // We'll use a simplified version that doesn't require workspace isolation
        // since we're doing a full system clear
        let chunk_prefix = format!("{}-chunk-", document_id);
        let chunk_ids: Vec<String> = keys
            .iter()
            .filter(|k| k.starts_with(&chunk_prefix))
            .cloned()
            .collect();

        let content_key = format!("{}-content", document_id);

        // Delete from KV storage - delete takes a slice of strings
        if !chunk_ids.is_empty() {
            if let Err(e) = state.kv_storage.delete(&chunk_ids).await {
                tracing::warn!(document_id = %document_id, error = %e, "Failed to delete chunks");
            }
        }

        // Delete metadata key
        if let Err(e) = state
            .kv_storage
            .delete(std::slice::from_ref(metadata_key))
            .await
        {
            tracing::warn!(key = %metadata_key, error = %e, "Failed to delete metadata");
        }

        // Delete content key
        if let Err(e) = state
            .kv_storage
            .delete(std::slice::from_ref(&content_key))
            .await
        {
            tracing::warn!(key = %content_key, error = %e, "Failed to delete content");
        }

        // Delete from vector storage (use default storage for bulk operations)
        if !chunk_ids.is_empty() {
            if let Err(e) = state.vector_storage.delete(&chunk_ids).await {
                tracing::warn!(
                    document_id = %document_id,
                    error = %e,
                    "Failed to delete chunk embeddings"
                );
            }
        }

        total_chunks_deleted += chunk_ids.len();
        deleted_count += 1;

        tracing::debug!(
            document_id = %document_id,
            chunks = chunk_ids.len(),
            "Deleted document in bulk operation"
        );
    }

    // Clean up orphaned graph entities (entities with no remaining source documents)
    // This is a simplified cleanup - full cascade is done per-document for precision
    let all_nodes = state.graph_storage.get_all_nodes().await?;
    for node in all_nodes {
        // Check if node has any source references
        let has_sources = node
            .properties
            .get("source_ids")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        if !has_sources {
            // Node has no sources, check source_id too
            let has_legacy_source = node
                .properties
                .get("source_id")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);

            if !has_legacy_source {
                // No sources at all, delete the orphaned entity
                if let Err(e) = state.graph_storage.delete_node(&node.id).await {
                    tracing::warn!(node_id = %node.id, error = %e, "Failed to delete orphaned node");
                } else {
                    total_entities_removed += 1;
                }
            }
        }
    }

    // Clean up orphaned edges
    let all_edges = state.graph_storage.get_all_edges().await?;
    let remaining_nodes = state.graph_storage.get_all_nodes().await?;
    let remaining_node_ids: std::collections::HashSet<String> =
        remaining_nodes.iter().map(|n| n.id.clone()).collect();

    for edge in all_edges {
        let is_orphaned = !remaining_node_ids.contains(&edge.source)
            || !remaining_node_ids.contains(&edge.target);

        if is_orphaned {
            if let Err(e) = state
                .graph_storage
                .delete_edge(&edge.source, &edge.target)
                .await
            {
                tracing::warn!(
                    source = %edge.source,
                    target = %edge.target,
                    error = %e,
                    "Failed to delete orphaned edge"
                );
            } else {
                total_relationships_removed += 1;
            }
        }
    }

    // Clean up PDF documents table
    // WHY: PDF documents have their own table separate from KV storage
    // The duplicate detection uses checksum from pdf_documents table, so we must clear it
    #[allow(unused_mut)] // mut only used when postgres feature is enabled
    let mut total_pdfs_deleted = 0usize;
    #[cfg(feature = "postgres")]
    if let Some(ref pdf_storage) = state.pdf_storage {
        // List all PDFs (no workspace filter to ensure full cleanup)
        let filter = ListPdfFilter {
            workspace_id: None,
            processing_status: None,
            page: Some(1),
            page_size: Some(10000), // Large page size to get all
        };

        match pdf_storage.list_pdfs(filter).await {
            Ok(pdf_list) => {
                for pdf in pdf_list.items {
                    if let Err(e) = pdf_storage.delete_pdf(&pdf.pdf_id).await {
                        tracing::warn!(
                            pdf_id = %pdf.pdf_id,
                            error = %e,
                            "Failed to delete PDF document"
                        );
                    } else {
                        total_pdfs_deleted += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to list PDF documents for cleanup");
            }
        }
    }

    tracing::info!(
        deleted = deleted_count,
        skipped = skipped_count,
        chunks = total_chunks_deleted,
        entities = total_entities_removed,
        relationships = total_relationships_removed,
        pdfs = total_pdfs_deleted,
        "Bulk delete complete"
    );

    Ok(Json(DeleteAllDocumentsResponse {
        deleted_count,
        total_chunks_deleted,
        total_entities_removed,
        total_relationships_removed,
        total_pdfs_deleted,
        skipped_count,
        skipped_documents,
    }))
}

/// Analyze the impact of deleting a document before actually deleting it.
///
/// This endpoint allows users to preview what would be affected by a document deletion
/// without actually performing the deletion. This is useful for understanding the
/// cascade effects before committing to a destructive operation.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/deletion-impact",
    tag = "Documents",
    params(
        ("document_id" = String, Path, description = "Document ID to analyze")
    ),
    responses(
        (status = 200, description = "Deletion impact analysis", body = DeletionImpactResponse),
        (status = 404, description = "Document not found")
    )
)]
pub async fn analyze_deletion_impact(
    State(state): State<AppState>,
    axum::extract::Path(document_id): axum::extract::Path<String>,
) -> ApiResult<Json<DeletionImpactResponse>> {
    let keys = state.kv_storage.keys().await?;

    // Find chunks belonging to this document
    let chunk_prefix = format!("{}-chunk-", document_id);
    let chunk_ids: Vec<String> = keys
        .iter()
        .filter(|k| k.starts_with(&chunk_prefix))
        .cloned()
        .collect();

    // Also check for metadata and content keys
    let metadata_key = format!("{}-metadata", document_id);
    let content_key = format!("{}-content", document_id);
    let has_metadata = keys.contains(&metadata_key);
    let has_content = keys.contains(&content_key);

    // Document must have either chunks, metadata, or content
    if chunk_ids.is_empty() && !has_metadata && !has_content {
        return Err(ApiError::NotFound(format!(
            "Document {} not found",
            document_id
        )));
    }

    let chunks_to_delete = chunk_ids.len();
    let mut entities_to_remove = 0usize;
    let mut entities_to_update = 0usize;
    let mut relationships_to_remove = 0usize;
    let mut relationships_to_update = 0usize;

    // Analyze entities (read-only)
    let all_nodes = state.graph_storage.get_all_nodes().await?;
    for node in all_nodes {
        if let Some(source_id) = node.properties.get("source_id").and_then(|v| v.as_str()) {
            let sources: Vec<&str> = source_id.split('|').collect();
            let remaining = sources
                .iter()
                .filter(|s| !s.starts_with(&chunk_prefix) && !s.starts_with(&document_id))
                .count();

            if remaining == 0 {
                entities_to_remove += 1;
            } else if remaining < sources.len() {
                entities_to_update += 1;
            }
        }
    }

    // Analyze edges (read-only)
    let all_edges = state.graph_storage.get_all_edges().await?;
    for edge in all_edges {
        if let Some(source_id) = edge.properties.get("source_id").and_then(|v| v.as_str()) {
            let sources: Vec<&str> = source_id.split('|').collect();
            let remaining = sources
                .iter()
                .filter(|s| !s.starts_with(&chunk_prefix) && !s.starts_with(&document_id))
                .count();

            if remaining == 0 {
                relationships_to_remove += 1;
            } else if remaining < sources.len() {
                relationships_to_update += 1;
            }
        }
    }

    Ok(Json(DeletionImpactResponse {
        document_id,
        chunks_to_delete,
        entities_to_remove,
        entities_to_update,
        relationships_to_remove,
        relationships_to_update,
        preview_only: true,
    }))
}
