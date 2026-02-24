use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::error::ApiError;
use crate::handlers::isolation::doc_belongs_to_workspace;
use crate::state::AppState;
use crate::handlers::workspaces_types::*;

// ============================================================================
// SPEC-032: Rebuild Embeddings Endpoint
// ============================================================================

/// Rebuild workspace embeddings with a new model.
///
/// This endpoint clears all vector embeddings for a workspace and optionally
/// updates the embedding model configuration. Documents will need to be
/// re-processed to regenerate embeddings.
///
/// ## Use Cases
///
/// - Changing embedding model (e.g., OpenAI → Ollama)
/// - Upgrading to a better embedding model
/// - Fixing corrupted embeddings
/// - Resetting after provider issues
///
/// ## Implementation Notes
///
/// Current implementation is **synchronous** and clears vectors immediately.
/// Future versions will support async background re-embedding.
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/rebuild-embeddings",
    request_body = RebuildEmbeddingsRequest,
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Rebuild started", body = RebuildEmbeddingsResponse),
        (status = 404, description = "Workspace not found"),
        (status = 400, description = "Invalid request"),
    ),
    tags = ["workspaces"]
)]
pub async fn rebuild_embeddings(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<RebuildEmbeddingsRequest>,
) -> Result<Json<RebuildEmbeddingsResponse>, ApiError> {
    use tracing::info;

    // 1. Get the workspace
    let workspace = state
        .workspace_service
        .get_workspace(workspace_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Workspace {} not found", workspace_id)))?;

    // 2. Get workspace stats to count documents
    let stats = state
        .workspace_service
        .get_workspace_stats(workspace_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // 3. Determine new embedding config
    let new_model = request
        .embedding_model
        .clone()
        .unwrap_or_else(|| workspace.embedding_model.clone());
    let new_provider = request
        .embedding_provider
        .clone()
        .unwrap_or_else(|| workspace.embedding_provider.clone());

    // WHY: Auto-detect dimension from model config when model changes
    // If embedding_dimension is explicitly provided in the request, use it.
    // Otherwise, look up the correct dimension from the model's config.
    // This ensures dimension is always consistent with the selected model.
    let new_dimension = if let Some(dim) = request.embedding_dimension {
        dim
    } else if new_model != workspace.embedding_model || new_provider != workspace.embedding_provider
    {
        // Model is changing - look up the correct dimension for the new model
        state
            .models_config
            .get_model(&new_provider, &new_model)
            .map(|m| m.capabilities.embedding_dimension)
            .unwrap_or_else(|| {
                tracing::warn!(
                    provider = %new_provider,
                    model = %new_model,
                    "No embedding dimension found for model, using workspace default"
                );
                workspace.embedding_dimension
            })
    } else {
        // No model change, keep existing dimension
        workspace.embedding_dimension
    };

    // 4. Check if config is actually changing
    let config_changed = new_model != workspace.embedding_model
        || new_provider != workspace.embedding_provider
        || new_dimension != workspace.embedding_dimension;

    if !config_changed && !request.force {
        return Err(ApiError::BadRequest(
            "Embedding configuration unchanged. Use 'force: true' to rebuild anyway.".to_string(),
        ));
    }

    // REQ-25: Validate chunk size vs embedding model compatibility (CRITICAL INVARIANT)
    // Get the new embedding model's context length to ensure chunks will fit
    let model_context_length = state
        .models_config
        .get_model(&new_provider, &new_model)
        .map(|m| m.capabilities.context_length)
        .unwrap_or(8192); // Default to safe value if model not found

    // Default chunk size is 1200 tokens (from chunker config)
    const DEFAULT_CHUNK_SIZE_TOKENS: usize = 1200;

    if model_context_length > 0 && DEFAULT_CHUNK_SIZE_TOKENS > model_context_length {
        info!(
            workspace_id = %workspace_id,
            chunk_size = DEFAULT_CHUNK_SIZE_TOKENS,
            model_context_length = model_context_length,
            warning = "Default chunk size exceeds model's context length",
            "Chunk-embedding compatibility warning - some chunks may fail to embed"
        );
        // Log warning but allow the operation to proceed
        // Future: Could add a strict mode that blocks incompatible changes
    }

    info!(
        workspace_id = %workspace_id,
        old_model = %workspace.embedding_model,
        new_model = %new_model,
        old_dimension = workspace.embedding_dimension,
        new_dimension = new_dimension,
        document_count = stats.document_count,
        model_context_length = model_context_length,
        "Starting embedding rebuild"
    );

    // 5. Clear vector storage for this specific workspace only
    // Uses workspace-scoped clearing to avoid affecting other workspaces
    let vectors_cleared = state
        .vector_storage
        .clear_workspace(&workspace_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to clear workspace vectors: {}", e)))?;

    info!(
        workspace_id = %workspace_id,
        vectors_cleared = vectors_cleared,
        "Vector storage cleared"
    );

    // OODA-225: Evict cached workspace vector storage when dimension changes
    // WHY: The WorkspaceVectorRegistry caches vector storage instances keyed by workspace_id.
    // When embedding dimension changes (e.g., 768 → 1536), the cached instance still references
    // the old dimension. Without eviction, queries will fail with "different vector dimensions"
    // because the query embedding (new dimension) doesn't match stored vectors (old dimension).
    // Evicting forces recreation with the new dimension on next access.
    if config_changed {
        state.vector_registry.evict(&workspace_id).await;
        info!(
            workspace_id = %workspace_id,
            old_dimension = workspace.embedding_dimension,
            new_dimension = new_dimension,
            "Evicted workspace vector storage cache for dimension change"
        );
    }

    // 6. Update workspace embedding config if changed (SPEC-032)
    if config_changed {
        use edgequake_core::UpdateWorkspaceRequest;

        let update_request = UpdateWorkspaceRequest {
            embedding_model: Some(new_model.clone()),
            embedding_provider: Some(new_provider.clone()),
            embedding_dimension: Some(new_dimension),
            ..Default::default()
        };

        state
            .workspace_service
            .update_workspace(workspace_id, update_request)
            .await
            .map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to update workspace embedding config: {}",
                    e
                ))
            })?;

        info!(
            workspace_id = %workspace_id,
            embedding_model = %new_model,
            embedding_provider = %new_provider,
            embedding_dimension = new_dimension,
            "Workspace embedding configuration updated"
        );
    }

    // 7. Queue documents for re-embedding (SPEC-032 REQ-25)
    // SPEC-041: PDF documents are re-queued as PdfProcessing tasks to re-extract
    // from the original PDF using the workspace's current vision LLM, then rechunk
    // and re-embed with the new embedding model.
    // Text/Markdown documents fall back to stored content (TextInsert).
    let (documents_queued, chunks_to_process, track_id) = if stats.document_count > 0 {
        use chrono::Utc;
        use edgequake_tasks::{PdfProcessingData, Task, TaskType, TextInsertData};

        let track_id = format!(
            "rebuild_embed_{}_{}",
            Utc::now().format("%Y%m%d_%H%M%S"),
            &Uuid::new_v4().to_string()[..8]
        );

        // Get all document metadata for this workspace
        let all_keys: Vec<String> = state
            .kv_storage
            .keys()
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to list document keys: {}", e)))?;

        let mut documents_queued = 0;
        let mut total_chunks = 0usize;

        for key in all_keys.iter().filter(|k| k.ends_with("-metadata")) {
            if let Some(value) = state.kv_storage.get_by_id(key).await.ok().flatten() {
                if let Some(obj) = value.as_object() {
                    // Check if document belongs to this workspace.
                    // WHY: rebuild must be strictly workspace-scoped so that triggering
                    // a rebuild on workspace X never reprocesses documents from workspace Y.
                    // Legacy documents may store workspace_id = "default" (string literal)
                    // instead of a real UUID; treat those as belonging to the workspace
                    // whose slug is also "default".
                    let doc_workspace = obj
                        .get("workspace_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default");

                    if !doc_belongs_to_workspace(
                        doc_workspace,
                        &workspace_id.to_string(),
                        &workspace.slug,
                    ) {
                        continue;
                    }

                    let doc_id = match obj.get("id").and_then(|v| v.as_str()) {
                        Some(id) => id.to_string(),
                        None => continue,
                    };

                    // Extract chunk count for this document
                    let doc_chunk_count =
                        obj.get("chunk_count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

                    let doc_title = obj
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&doc_id)
                        .to_string();

                    // SPEC-041: Route by source type BEFORE reading stored content.
                    // PDF docs with valid pdf_id → re-process from original PDF bytes.
                    // All others → re-process from stored content (require content present).
                    let source_type = obj.get("source_type").and_then(|v| v.as_str());
                    let pdf_id_str = obj.get("pdf_id").and_then(|v| v.as_str());

                    // Determine if we can use PDF reprocessing path
                    let pdf_task_opt = if source_type == Some("pdf") {
                        pdf_id_str
                            .and_then(|pid| Uuid::parse_str(pid).ok())
                            .map(|pdf_id_uuid| {
                                let vision_provider = workspace
                                    .vision_llm_provider
                                    .as_deref()
                                    .filter(|p| !p.is_empty())
                                    .unwrap_or("ollama")
                                    .to_string();
                                let vision_model =
                                    workspace.vision_llm_model.clone().filter(|m| !m.is_empty());
                                PdfProcessingData {
                                    pdf_id: pdf_id_uuid,
                                    tenant_id: workspace.tenant_id,
                                    workspace_id,
                                    enable_vision: true,
                                    vision_provider,
                                    vision_model,
                                    // FIX-REBUILD: Pass existing document ID so the
                                    // processor updates the existing document in-place.
                                    existing_document_id: Some(doc_id.clone()),
                                }
                            })
                    } else {
                        None
                    };

                    // Update document status to pending
                    let metadata_key = format!("{}-metadata", doc_id);
                    if let Some(mut metadata) = state
                        .kv_storage
                        .get_by_id(&metadata_key)
                        .await
                        .ok()
                        .flatten()
                    {
                        if let Some(obj) = metadata.as_object_mut() {
                            obj.insert("status".to_string(), serde_json::json!("pending"));
                            obj.insert("track_id".to_string(), serde_json::json!(track_id));
                            obj.insert(
                                "reprocess_at".to_string(),
                                serde_json::json!(Utc::now().to_rfc3339()),
                            );
                            let _ = state.kv_storage.upsert(&[(metadata_key, metadata)]).await;
                        }
                    }

                    let (task_type, task_value) = if let Some(pdf_task) = pdf_task_opt {
                        // Re-extract from original PDF: new vision LLM + rechunk + re-embed.
                        (
                            TaskType::PdfProcessing,
                            serde_json::to_value(&pdf_task).unwrap(),
                        )
                    } else {
                        // Fallback: text/markdown or PDF without stored pdf_id.
                        // Read stored content — skip doc if missing.
                        let content_key = format!("{}-content", doc_id);
                        let content = match state.kv_storage.get_by_id(&content_key).await {
                            Ok(Some(cv)) => cv
                                .get("content")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            _ => None,
                        };
                        let content = match content {
                            Some(c) => c,
                            None => continue,
                        };
                        let text_task = TextInsertData {
                            text: content,
                            file_source: doc_title.clone(),
                            workspace_id: workspace_id.to_string(),
                            metadata: Some(serde_json::json!({
                                "document_id": doc_id,
                                "title": doc_title,
                                "track_id": track_id,
                                "is_reprocess": true,
                                "is_embedding_rebuild": true,
                                "workspace_id": workspace_id.to_string(),
                                "tenant_id": workspace.tenant_id.to_string(),
                            })),
                        };
                        (TaskType::Insert, serde_json::to_value(&text_task).unwrap())
                    };

                    let task = Task::new(workspace.tenant_id, workspace_id, task_type, task_value);

                    // Store and queue task
                    if state.task_storage.create_task(&task).await.is_ok()
                        && state.task_queue.send(task).await.is_ok()
                    {
                        documents_queued += 1;
                        total_chunks += doc_chunk_count;
                    }
                }
            }
        }

        info!(
            workspace_id = %workspace_id,
            track_id = %track_id,
            documents_queued = documents_queued,
            total_chunks = total_chunks,
            "Documents queued for re-embedding"
        );

        (documents_queued, total_chunks, Some(track_id))
    } else {
        (0, 0, None)
    };

    // 8. Build response
    // Estimate: ~1 second per document for embedding (conservative)
    let estimated_time = if stats.document_count > 0 {
        Some(stats.document_count as u64)
    } else {
        None
    };

    // REQ-25: Generate compatibility warning if chunks may exceed model limit
    let compatibility_warning = if model_context_length > 0
        && DEFAULT_CHUNK_SIZE_TOKENS > model_context_length
    {
        Some(format!(
            "Default chunk size ({} tokens) exceeds model's context length ({} tokens). Some chunks may fail to embed.",
            DEFAULT_CHUNK_SIZE_TOKENS, model_context_length
        ))
    } else {
        None
    };
    let has_compatibility_warning = compatibility_warning.is_some();

    // Determine status based on whether documents were queued
    let status = if documents_queued > 0 {
        "processing".to_string()
    } else if vectors_cleared > 0 {
        "vectors_cleared".to_string()
    } else {
        "no_change".to_string()
    };

    let response = RebuildEmbeddingsResponse {
        workspace_id,
        status,
        documents_to_process: documents_queued,
        chunks_to_process,
        vectors_cleared,
        embedding_model: new_model.clone(),
        embedding_provider: new_provider.clone(),
        embedding_dimension: new_dimension,
        model_context_length,
        estimated_time_seconds: estimated_time,
        job_id: track_id.clone(),
        compatibility_warning,
    };

    info!(
        workspace_id = %workspace_id,
        status = %response.status,
        documents_queued = documents_queued,
        chunks_to_process = chunks_to_process,
        vectors_cleared = vectors_cleared,
        embedding_model = %new_model,
        embedding_provider = %new_provider,
        model_context_length = model_context_length,
        has_warning = has_compatibility_warning,
        track_id = ?track_id,
        "Embedding rebuild complete - documents queued for re-embedding"
    );

    Ok(Json(response))
}

// ============================================================================
// Rebuild Knowledge Graph Endpoint (LLM Model Change)
// ============================================================================

/// Rebuild knowledge graph for a workspace after LLM model change.
///
/// This operation:
/// 1. Clears all entities and relationships from the graph storage
/// 2. Optionally clears vector embeddings (default: yes)
/// 3. Queues all documents for reprocessing with the new LLM model
///
/// Use this when:
/// - Changing the extraction/LLM model (e.g., gpt-4o-mini → gemma3:12b)
/// - Upgrading to a new LLM version with better entity extraction
/// - Migrating between LLM providers
///
/// ## WARNING
///
/// This is a destructive operation. All existing knowledge graph data
/// (entities, relationships) will be deleted. The workspace will be empty
/// until document reprocessing is complete.
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph",
    request_body = RebuildKnowledgeGraphRequest,
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Knowledge graph rebuild started", body = RebuildKnowledgeGraphResponse),
        (status = 404, description = "Workspace not found"),
        (status = 400, description = "Invalid request"),
    ),
    tags = ["workspaces"]
)]
pub async fn rebuild_knowledge_graph(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<RebuildKnowledgeGraphRequest>,
) -> Result<Json<RebuildKnowledgeGraphResponse>, ApiError> {
    use chrono::Utc;
    use tracing::info;

    // 1. Get the workspace
    let workspace = state
        .workspace_service
        .get_workspace(workspace_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Workspace {} not found", workspace_id)))?;

    // 2. Get workspace stats
    let stats = state
        .workspace_service
        .get_workspace_stats(workspace_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // 3. Determine new LLM config
    let new_llm_model = request
        .llm_model
        .clone()
        .unwrap_or_else(|| workspace.llm_model.clone());
    let new_llm_provider = request
        .llm_provider
        .clone()
        .unwrap_or_else(|| workspace.llm_provider.clone());

    // 4. Check if config is actually changing
    let config_changed =
        new_llm_model != workspace.llm_model || new_llm_provider != workspace.llm_provider;

    if !config_changed && !request.force {
        return Err(ApiError::BadRequest(
            "LLM configuration unchanged. Use 'force: true' to rebuild anyway.".to_string(),
        ));
    }

    info!(
        workspace_id = %workspace_id,
        old_model = %workspace.llm_model,
        new_model = %new_llm_model,
        old_provider = %workspace.llm_provider,
        new_provider = %new_llm_provider,
        document_count = stats.document_count,
        rebuild_embeddings = request.rebuild_embeddings,
        "Starting knowledge graph rebuild"
    );

    // 5. Clear graph storage (workspace-scoped)
    let (nodes_cleared, edges_cleared) =
        state
            .graph_storage
            .clear_workspace(&workspace_id)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to clear graph: {}", e)))?;

    info!(
        workspace_id = %workspace_id,
        nodes_cleared = nodes_cleared,
        edges_cleared = edges_cleared,
        "Graph storage cleared"
    );

    // 6. Optionally clear vectors (if also changing embeddings)
    let vectors_cleared = if request.rebuild_embeddings {
        let count = state
            .vector_storage
            .clear_workspace(&workspace_id)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to clear vectors: {}", e)))?;

        // OODA-225: Evict cached workspace vector storage when clearing vectors
        // WHY: If rebuild_embeddings is requested, the embedding model/dimension may change.
        // The cached vector storage instance holds the old dimension configuration.
        // Evicting forces recreation with correct dimension on next access.
        state.vector_registry.evict(&workspace_id).await;

        info!(
            workspace_id = %workspace_id,
            vectors_cleared = count,
            "Vector storage cleared and cache evicted"
        );
        count
    } else {
        0
    };

    // 7. Generate track ID for reprocessing batch
    let track_id = format!(
        "rebuild_kg_{}_{}",
        Utc::now().format("%Y%m%d_%H%M%S"),
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    // 8. Update workspace LLM config if changed (SPEC-032)
    if config_changed {
        use edgequake_core::UpdateWorkspaceRequest;

        let update_request = UpdateWorkspaceRequest {
            llm_model: Some(new_llm_model.clone()),
            llm_provider: Some(new_llm_provider.clone()),
            ..Default::default()
        };

        state
            .workspace_service
            .update_workspace(workspace_id, update_request)
            .await
            .map_err(|e| {
                ApiError::Internal(format!("Failed to update workspace LLM config: {}", e))
            })?;

        info!(
            workspace_id = %workspace_id,
            llm_model = %new_llm_model,
            llm_provider = %new_llm_provider,
            "Workspace LLM configuration updated"
        );
    }

    // 9. Queue all documents for reprocessing (SPEC-032 REQ-24)
    // SPEC-041: PDF documents are re-queued as PdfProcessing tasks so the full
    // pipeline runs from the original PDF bytes: vision extraction → chunking →
    // embedding → entity extraction.  Only text/markdown documents fall back to
    // the stored content (TextInsert).
    let (documents_queued, chunks_to_process) = if stats.document_count > 0 {
        use edgequake_tasks::{PdfProcessingData, Task, TaskType, TextInsertData};

        // Get all document metadata for this workspace
        let all_keys: Vec<String> = state
            .kv_storage
            .keys()
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to list document keys: {}", e)))?;

        let mut documents_queued = 0;
        let mut total_chunks = 0usize;

        for key in all_keys.iter().filter(|k| k.ends_with("-metadata")) {
            if let Some(value) = state.kv_storage.get_by_id(key).await.ok().flatten() {
                if let Some(obj) = value.as_object() {
                    // Check if document belongs to this workspace.
                    // WHY: rebuild must be strictly workspace-scoped so that triggering
                    // a rebuild on workspace X never reprocesses documents from workspace Y.
                    // Legacy documents may store workspace_id = "default" (string literal)
                    // instead of a real UUID; treat those as belonging to the workspace
                    // whose slug is also "default".
                    let doc_workspace = obj
                        .get("workspace_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default");

                    if !doc_belongs_to_workspace(
                        doc_workspace,
                        &workspace_id.to_string(),
                        &workspace.slug,
                    ) {
                        continue;
                    }

                    let doc_id = match obj.get("id").and_then(|v| v.as_str()) {
                        Some(id) => id.to_string(),
                        None => continue,
                    };

                    // Extract chunk count for this document
                    let doc_chunk_count =
                        obj.get("chunk_count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

                    let doc_title = obj
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&doc_id)
                        .to_string();

                    // Update document status to pending before queuing
                    let metadata_key = format!("{}-metadata", doc_id);
                    if let Some(mut metadata) = state
                        .kv_storage
                        .get_by_id(&metadata_key)
                        .await
                        .ok()
                        .flatten()
                    {
                        if let Some(obj) = metadata.as_object_mut() {
                            obj.insert("status".to_string(), serde_json::json!("pending"));
                            obj.insert("track_id".to_string(), serde_json::json!(track_id));
                            obj.insert(
                                "reprocess_at".to_string(),
                                serde_json::json!(Utc::now().to_rfc3339()),
                            );
                            let _ = state.kv_storage.upsert(&[(metadata_key, metadata)]).await;
                        }
                    }

                    // SPEC-041: Route by source type.
                    // PDF → re-extract from original bytes using workspace vision LLM.
                    // Text/Markdown → re-process from stored content.
                    let source_type = obj.get("source_type").and_then(|v| v.as_str());
                    let pdf_id_str = obj.get("pdf_id").and_then(|v| v.as_str());

                    let (task_type, task_value) = if source_type == Some("pdf") {
                        if let Some(pid_str) = pdf_id_str {
                            if let Ok(pdf_id_uuid) = Uuid::parse_str(pid_str) {
                                // WHY: Re-process from original PDF so the new vision LLM,
                                // LLM, and embedding model all apply to this document.
                                // vision_provider/model come from the workspace config and
                                // will override any previously used model.
                                let vision_provider = workspace
                                    .vision_llm_provider
                                    .as_deref()
                                    .filter(|p| !p.is_empty())
                                    .unwrap_or("ollama")
                                    .to_string();
                                let vision_model =
                                    workspace.vision_llm_model.clone().filter(|m| !m.is_empty());

                                let pdf_task = PdfProcessingData {
                                    pdf_id: pdf_id_uuid,
                                    tenant_id: workspace.tenant_id,
                                    workspace_id,
                                    enable_vision: true,
                                    vision_provider,
                                    vision_model,
                                    // FIX-REBUILD: Pass existing document ID so the
                                    // processor updates the existing document in-place
                                    // instead of creating an orphaned duplicate.
                                    existing_document_id: Some(doc_id.clone()),
                                };
                                (
                                    TaskType::PdfProcessing,
                                    serde_json::to_value(&pdf_task).unwrap(),
                                )
                            } else {
                                // Malformed pdf_id — fall back to stored content
                                tracing::warn!(doc_id = %doc_id, pdf_id = %pid_str, "Malformed pdf_id, falling back to text reprocess");
                                let content_key = format!("{}-content", doc_id);
                                let content = match state.kv_storage.get_by_id(&content_key).await {
                                    Ok(Some(cv)) => cv
                                        .get("content")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    _ => None,
                                };
                                match content {
                                    Some(c) => {
                                        let text_task = TextInsertData {
                                            text: c,
                                            file_source: doc_title.clone(),
                                            workspace_id: workspace_id.to_string(),
                                            metadata: Some(
                                                serde_json::json!({ "document_id": doc_id, "title": doc_title, "track_id": track_id, "is_reprocess": true, "is_kg_rebuild": true, "workspace_id": workspace_id.to_string(), "tenant_id": workspace.tenant_id.to_string() }),
                                            ),
                                        };
                                        (
                                            TaskType::Insert,
                                            serde_json::to_value(&text_task).unwrap(),
                                        )
                                    }
                                    None => continue,
                                }
                            }
                        } else {
                            // PDF doc without pdf_id stored yet — fall back to text
                            let content_key = format!("{}-content", doc_id);
                            let content = match state.kv_storage.get_by_id(&content_key).await {
                                Ok(Some(cv)) => cv
                                    .get("content")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                _ => None,
                            };
                            match content {
                                Some(c) => {
                                    let text_task = TextInsertData {
                                        text: c,
                                        file_source: doc_title.clone(),
                                        workspace_id: workspace_id.to_string(),
                                        metadata: Some(
                                            serde_json::json!({ "document_id": doc_id, "title": doc_title, "track_id": track_id, "is_reprocess": true, "is_kg_rebuild": true, "workspace_id": workspace_id.to_string(), "tenant_id": workspace.tenant_id.to_string() }),
                                        ),
                                    };
                                    (TaskType::Insert, serde_json::to_value(&text_task).unwrap())
                                }
                                None => continue,
                            }
                        }
                    } else {
                        // Text/Markdown document — re-process from stored content.
                        let content_key = format!("{}-content", doc_id);
                        let content = match state.kv_storage.get_by_id(&content_key).await {
                            Ok(Some(content_value)) => content_value
                                .get("content")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            _ => None,
                        };
                        let content = match content {
                            Some(c) => c,
                            None => continue,
                        };
                        let text_task = TextInsertData {
                            text: content,
                            file_source: doc_title.clone(),
                            workspace_id: workspace_id.to_string(),
                            metadata: Some(serde_json::json!({
                                "document_id": doc_id,
                                "title": doc_title,
                                "track_id": track_id,
                                "is_reprocess": true,
                                "is_kg_rebuild": true,
                                "workspace_id": workspace_id.to_string(),
                                "tenant_id": workspace.tenant_id.to_string(),
                            })),
                        };
                        (TaskType::Insert, serde_json::to_value(&text_task).unwrap())
                    };

                    let task = Task::new(workspace.tenant_id, workspace_id, task_type, task_value);

                    // Store and queue task
                    if state.task_storage.create_task(&task).await.is_ok()
                        && state.task_queue.send(task).await.is_ok()
                    {
                        documents_queued += 1;
                        total_chunks += doc_chunk_count;
                    }
                }
            }
        }

        info!(
            workspace_id = %workspace_id,
            track_id = %track_id,
            documents_queued = documents_queued,
            total_chunks = total_chunks,
            "Documents queued for knowledge graph rebuild"
        );

        (documents_queued, total_chunks)
    } else {
        (0, 0)
    };

    // 10. Build response
    let estimated_time = if stats.document_count > 0 {
        // Estimate: ~2 seconds per document (extraction + embedding)
        Some(stats.document_count as u64 * 2)
    } else {
        None
    };

    // Determine status based on whether documents were queued
    let status = if documents_queued > 0 {
        "processing".to_string()
    } else if nodes_cleared > 0 || edges_cleared > 0 {
        "graph_cleared".to_string()
    } else {
        "no_change".to_string()
    };

    let response = RebuildKnowledgeGraphResponse {
        workspace_id,
        status,
        nodes_cleared,
        edges_cleared,
        vectors_cleared,
        documents_to_process: documents_queued,
        chunks_to_process,
        llm_model: new_llm_model.clone(),
        llm_provider: new_llm_provider.clone(),
        estimated_time_seconds: estimated_time,
        track_id: Some(track_id.clone()),
    };

    info!(
        workspace_id = %workspace_id,
        status = %response.status,
        nodes = nodes_cleared,
        edges = edges_cleared,
        vectors = vectors_cleared,
        documents_queued = documents_queued,
        chunks_to_process = chunks_to_process,
        llm_model = %new_llm_model,
        llm_provider = %new_llm_provider,
        track_id = %track_id,
        "Knowledge graph rebuild complete - documents queued for reprocessing"
    );

    Ok(Json(response))
}

// SPEC-032: Reprocess All Documents Endpoint
// Focus Area 5 - Trigger document reprocessing after rebuild

/// Reprocess all documents in a workspace.
///
/// This endpoint queues all documents for re-embedding, typically used after
/// a rebuild-embeddings operation to regenerate vector embeddings. Progress
/// can be monitored via the pipeline status endpoint.
///
/// ## Use Cases
///
/// - Regenerate embeddings after model change
/// - Re-extract entities after LLM update
/// - Bulk re-processing for quality improvements
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/reprocess-documents",
    request_body = ReprocessAllRequest,
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Documents queued for reprocessing", body = ReprocessAllResponse),
        (status = 404, description = "Workspace not found"),
        (status = 400, description = "Invalid request"),
    ),
    tags = ["workspaces"]
)]
pub async fn reprocess_all_documents(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<ReprocessAllRequest>,
) -> Result<Json<ReprocessAllResponse>, ApiError> {
    use chrono::Utc;
    use edgequake_tasks::{PdfProcessingData, Task, TaskType, TextInsertData};
    use tracing::info;

    // 1. Verify workspace exists
    let workspace = state
        .workspace_service
        .get_workspace(workspace_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Workspace {} not found", workspace_id)))?;

    // 2. Generate track ID for this batch
    let track_id = format!(
        "reprocess_{}_{}",
        Utc::now().format("%Y%m%d_%H%M%S"),
        &Uuid::new_v4().to_string()[..8]
    );

    info!(
        workspace_id = %workspace_id,
        track_id = %track_id,
        include_completed = request.include_completed,
        "Starting reprocess all documents"
    );

    // 3. Get all document metadata for this workspace
    let all_keys: Vec<String> = state
        .kv_storage
        .keys()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to list document keys: {}", e)))?;

    // REQ-24: Debug logging for document discovery
    let metadata_keys_count = all_keys.iter().filter(|k| k.ends_with("-metadata")).count();
    info!(
        workspace_id = %workspace_id,
        total_keys = all_keys.len(),
        metadata_keys = metadata_keys_count,
        "Scanning KV storage for documents to reprocess"
    );

    let mut documents_found = 0;
    let mut documents_queued = 0;
    let mut documents_skipped = 0;
    let mut skip_reasons: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    // 4. Process each document
    for key in all_keys.iter().filter(|k| k.ends_with("-metadata")) {
        if documents_queued >= request.max_documents {
            *skip_reasons.entry("max_documents_reached").or_insert(0) += 1;
            break;
        }

        if let Some(value) =
            state.kv_storage.get_by_id(key).await.map_err(|e| {
                ApiError::Internal(format!("Failed to get document metadata: {}", e))
            })?
        {
            if let Some(obj) = value.as_object() {
                // Check if document belongs to this workspace.
                // WHY: reprocess must be strictly workspace-scoped so that triggering
                // a reprocess on workspace X never reprocesses documents from workspace Y.
                // Legacy documents may store workspace_id = "default" (string literal)
                // instead of a real UUID; treat those as belonging to the workspace
                // whose slug is also "default".
                let doc_workspace = obj
                    .get("workspace_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                if !doc_belongs_to_workspace(
                    doc_workspace,
                    &workspace_id.to_string(),
                    &workspace.slug,
                ) {
                    *skip_reasons.entry("wrong_workspace").or_insert(0) += 1;
                    continue;
                }

                documents_found += 1;

                let status = obj.get("status").and_then(|v| v.as_str());
                let doc_id = obj.get("id").and_then(|v| v.as_str());
                let title = obj.get("title").and_then(|v| v.as_str());

                // Skip if not including completed and already completed
                if !request.include_completed && status == Some("completed") {
                    documents_skipped += 1;
                    *skip_reasons.entry("completed_excluded").or_insert(0) += 1;
                    continue;
                }

                // Skip if currently processing
                if status == Some("processing") {
                    documents_skipped += 1;
                    *skip_reasons.entry("already_processing").or_insert(0) += 1;
                    continue;
                }

                // Get document ID
                let doc_id = match doc_id {
                    Some(id) => id.to_string(),
                    None => {
                        documents_skipped += 1;
                        *skip_reasons.entry("no_doc_id").or_insert(0) += 1;
                        continue;
                    }
                };

                // Get document content — deferred for PDFs (may not need it)
                let source_type = obj.get("source_type").and_then(|v| v.as_str());
                let pdf_id_str = obj.get("pdf_id").and_then(|v| v.as_str());
                let doc_title = title.unwrap_or(&doc_id).to_string();

                // Determine PDF reprocessing task (no content needed for valid PDFs)
                let pdf_task_opt = if source_type == Some("pdf") {
                    pdf_id_str
                        .and_then(|pid| Uuid::parse_str(pid).ok())
                        .map(|pdf_id_uuid| {
                            let vision_provider = workspace
                                .vision_llm_provider
                                .as_deref()
                                .filter(|p| !p.is_empty())
                                .unwrap_or("ollama")
                                .to_string();
                            let vision_model =
                                workspace.vision_llm_model.clone().filter(|m| !m.is_empty());
                            PdfProcessingData {
                                pdf_id: pdf_id_uuid,
                                tenant_id: workspace.tenant_id,
                                workspace_id,
                                enable_vision: true,
                                vision_provider,
                                vision_model,
                                // FIX-REBUILD: Pass existing document ID so the
                                // processor updates the existing document in-place.
                                existing_document_id: Some(doc_id.clone()),
                            }
                        })
                } else {
                    None
                };

                // For non-PDF fallback paths: read and require stored content
                let text_content_opt = if pdf_task_opt.is_none() {
                    let content_key = format!("{}-content", doc_id);
                    match state.kv_storage.get_by_id(&content_key).await {
                        Ok(Some(cv)) => cv
                            .get("content")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        _ => None,
                    }
                } else {
                    None // Not needed for PDF reprocessing
                };

                // Skip text documents with no stored content
                if pdf_task_opt.is_none() && text_content_opt.is_none() {
                    documents_skipped += 1;
                    *skip_reasons.entry("no_content").or_insert(0) += 1;
                    continue;
                }

                // Update document status to pending
                let metadata_key = format!("{}-metadata", doc_id);
                if let Some(mut metadata) = state
                    .kv_storage
                    .get_by_id(&metadata_key)
                    .await
                    .ok()
                    .flatten()
                {
                    if let Some(obj) = metadata.as_object_mut() {
                        obj.insert("status".to_string(), serde_json::json!("pending"));
                        obj.insert("track_id".to_string(), serde_json::json!(track_id));
                        obj.insert(
                            "reprocess_at".to_string(),
                            serde_json::json!(Utc::now().to_rfc3339()),
                        );

                        let _ = state.kv_storage.upsert(&[(metadata_key, metadata)]).await;
                    }
                }

                // SPEC-041: Route by source type.
                // PDF → re-extract from original PDF using workspace's current vision LLM.
                // Text/Markdown → re-process from stored content.
                let (task_type, task_value) = if let Some(pdf_task) = pdf_task_opt {
                    // Re-extract from original PDF.
                    (
                        TaskType::PdfProcessing,
                        serde_json::to_value(&pdf_task).unwrap(),
                    )
                } else {
                    // Text/Markdown — re-process from stored content.
                    let content = text_content_opt.unwrap_or_default();
                    (
                        TaskType::Insert,
                        serde_json::to_value(&TextInsertData {
                            text: content,
                            file_source: doc_title.clone(),
                            workspace_id: workspace_id.to_string(),
                            metadata: Some(serde_json::json!({
                                "document_id": doc_id,
                                "title": doc_title,
                                "track_id": track_id,
                                "is_reprocess": true,
                                "workspace_id": workspace_id.to_string(),
                                "tenant_id": workspace.tenant_id.to_string(),
                            })),
                        })
                        .unwrap(),
                    )
                };

                let task = Task::new(workspace.tenant_id, workspace_id, task_type, task_value);

                // Store and queue task
                if let Err(e) = state.task_storage.create_task(&task).await {
                    info!(error = %e, doc_id = %doc_id, "Failed to create task, skipping");
                    documents_skipped += 1;
                    *skip_reasons.entry("task_create_failed").or_insert(0) += 1;
                    continue;
                }

                if let Err(e) = state.task_queue.send(task).await {
                    info!(error = %e, doc_id = %doc_id, "Failed to queue task, skipping");
                    documents_skipped += 1;
                    *skip_reasons.entry("task_queue_failed").or_insert(0) += 1;
                    continue;
                }

                documents_queued += 1;
            }
        }
    }

    // REQ-24: Log detailed skip reasons for debugging
    if !skip_reasons.is_empty() {
        info!(
            workspace_id = %workspace_id,
            skip_reasons = ?skip_reasons,
            "Document skip reasons breakdown"
        );
    }

    // 5. Estimate processing time (1 second per document conservative)
    let estimated_time = if documents_queued > 0 {
        Some(documents_queued as u64)
    } else {
        None
    };

    let response = ReprocessAllResponse {
        track_id,
        workspace_id,
        status: if documents_queued > 0 {
            "processing".to_string()
        } else {
            "no_documents".to_string()
        },
        documents_found,
        documents_queued,
        documents_skipped,
        estimated_time_seconds: estimated_time,
    };

    info!(
        workspace_id = %workspace_id,
        found = documents_found,
        queued = documents_queued,
        skipped = documents_skipped,
        "Reprocess all documents complete"
    );

    Ok(Json(response))
}