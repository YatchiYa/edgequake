use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::Multipart;
use chrono::Utc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::file_validation::validate_file;
use crate::middleware::TenantContext;
use crate::services::ContentHasher;
use crate::state::AppState;
use edgequake_core::MetricsTriggerType;
#[cfg(feature = "postgres")]
use edgequake_storage::ListPdfFilter;

use crate::handlers::documents_types::*;
use super::storage_helpers::{delete_document_for_reingestion, get_workspace_vector_storage_strict};
#[allow(unused_imports)]
use super::storage_helpers::get_workspace_vector_storage_with_fallback;

/// Upload a document for processing.
///
/// # Implements
///
/// - **UC0001**: Upload Document
/// - **FEAT0001**: Document Ingestion Pipeline
/// - **FEAT0002**: Entity Extraction
/// - **FEAT0003**: Relationship Discovery
///
/// # Enforces
///
/// - **BR0001**: Content uniqueness (SHA-256 hash computed)
/// - **BR0201**: Tenant isolation (scoped to workspace)
/// - **BR0302**: Document size limits enforced
///
/// # Request Flow
///
/// ```text
/// POST /api/v1/documents
///        ↓
///   Validate content size
///        ↓
///   Compute SHA-256 hash
///        ↓
///   Store metadata + content
///        ↓
///   async_processing?
///     ├─ true: Create task → Return task_id
///     └─ false: Process inline → Return entities
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/documents",
    tag = "Documents",
    request_body = UploadDocumentRequest,
    responses(
        (status = 201, description = "Document uploaded successfully", body = UploadDocumentResponse),
        (status = 400, description = "Invalid request"),
        (status = 413, description = "Document too large")
    )
)]
pub async fn upload_document(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Json(request): Json<UploadDocumentRequest>,
) -> ApiResult<(StatusCode, Json<UploadDocumentResponse>)> {
    debug!(
        tenant_id = ?tenant_ctx.tenant_id,
        workspace_id = ?tenant_ctx.workspace_id,
        "Uploading document with tenant context"
    );

    // Validate document content
    crate::validation::validate_content(&request.content, state.config.max_document_size)?;

    // Generate or use provided track_id
    let track_id = request.track_id.unwrap_or_else(|| {
        format!(
            "upload_{}_{}",
            Utc::now().format("%Y%m%d_%H%M%S"),
            &Uuid::new_v4().to_string()[..8]
        )
    });

    // WHY-OODA83: Use ContentHasher service for consistent hash computation (DRY)
    let content_hash = ContentHasher::hash_str(&request.content);

    // Extract tenant context for storage (needed for hash_key)
    let workspace_id_for_storage = tenant_ctx
        .workspace_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let tenant_id_for_storage = tenant_ctx.tenant_id.clone();

    // WHY-OODA81+84: Workspace-scoped duplicate detection
    // FIX-4: Duplicates now trigger re-ingestion instead of rejection
    // Same content in same workspace = re-ingest (delete old data, process new)
    // Same content in different workspace = allowed (multi-tenancy)
    let hash_key = ContentHasher::workspace_hash_key(&workspace_id_for_storage, &content_hash);
    debug!(hash_key = %hash_key, workspace_id = %workspace_id_for_storage, "Checking for workspace-scoped duplicate hash");
    if let Some(existing_doc_id) = state.kv_storage.get_by_id(&hash_key).await? {
        debug!(existing_doc_id = ?existing_doc_id, "Found existing document for hash in workspace");
        if let Some(doc_id_str) = existing_doc_id.as_str() {
            // FIX-4: Try to delete old document data for re-ingestion
            match delete_document_for_reingestion(doc_id_str, &state, &workspace_id_for_storage)
                .await
            {
                Ok(true) => {
                    // Successfully deleted - proceed with new upload
                    tracing::info!(
                        old_doc_id = %doc_id_str,
                        workspace_id = %workspace_id_for_storage,
                        "Duplicate document found - old data deleted, proceeding with re-ingestion"
                    );
                    // Hash key will be updated below with new document_id
                }
                Ok(false) => {
                    // Document still processing - return duplicate response
                    tracing::warn!(
                        old_doc_id = %doc_id_str,
                        "Duplicate document is still being processed - cannot re-ingest"
                    );
                    return Ok((
                        StatusCode::OK,
                        Json(UploadDocumentResponse {
                            document_id: doc_id_str.to_string(),
                            status: "duplicate_processing".to_string(),
                            task_id: None,
                            track_id: track_id.clone(),
                            duplicate_of: Some(doc_id_str.to_string()),
                            chunk_count: None,
                            entity_count: None,
                            relationship_count: None,
                            cost: None,
                        }),
                    ));
                }
                Err(e) => {
                    // Failed to delete - log error and proceed with re-ingestion anyway
                    tracing::warn!(
                        old_doc_id = %doc_id_str,
                        error = %e,
                        "Failed to delete old document data - proceeding with re-ingestion"
                    );
                }
            }
        }
    }

    // Generate document ID
    let document_id = Uuid::new_v4().to_string();

    // Store hash mapping for deduplication (workspace-scoped)
    // WHY-OODA81+84: Must store before creating document to prevent race conditions
    state
        .kv_storage
        .upsert(&[(hash_key.clone(), serde_json::json!(document_id))])
        .await?;
    debug!(hash_key = %hash_key, document_id = %document_id, "Stored workspace-scoped hash mapping");

    // Generate content summary
    let content_summary = crate::validation::generate_content_summary(&request.content);
    let content_length = request.content.len();

    // Store document metadata (including title, content_summary, content_length, track_id, tenant context)
    let doc_metadata_key = format!("{}-metadata", document_id);
    let initial_status = if request.async_processing {
        "pending"
    } else {
        "processing"
    };

    // OODA-04: Include file_size_bytes, sha256_checksum, document_type for unified lineage
    // WHY: Every document—markdown or PDF—must carry the same lineage fields so
    // API consumers get consistent metadata regardless of source type.
    let doc_metadata = serde_json::json!({
        "id": document_id,
        "title": request.title,
        "content_summary": content_summary,
        "content_length": content_length,
        "content_hash": content_hash,
        "file_size_bytes": content_length,
        "sha256_checksum": content_hash,
        "document_type": "markdown",
        "track_id": track_id,
        "created_at": Utc::now().to_rfc3339(),
        "status": initial_status,
        "tenant_id": tenant_id_for_storage,
        "workspace_id": workspace_id_for_storage,
        // SPEC-002: Unified Ingestion Pipeline fields
        "source_type": "markdown",
        "current_stage": "uploading",
        "stage_progress": 0.0,
        "stage_message": "Document received, starting processing",
    });
    state
        .kv_storage
        .upsert(&[(doc_metadata_key.clone(), doc_metadata)])
        .await?;

    // Store the document content for processing
    let doc_content_key = format!("{}-content", document_id);
    let doc_content = serde_json::json!({
        "content": request.content,
    });
    state
        .kv_storage
        .upsert(&[(doc_content_key, doc_content)])
        .await?;

    // Handle async vs sync processing
    if request.async_processing {
        // Create task for background processing
        use edgequake_tasks::{Task, TaskType, TextInsertData};

        // Use tenant context for workspace_id, fallback to "default"
        let workspace_id = tenant_ctx
            .workspace_id
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let tenant_id = tenant_ctx
            .tenant_id
            .clone()
            .unwrap_or_else(|| "default".to_string());

        let task_data = TextInsertData {
            text: request.content.clone(),
            file_source: request.title.clone().unwrap_or_else(|| document_id.clone()),
            workspace_id: workspace_id.clone(),
            metadata: Some(serde_json::json!({
                "document_id": document_id,
                "title": request.title,
                "tenant_id": tenant_id,
                "workspace_id": workspace_id,
            })),
        };

        let task = Task::new(
            uuid::Uuid::parse_str(&tenant_id)
                .map_err(|_| ApiError::ValidationError("Invalid tenant ID".to_string()))?,
            uuid::Uuid::parse_str(&workspace_id)
                .map_err(|_| ApiError::ValidationError("Invalid workspace ID".to_string()))?,
            TaskType::Insert,
            serde_json::to_value(task_data).unwrap(),
        );
        let task_id = task.track_id.clone();

        // Store task
        state
            .task_storage
            .create_task(&task)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to create task: {}", e)))?;

        // Queue task for processing
        state
            .task_queue
            .send(task)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to queue task: {}", e)))?;

        Ok((
            StatusCode::CREATED,
            Json(UploadDocumentResponse {
                document_id,
                status: "pending".to_string(),
                task_id: Some(task_id),
                track_id,
                duplicate_of: None,
                chunk_count: None,
                entity_count: None,
                relationship_count: None,
                cost: None, // Cost will be calculated when processing completes
            }),
        ))
    } else {
        // Synchronous processing (original behavior)
        // Broadcast job started
        let start_time = std::time::Instant::now();
        state.progress_broadcaster.job_started(&document_id, 1, 1);

        // SPEC-032: Use workspace-specific pipeline with workspace LLM configuration
        // This ensures the workspace's LLM model is used for entity extraction
        let workspace_pipeline = state
            .create_workspace_pipeline(&workspace_id_for_storage)
            .await;

        // OODA-01: Add HTTP-level timeout to prevent indefinite hangs
        // WHY: Large documents (100KB+) can take 5-10 minutes to process,
        // but HTTP clients expect responses within 60-120 seconds.
        // Without this timeout, requests hang indefinitely causing poor UX.
        //
        // Timeout Strategy:
        // - 120 seconds (2 minutes): Conservative limit for synchronous mode
        // - For larger documents, users should use async_processing: true
        // - Timeout applies to ENTIRE pipeline, not just individual LLM calls
        //
        // See: specs/002-bullet-proof-ingestion-process.md
        const SYNC_PROCESSING_TIMEOUT_SECS: u64 = 120;

        let processing_start = std::time::Instant::now();
        debug!(
            document_id = %document_id,
            content_length = request.content.len(),
            timeout_secs = SYNC_PROCESSING_TIMEOUT_SECS,
            "Starting synchronous document processing"
        );

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(SYNC_PROCESSING_TIMEOUT_SECS),
            // SPEC-003: Use resilient processing with chunk-level error isolation
            // WHY: Map-reduce pattern continues processing even if some chunks fail
            workspace_pipeline.process_with_resilience(&document_id, &request.content, None),
        )
        .await
        .map_err(|_elapsed| {
            let processing_time = processing_start.elapsed();
            warn!(
                document_id = %document_id,
                timeout_secs = SYNC_PROCESSING_TIMEOUT_SECS,
                processing_time_secs = processing_time.as_secs(),
                content_length = request.content.len(),
                "Document processing timeout - consider using async mode for large documents"
            );
            ApiError::Timeout(format!(
                "Document processing exceeded {} seconds. For large documents (>50KB), \
                 use async_processing: true to avoid timeouts. \
                 Current document size: {} bytes",
                SYNC_PROCESSING_TIMEOUT_SECS,
                request.content.len()
            ))
        })??;

        // SPEC-003: Log partial success if some chunks failed
        if result.stats.failed_chunks > 0 {
            warn!(
                document_id = %document_id,
                successful_chunks = result.stats.successful_chunks,
                failed_chunks = result.stats.failed_chunks,
                total_chunks = result.stats.chunk_count,
                "Document processed with partial success - some chunks failed extraction"
            );

            // Emit WebSocket events for failed chunks
            if let Some(ref chunk_errors) = result.stats.chunk_errors {
                for error_info in chunk_errors {
                    state.progress_broadcaster.broadcast_chunk_failure(
                        document_id.clone(),
                        document_id.clone(), // Use doc_id as track_id for sync
                        error_info.chunk_index as u32,
                        result.stats.chunk_count as u32,
                        error_info.error_message.clone(),
                        error_info.was_timeout,
                        error_info.retry_attempts,
                    );
                }
            }
        }

        let processing_time = processing_start.elapsed();
        debug!(
            document_id = %document_id,
            processing_time_secs = processing_time.as_secs(),
            processing_time_ms = processing_time.as_millis(),
            chunk_count = result.chunks.len(),
            entity_count = result.stats.entity_count,
            "Document processing completed successfully"
        );

        // Store chunks in KV storage
        let chunks: Vec<(String, serde_json::Value)> = result
            .chunks
            .iter()
            .map(|c| {
                (
                    c.id.clone(),
                    serde_json::json!({
                        "content": c.content,
                        "document_id": document_id,
                        "index": c.index,
                    }),
                )
            })
            .collect();

        state.kv_storage.upsert(&chunks).await?;

        // SPEC-033: Get workspace-specific vector storage for document embeddings
        // This ensures embeddings are stored with correct dimension per workspace
        // WHY-OODA223: STRICT mode - fail loudly if workspace storage unavailable
        // to prevent data from being stored in the wrong (global) table
        let workspace_vector_storage =
            get_workspace_vector_storage_strict(&state, &workspace_id_for_storage).await?;

        // Store chunk embeddings in vector storage for semantic search
        let mut chunk_embeddings_stored = 0;
        for chunk in &result.chunks {
            if let Some(embedding) = &chunk.embedding {
                let mut metadata = serde_json::json!({
                    "type": "chunk",
                    "document_id": document_id,
                    "index": chunk.index,
                    "content": chunk.content,
                    "start_line": chunk.start_line,
                    "end_line": chunk.end_line,
                    "chunk_index": chunk.index,
                });

                // Add tenant and workspace IDs if present
                if let Some(ref tid) = tenant_id_for_storage {
                    metadata["tenant_id"] = serde_json::json!(tid);
                }
                metadata["workspace_id"] = serde_json::json!(&workspace_id_for_storage);

                match workspace_vector_storage
                    .upsert(&[(chunk.id.clone(), embedding.clone(), metadata)])
                    .await
                {
                    Ok(_) => {
                        chunk_embeddings_stored += 1;
                        tracing::info!(chunk_id = %chunk.id, "VECTOR STORAGE: Chunk embedding stored OK");
                    }
                    Err(e) => {
                        tracing::error!(chunk_id = %chunk.id, error = %e, "VECTOR STORAGE: Failed to store chunk embedding");
                    }
                }
            }
        }
        tracing::info!(
            chunk_embeddings_stored = chunk_embeddings_stored,
            total_chunks = result.chunks.len(),
            "VECTOR STORAGE: Chunk embedding storage complete"
        );

        // Broadcast document progress (chunking complete)
        state
            .progress_broadcaster
            .document_progress(&document_id, 0, 1, 3);

        // Store entities and relationships in graph storage
        for extraction in &result.extractions {
            for entity in &extraction.entities {
                // OODA-06 FIX (GAP-07): Merge source_ids with existing entity sources
                // WHY: When the same entity appears in multiple documents, we must
                // accumulate source_ids from ALL documents, not replace with just the current one.
                // Without this, deleting one document could orphan an entity that's still
                // referenced by other documents.
                let merged_source_ids = match state.graph_storage.get_node(&entity.name).await {
                    Ok(Some(existing)) => {
                        let mut existing_sources: std::collections::HashSet<String> = existing
                            .properties
                            .get("source_ids")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        // Add new document reference (HashSet deduplicates)
                        existing_sources.insert(document_id.clone());
                        existing_sources.into_iter().collect::<Vec<_>>()
                    }
                    _ => vec![document_id.clone()],
                };

                let mut properties = std::collections::HashMap::new();
                properties.insert(
                    "entity_type".to_string(),
                    serde_json::json!(entity.entity_type),
                );
                properties.insert(
                    "description".to_string(),
                    serde_json::json!(entity.description),
                );
                properties.insert(
                    "importance".to_string(),
                    serde_json::json!(entity.importance),
                );
                properties.insert(
                    "source_ids".to_string(),
                    serde_json::json!(merged_source_ids),
                );
                // CRITICAL: Store source_chunk_ids for Local/Global query mode chunk retrieval
                properties.insert(
                    "source_chunk_ids".to_string(),
                    serde_json::json!(&entity.source_chunk_ids),
                );

                state
                    .graph_storage
                    .upsert_node(&entity.name, properties)
                    .await?;

                // CRITICAL: Also store entity embedding in vector storage for query_local retrieval
                // SPEC-033: Use workspace-specific vector storage
                if let Some(embedding) = &entity.embedding {
                    let mut metadata = serde_json::json!({
                        "type": "entity",
                        "entity_name": entity.name,
                        "entity_type": entity.entity_type,
                        "description": entity.description,
                        "document_id": document_id,
                        "source_chunk_ids": entity.source_chunk_ids,
                    });
                    if let Some(ref tid) = tenant_id_for_storage {
                        metadata["tenant_id"] = serde_json::json!(tid);
                    }
                    metadata["workspace_id"] = serde_json::json!(&workspace_id_for_storage);

                    let entity_id = format!("entity:{}", entity.name);
                    if let Err(e) = workspace_vector_storage
                        .upsert(&[(entity_id.clone(), embedding.clone(), metadata)])
                        .await
                    {
                        tracing::error!(entity_id = %entity_id, error = %e, "Failed to store entity embedding");
                    }
                }
            }

            for relationship in &extraction.relationships {
                // OODA-06 FIX (GAP-07): Merge source_ids with existing edge sources
                // WHY: Same as entities - when the same relationship appears in multiple
                // documents, we must accumulate source_ids from ALL documents.
                let merged_source_ids = match state
                    .graph_storage
                    .get_edge(&relationship.source, &relationship.target)
                    .await
                {
                    Ok(Some(existing)) => {
                        let mut existing_sources: std::collections::HashSet<String> = existing
                            .properties
                            .get("source_ids")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        existing_sources.insert(document_id.clone());
                        existing_sources.into_iter().collect::<Vec<_>>()
                    }
                    _ => vec![document_id.clone()],
                };

                let mut properties = std::collections::HashMap::new();
                properties.insert(
                    "relation_type".to_string(),
                    serde_json::json!(relationship.relation_type),
                );
                properties.insert(
                    "description".to_string(),
                    serde_json::json!(relationship.description),
                );
                properties.insert("weight".to_string(), serde_json::json!(relationship.weight));
                properties.insert(
                    "keywords".to_string(),
                    serde_json::json!(relationship.keywords),
                );
                properties.insert(
                    "source_ids".to_string(),
                    serde_json::json!(merged_source_ids),
                );
                // CRITICAL: Store source_chunk_id for relationship chunk linkage
                if let Some(ref chunk_id) = relationship.source_chunk_id {
                    properties.insert(
                        "source_chunk_ids".to_string(),
                        serde_json::json!(vec![chunk_id]),
                    );
                }

                state
                    .graph_storage
                    .upsert_edge(&relationship.source, &relationship.target, properties)
                    .await?;
            }
        }

        // Broadcast document progress (extraction complete)
        state
            .progress_broadcaster
            .document_progress(&document_id, result.stats.entity_count, 2, 3);

        // OODA-03: Determine final status based on chunk extraction results
        // - "completed": All chunks extracted successfully WITH entities
        // - "partial_success": Some chunks succeeded, some failed (users need visibility)
        // - "partial_failure": Document processed but 0 entities extracted (FIX-5)
        // - "failed": All chunks failed (already handled upstream by error return)
        let final_status = if result.stats.failed_chunks > 0 {
            if result.stats.successful_chunks > 0 {
                "partial_success"
            } else {
                "failed"
            }
        } else if result.stats.entity_count == 0 && result.stats.chunk_count > 0 {
            // FIX-RELIABILITY: Document processed but 0 entities extracted.
            // WHY: This can happen legitimately:
            //   1. Pipeline has no extractor configured (test/mock mode)
            //   2. Content has no named entities (code, numbers, poetry)
            //   3. LLM returned unparseable response
            // Chunks are still stored and useful for semantic search.
            "partial_failure"
        } else {
            "completed"
        };

        // Update document status (preserve content_summary, content_length, track_id, tenant context)
        let doc_metadata = serde_json::json!({
            "id": document_id,
            "title": request.title,
            "content_summary": content_summary,
            "content_length": content_length,
            "content_hash": content_hash,
            "track_id": track_id,
            "created_at": Utc::now().to_rfc3339(),
            "status": final_status,
            "chunk_count": result.stats.chunk_count,
            "successful_chunks": result.stats.successful_chunks,
            "failed_chunks": result.stats.failed_chunks,
            "entity_count": result.stats.entity_count,
            "relationship_count": result.stats.relationship_count,
            "tenant_id": tenant_id_for_storage,
            "workspace_id": workspace_id_for_storage,
            "cost_usd": result.stats.cost_usd,
            "input_tokens": result.stats.input_tokens,
            "output_tokens": result.stats.output_tokens,
            "total_tokens": result.stats.total_tokens,
            "llm_model": result.stats.llm_model,
            "embedding_model": result.stats.embedding_model,
        });
        state
            .kv_storage
            .upsert(&[(doc_metadata_key, doc_metadata)])
            .await?;

        // Broadcast job finished
        let duration = start_time.elapsed();
        state
            .progress_broadcaster
            .document_progress(&document_id, result.stats.entity_count, 3, 3);
        state
            .progress_broadcaster
            .job_finished(1, duration.as_millis() as u64);

        // Build cost info from stats
        let cost = Some(DocumentCostInfo {
            total_cost_usd: result.stats.cost_usd,
            formatted_cost: format!("${:.6}", result.stats.cost_usd),
            input_tokens: result.stats.input_tokens,
            output_tokens: result.stats.output_tokens,
            total_tokens: result.stats.total_tokens,
            llm_model: result.stats.llm_model.clone(),
            embedding_model: result.stats.embedding_model.clone(),
        });

        // OODA-21: Record metrics snapshot for trend analysis after upload
        // Best-effort: log error but don't fail the upload
        if let Ok(workspace_uuid) = Uuid::parse_str(&workspace_id_for_storage) {
            if let Err(e) = state
                .workspace_service
                .record_metrics_snapshot(workspace_uuid, MetricsTriggerType::Event)
                .await
            {
                tracing::warn!(
                    workspace_id = %workspace_id_for_storage,
                    error = %e,
                    "Failed to record post-upload metrics snapshot"
                );
            } else {
                tracing::debug!(
                    workspace_id = %workspace_id_for_storage,
                    "Recorded post-upload metrics snapshot"
                );
            }

            // OODA-ITERATION-03-FIX: Invalidate workspace stats cache after document processing
            // WHY: The cache contains stale entity/relationship counts (0 before fix, or old counts)
            // Without this, Dashboard shows 0 entities while Workspace page shows correct counts
            // because both pages use the same cached stats, but cache was populated before
            // the document was processed. This ensures the next stats request fetches fresh data.
            crate::handlers::workspaces::invalidate_workspace_stats_cache(workspace_uuid).await;
        }

        Ok((
            StatusCode::CREATED,
            Json(UploadDocumentResponse {
                document_id,
                status: "processed".to_string(),
                task_id: None,
                track_id,
                duplicate_of: None,
                chunk_count: Some(result.stats.chunk_count),
                entity_count: Some(result.stats.entity_count),
                relationship_count: Some(result.stats.relationship_count),
                cost,
            }),
        ))
    }
}


/// Upload a file via multipart form.
///
/// Supports text-based files: .txt, .md, .json, .csv, .html
#[utoipa::path(
    post,
    path = "/api/v1/documents/upload",
    tag = "Documents",
    request_body(content_type = "multipart/form-data", description = "File to upload"),
    responses(
        (status = 201, description = "File uploaded successfully", body = FileUploadResponse),
        (status = 400, description = "Invalid file or request"),
        (status = 409, description = "Duplicate file (already processed)"),
        (status = 413, description = "File too large")
    )
)]
pub async fn upload_file(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<FileUploadResponse>)> {
    debug!(
        tenant_id = ?tenant_ctx.tenant_id,
        workspace_id = ?tenant_ctx.workspace_id,
        "Uploading file with tenant context"
    );

    let mut filename = String::new();
    let mut content = Vec::new();
    let mut metadata: Option<serde_json::Value> = None;

    // Process multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                // Get filename
                filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unnamed.txt".to_string());

                // Read file content
                content = field
                    .bytes()
                    .await
                    .map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read file content: {}", e))
                    })?
                    .to_vec();
            }
            "metadata" => {
                // Optional metadata field
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read metadata: {}", e)))?;

                if !text.is_empty() {
                    metadata = serde_json::from_str(&text).ok();
                }
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate we got a file
    if content.is_empty() {
        return Err(ApiError::BadRequest("No file provided".to_string()));
    }

    // Validate file (size, extension, UTF-8, non-empty)
    let (_extension, text_content, mime_type) =
        validate_file(&filename, &content, state.config.max_document_size)?;

    // WHY-OODA83: Use ContentHasher service for consistent hash computation (DRY)
    let content_hash = ContentHasher::hash_bytes(&content);
    debug!(content_hash = %content_hash, "Computed content hash");

    // Extract tenant context for workspace-scoped uniqueness
    // WHY-OODA81: Uniqueness must be scoped to workspace, not global
    // Same document in different workspaces is allowed (multi-tenancy)
    let workspace_id_for_storage = tenant_ctx
        .workspace_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let tenant_id_for_storage = tenant_ctx.tenant_id.clone();

    // WHY-OODA81+83: Use ContentHasher for workspace-scoped hash key
    // FIX-4: Duplicates now trigger re-ingestion instead of rejection
    let hash_key = ContentHasher::workspace_hash_key(&workspace_id_for_storage, &content_hash);
    debug!(hash_key = %hash_key, workspace_id = %workspace_id_for_storage, "Checking for workspace-scoped duplicate hash");
    if let Some(existing_doc_id) = state.kv_storage.get_by_id(&hash_key).await? {
        debug!(existing_doc_id = ?existing_doc_id, "Found existing document for hash in workspace");
        if let Some(doc_id_str) = existing_doc_id.as_str() {
            // FIX-4: Try to delete old document data for re-ingestion
            match delete_document_for_reingestion(doc_id_str, &state, &workspace_id_for_storage)
                .await
            {
                Ok(true) => {
                    // Successfully deleted - proceed with new upload
                    tracing::info!(
                        old_doc_id = %doc_id_str,
                        workspace_id = %workspace_id_for_storage,
                        filename = %filename,
                        "Duplicate file found - old data deleted, proceeding with re-ingestion"
                    );
                    // Hash key will be updated below with new document_id
                }
                Ok(false) => {
                    // Document still processing - return duplicate response
                    tracing::warn!(
                        old_doc_id = %doc_id_str,
                        filename = %filename,
                        "Duplicate file is still being processed - cannot re-ingest"
                    );
                    return Ok((
                        StatusCode::OK,
                        Json(FileUploadResponse {
                            document_id: doc_id_str.to_string(),
                            filename,
                            size: content.len(),
                            content_hash,
                            status: "duplicate_processing".to_string(),
                            chunk_count: 0,
                            entity_count: 0,
                            relationship_count: 0,
                            is_duplicate: true,
                        }),
                    ));
                }
                Err(e) => {
                    // Failed to delete - log error and proceed with re-ingestion anyway
                    tracing::warn!(
                        old_doc_id = %doc_id_str,
                        filename = %filename,
                        error = %e,
                        "Failed to delete old file data - proceeding with re-ingestion"
                    );
                }
            }
        }
    }

    // Generate document ID
    let document_id = Uuid::new_v4().to_string();

    // Store hash mapping for deduplication (workspace-scoped)
    state
        .kv_storage
        .upsert(&[(hash_key, serde_json::json!(document_id))])
        .await?;

    // Generate content summary
    let content_summary = crate::validation::generate_content_summary(&text_content);

    // Generate track ID
    let track_id = format!(
        "upload_{}_{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        &Uuid::new_v4().to_string()[..8]
    );

    // Store comprehensive document metadata
    let doc_metadata_key = format!("{}-metadata", document_id);
    let doc_metadata = serde_json::json!({
        "id": document_id,
        "title": filename,
        "file_name": filename,
        "file_size": content.len(),
        "mime_type": mime_type,
        "source_type": "file",
        "content_summary": content_summary,
        "content_length": text_content.len(),
        "content_hash": content_hash,
        "track_id": track_id,
        "created_at": Utc::now().to_rfc3339(),
        "status": "processing",
        "tenant_id": tenant_id_for_storage,
        "workspace_id": workspace_id_for_storage,
        "custom_metadata": metadata,
    });
    state
        .kv_storage
        .upsert(&[(doc_metadata_key.clone(), doc_metadata)])
        .await?;

    // Store document content
    let doc_content_key = format!("{}-content", document_id);
    let doc_content = serde_json::json!({
        "content": text_content,
    });
    state
        .kv_storage
        .upsert(&[(doc_content_key, doc_content)])
        .await?;

    // Process through pipeline
    let result = state
        .pipeline
        .process_with_resilience(&document_id, &text_content, None)
        .await?;

    // Log partial failures but continue (resilient processing)
    if result.stats.failed_chunks > 0 {
        tracing::warn!(
            document_id = %document_id,
            failed_chunks = result.stats.failed_chunks,
            chunk_count = result.stats.chunk_count,
            "File upload pipeline completed with partial failures"
        );
    }

    // Store chunks in KV storage
    let chunks: Vec<(String, serde_json::Value)> = result
        .chunks
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                serde_json::json!({
                    "content": c.content,
                    "document_id": document_id,
                    "index": c.index,
                    "source_file": filename,
                }),
            )
        })
        .collect();

    state.kv_storage.upsert(&chunks).await?;

    // SPEC-033: Get workspace-specific vector storage for file embeddings
    // WHY-OODA223: STRICT mode - fail loudly if workspace storage unavailable
    // to prevent file embeddings from being stored in the wrong (global) table
    let workspace_vector_storage =
        get_workspace_vector_storage_strict(&state, &workspace_id_for_storage).await?;

    // Store chunk embeddings in vector storage for semantic search
    let mut chunk_embeddings_stored = 0;
    for chunk in &result.chunks {
        if let Some(embedding) = &chunk.embedding {
            let mut metadata = serde_json::json!({
                "type": "chunk",
                "document_id": document_id,
                "index": chunk.index,
                "content": chunk.content,
                "source_file": filename,
            });

            // Add tenant and workspace IDs if present
            if let Some(ref tid) = tenant_id_for_storage {
                metadata["tenant_id"] = serde_json::json!(tid);
            }
            metadata["workspace_id"] = serde_json::json!(&workspace_id_for_storage);

            match workspace_vector_storage
                .upsert(&[(chunk.id.clone(), embedding.clone(), metadata)])
                .await
            {
                Ok(_) => {
                    chunk_embeddings_stored += 1;
                    tracing::info!(chunk_id = %chunk.id, "VECTOR STORAGE: Chunk embedding stored OK");
                }
                Err(e) => {
                    tracing::error!(chunk_id = %chunk.id, error = %e, "VECTOR STORAGE: Failed to store chunk embedding");
                }
            }
        }
    }
    tracing::info!(
        chunk_embeddings_stored = chunk_embeddings_stored,
        total_chunks = result.chunks.len(),
        "VECTOR STORAGE: Chunk embedding storage complete"
    );

    // Store entities and relationships in graph storage
    tracing::info!(
        extraction_count = result.extractions.len(),
        "GRAPH STORAGE: Processing extractions"
    );
    for extraction in &result.extractions {
        tracing::info!(
            entity_count = extraction.entities.len(),
            relationship_count = extraction.relationships.len(),
            "GRAPH STORAGE: Extraction content"
        );
        for entity in &extraction.entities {
            tracing::info!(
                entity_name = %entity.name,
                entity_type = %entity.entity_type,
                source_chunk_ids = ?entity.source_chunk_ids,
                "GRAPH STORAGE: Storing entity with chunk linkage"
            );
            let mut properties = std::collections::HashMap::new();
            properties.insert(
                "entity_type".to_string(),
                serde_json::json!(entity.entity_type),
            );
            properties.insert(
                "description".to_string(),
                serde_json::json!(entity.description),
            );
            properties.insert(
                "importance".to_string(),
                serde_json::json!(entity.importance),
            );
            properties.insert(
                "source_ids".to_string(),
                serde_json::json!(vec![&document_id]),
            );
            // CRITICAL: Store source_chunk_ids for Local/Global query mode chunk retrieval
            properties.insert(
                "source_chunk_ids".to_string(),
                serde_json::json!(&entity.source_chunk_ids),
            );
            // Add tenant scoping
            if let Some(ref tid) = tenant_id_for_storage {
                properties.insert("tenant_id".to_string(), serde_json::json!(tid));
            }
            properties.insert(
                "workspace_id".to_string(),
                serde_json::json!(&workspace_id_for_storage),
            );

            match state
                .graph_storage
                .upsert_node(&entity.name, properties)
                .await
            {
                Ok(_) => {
                    tracing::info!(entity_name = %entity.name, "GRAPH STORAGE: Entity stored OK")
                }
                Err(e) => {
                    tracing::error!(entity_name = %entity.name, error = %e, "GRAPH STORAGE: Failed to store entity")
                }
            }

            // CRITICAL: Also store entity embedding in vector storage for query_local retrieval
            tracing::info!(
                entity_name = %entity.name,
                has_embedding = entity.embedding.is_some(),
                embedding_dim = entity.embedding.as_ref().map(|e| e.len()).unwrap_or(0),
                "Checking entity embedding for storage"
            );
            // SPEC-033: Use workspace-specific vector storage for entity embeddings
            if let Some(embedding) = &entity.embedding {
                let mut metadata = serde_json::json!({
                    "type": "entity",
                    "entity_name": entity.name,
                    "entity_type": entity.entity_type,
                    "description": entity.description,
                    "document_id": document_id,
                    "source_chunk_ids": entity.source_chunk_ids,
                });
                if let Some(ref tid) = tenant_id_for_storage {
                    metadata["tenant_id"] = serde_json::json!(tid);
                }
                metadata["workspace_id"] = serde_json::json!(&workspace_id_for_storage);

                // Use entity name as vector ID for dedup
                let entity_id = format!("entity:{}", entity.name);
                if let Err(e) = workspace_vector_storage
                    .upsert(&[(entity_id.clone(), embedding.clone(), metadata)])
                    .await
                {
                    tracing::error!(entity_id = %entity_id, error = %e, "VECTOR STORAGE: Failed to store entity embedding");
                } else {
                    tracing::info!(entity_id = %entity_id, "VECTOR STORAGE: Entity embedding stored OK");
                }
            }
        }

        for relationship in &extraction.relationships {
            let mut properties = std::collections::HashMap::new();
            properties.insert(
                "relation_type".to_string(),
                serde_json::json!(relationship.relation_type),
            );
            properties.insert(
                "description".to_string(),
                serde_json::json!(relationship.description),
            );
            properties.insert("weight".to_string(), serde_json::json!(relationship.weight));
            properties.insert(
                "keywords".to_string(),
                serde_json::json!(relationship.keywords),
            );
            properties.insert(
                "source_ids".to_string(),
                serde_json::json!(vec![&document_id]),
            );
            // CRITICAL: Store source_chunk_id for relationship chunk linkage
            if let Some(ref chunk_id) = relationship.source_chunk_id {
                properties.insert(
                    "source_chunk_ids".to_string(),
                    serde_json::json!(vec![chunk_id]),
                );
            }
            // Add tenant scoping
            if let Some(ref tid) = tenant_id_for_storage {
                properties.insert("tenant_id".to_string(), serde_json::json!(tid));
            }
            properties.insert(
                "workspace_id".to_string(),
                serde_json::json!(&workspace_id_for_storage),
            );

            let _ = state
                .graph_storage
                .upsert_edge(&relationship.source, &relationship.target, properties)
                .await;
        }
    }

    // Update document metadata with completion stats and lineage
    let completed_metadata = serde_json::json!({
        "id": document_id,
        "title": filename,
        "file_name": filename,
        "file_size": content.len(),
        "mime_type": mime_type,
        "source_type": "file",
        "content_summary": content_summary,
        "content_length": text_content.len(),
        "content_hash": content_hash,
        "track_id": track_id,
        "created_at": Utc::now().to_rfc3339(),
        "processed_at": Utc::now().to_rfc3339(),
        "status": "completed",
        "chunk_count": result.stats.chunk_count,
        "entity_count": result.stats.entity_count,
        "relationship_count": result.stats.relationship_count,
        "tenant_id": tenant_id_for_storage,
        "workspace_id": workspace_id_for_storage,
        "custom_metadata": metadata,
        // Lineage information
        "llm_model": result.stats.llm_model,
        "embedding_model": result.stats.embedding_model,
        "embedding_dimensions": result.stats.embedding_dimensions,
        "entity_types": result.stats.entity_types,
        "relationship_types": result.stats.relationship_types,
        "keywords": result.stats.keywords,
        "chunking_strategy": result.stats.chunking_strategy,
        "avg_chunk_size": result.stats.avg_chunk_size,
        "processing_duration_ms": result.stats.processing_time_ms,
    });
    state
        .kv_storage
        .upsert(&[(doc_metadata_key, completed_metadata)])
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(FileUploadResponse {
            document_id,
            filename,
            size: content.len(),
            content_hash,
            status: "processed".to_string(),
            chunk_count: result.stats.chunk_count,
            entity_count: result.stats.entity_count,
            relationship_count: result.stats.relationship_count,
            is_duplicate: false,
        }),
    ))
}

/// Upload multiple files via multipart form.
#[utoipa::path(
    post,
    path = "/api/v1/documents/upload/batch",
    tag = "Documents",
    request_body(content_type = "multipart/form-data", description = "Files to upload"),
    responses(
        (status = 201, description = "Batch upload completed", body = BatchUploadResponse),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn upload_files_batch(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<BatchUploadResponse>)> {
    let mut results = Vec::new();
    let mut processed = 0usize;
    let mut duplicates = 0usize;
    let mut failed = 0usize;

    // Collect all files first
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "files" || field_name == "file" {
            let filename = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("file_{}.txt", files.len()));

            let content = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?
                .to_vec();

            files.push((filename, content));
        }
    }

    // Process each file (uses default workspace for batch uploads)
    // WHY-OODA81: Batch upload uses "default" workspace for dedup scoping
    // For proper workspace isolation, use the single file upload endpoint with tenant context
    let workspace_id = "default".to_string();
    for (filename, content) in files {
        let result = process_single_file(&state, &filename, &content, &workspace_id).await;

        match result {
            Ok((doc_id, is_duplicate)) => {
                if is_duplicate {
                    duplicates += 1;
                    results.push(BatchFileResult {
                        filename,
                        document_id: Some(doc_id),
                        status: "duplicate".to_string(),
                        error: None,
                    });
                } else {
                    processed += 1;
                    results.push(BatchFileResult {
                        filename,
                        document_id: Some(doc_id),
                        status: "processed".to_string(),
                        error: None,
                    });
                }
            }
            Err(e) => {
                failed += 1;
                results.push(BatchFileResult {
                    filename,
                    document_id: None,
                    status: "failed".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(BatchUploadResponse {
            total_files: results.len(),
            processed,
            duplicates,
            failed,
            results,
        }),
    ))
}

/// Process a single file and return (document_id, is_duplicate).
///
/// WHY-OODA81: workspace_id parameter enables workspace-scoped duplicate detection.
/// Same document in different workspaces is allowed (multi-tenancy support).
///
/// Note: Uses default vector storage for batch uploads without tenant context.
/// For workspace-specific storage, use the main upload_file endpoint with tenant context.
async fn process_single_file(
    state: &AppState,
    filename: &str,
    content: &[u8],
    workspace_id: &str,
) -> Result<(String, bool), ApiError> {
    // Validate file (size, extension, UTF-8, non-empty)
    let (_extension, text_content, _mime_type) =
        validate_file(filename, content, state.config.max_document_size)?;

    // WHY-OODA83: Use ContentHasher service for consistent hash computation (DRY)
    let content_hash = ContentHasher::hash_bytes(content);

    // WHY-OODA81+83: Use ContentHasher for workspace-scoped hash key
    let hash_key = ContentHasher::workspace_hash_key(workspace_id, &content_hash);
    if let Some(existing) = state.kv_storage.get_by_id(&hash_key).await? {
        if let Some(doc_id) = existing.as_str() {
            return Ok((doc_id.to_string(), true));
        }
    }

    // Generate document ID
    let document_id = Uuid::new_v4().to_string();

    // Store hash mapping
    state
        .kv_storage
        .upsert(&[(hash_key, serde_json::json!(document_id))])
        .await?;

    // Process through pipeline (resilient - tolerates partial chunk failures)
    let result = state
        .pipeline
        .process_with_resilience(&document_id, &text_content, None)
        .await?;

    if result.stats.failed_chunks > 0 {
        tracing::warn!(
            document_id = %document_id,
            filename = %filename,
            failed_chunks = result.stats.failed_chunks,
            chunk_count = result.stats.chunk_count,
            "Batch file pipeline completed with partial failures"
        );
    }

    // Store chunks
    let chunks: Vec<(String, serde_json::Value)> = result
        .chunks
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                serde_json::json!({
                    "content": c.content,
                    "document_id": document_id,
                    "index": c.index,
                    "source_file": filename,
                }),
            )
        })
        .collect();

    state.kv_storage.upsert(&chunks).await?;

    // Store chunk embeddings in vector storage for semantic search
    // Note: Batch upload uses default vector storage since there's no workspace context.
    // For workspace-specific storage, use the main upload_file endpoint with tenant context.
    for chunk in &result.chunks {
        if let Some(embedding) = &chunk.embedding {
            let metadata = serde_json::json!({
                "type": "chunk",
                "document_id": document_id,
                "index": chunk.index,
                "content": chunk.content,
                "source_file": filename,
            });

            let _ = state
                .vector_storage
                .upsert(&[(chunk.id.clone(), embedding.clone(), metadata)])
                .await;
        }
    }

    Ok((document_id, false))
}
