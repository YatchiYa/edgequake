use axum::{extract::State, Json};
use chrono::Utc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;
#[cfg(feature = "postgres")]
use edgequake_storage::ListPdfFilter;

use crate::handlers::documents_types::*;
#[allow(unused_imports)]
use super::storage_helpers::get_workspace_vector_storage_with_fallback;

/// List all documents.
#[utoipa::path(
    get,
    path = "/api/v1/documents",
    tag = "Documents",
    responses(
        (status = 200, description = "Documents retrieved", body = ListDocumentsResponse)
    )
)]
#[allow(clippy::field_reassign_with_default)]
pub async fn list_documents(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
) -> ApiResult<Json<ListDocumentsResponse>> {
    debug!(
        tenant_id = ?tenant_ctx.tenant_id,
        workspace_id = ?tenant_ctx.workspace_id,
        "Listing documents with tenant context"
    );

    // SECURITY: Enforce strict tenant context requirement - NO EXCEPTIONS
    // This matches the strict filtering in entities.rs and relationships.rs (commit d11edba8)
    if tenant_ctx.tenant_id.is_none() || tenant_ctx.workspace_id.is_none() {
        warn!(
            tenant_id = ?tenant_ctx.tenant_id,
            workspace_id = ?tenant_ctx.workspace_id,
            "Tenant context missing - returning empty document list for security"
        );
        return Ok(Json(ListDocumentsResponse {
            documents: vec![],
            total: 0,
            page: 1,
            page_size: 100,
            total_pages: 0,
            has_more: false,
            status_counts: StatusCounts {
                pending: 0,
                processing: 0,
                completed: 0,
                partial_failure: 0,
                failed: 0,
                cancelled: 0,
            },
        }));
    }

    let keys = state.kv_storage.keys().await?;
    debug!(key_count = keys.len(), "Total keys in KV storage");
    debug!(keys = ?keys, "All keys in KV storage");

    // Group by document and collect metadata keys
    let mut doc_chunks: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut metadata_keys: Vec<String> = Vec::new();

    for key in &keys {
        if key.ends_with("-metadata") {
            debug!(metadata_key = %key, "Found metadata key");
            metadata_keys.push(key.clone());
        } else if key.contains("-chunk-") {
            // Only count actual chunk keys (e.g., "doc-id-chunk-0")
            if let Some(doc_id) = key.split("-chunk-").next() {
                // Filter out non-document keys (like -metadata, -content suffixes)
                if !doc_id.ends_with("-metadata") && !doc_id.ends_with("-content") {
                    *doc_chunks.entry(doc_id.to_string()).or_default() += 1;
                }
            }
        }
    }

    // Fetch all metadata and store complete document info
    debug!(
        metadata_keys_count = metadata_keys.len(),
        "Fetching metadata for keys"
    );
    let metadata_values = state.kv_storage.get_by_ids(&metadata_keys).await?;
    debug!(
        metadata_values_count = metadata_values.len(),
        "Metadata values retrieved"
    );

    // Store complete document metadata, keyed by document ID
    #[derive(Default)]
    struct DocMetadata {
        title: Option<String>,
        file_name: Option<String>,
        content_summary: Option<String>,
        content_length: Option<usize>,
        status: Option<String>,
        error_message: Option<String>,
        track_id: Option<String>,
        created_at: Option<String>,
        updated_at: Option<String>,
        entity_count: Option<usize>,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        cost_usd: Option<f64>,
        input_tokens: Option<usize>,
        output_tokens: Option<usize>,
        total_tokens: Option<usize>,
        llm_model: Option<String>,
        embedding_model: Option<String>,
        // SPEC-002: Unified Ingestion Pipeline fields
        source_type: Option<String>,
        current_stage: Option<String>,
        stage_progress: Option<f32>,
        stage_message: Option<String>,
        pdf_id: Option<String>,
    }

    let mut doc_metadata: std::collections::HashMap<String, DocMetadata> =
        std::collections::HashMap::new();

    for value in metadata_values {
        debug!(value = ?value, "Processing metadata value");
        if let Some(obj) = value.as_object() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let title_val = obj.get("title");
                debug!(doc_id = %id, title = ?title_val, "Extracted ID and title from metadata");

                // WHY: We build DocMetadata incrementally because fields are extracted
                // conditionally from JSON, and some fields depend on others (e.g., file_name
                // is derived from title). Struct initializer syntax doesn't work well here.
                let mut meta = DocMetadata::default();

                // Get title from metadata
                meta.title = obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Use title as file_name fallback if it looks like a filename
                if let Some(ref title) = meta.title {
                    if title.contains('.') {
                        meta.file_name = Some(title.clone());
                    }
                }

                // Get content_summary
                meta.content_summary = obj
                    .get("content_summary")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get content_length
                meta.content_length = obj
                    .get("content_length")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);

                // Get status
                meta.status = obj
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get error_message
                meta.error_message = obj
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get track_id
                meta.track_id = obj
                    .get("track_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get created_at
                meta.created_at = obj
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get updated_at
                meta.updated_at = obj
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get entity_count
                meta.entity_count = obj
                    .get("entity_count")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);

                // Get tenant_id
                meta.tenant_id = obj
                    .get("tenant_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get workspace_id
                meta.workspace_id = obj
                    .get("workspace_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get cost_usd
                meta.cost_usd = obj.get("cost_usd").and_then(|v| v.as_f64());

                // Get input_tokens
                meta.input_tokens = obj
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);

                // Get output_tokens
                meta.output_tokens = obj
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);

                // Get total_tokens
                meta.total_tokens = obj
                    .get("total_tokens")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);

                // Get llm_model
                meta.llm_model = obj
                    .get("llm_model")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Get embedding_model
                meta.embedding_model = obj
                    .get("embedding_model")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // SPEC-002: Get source_type
                meta.source_type = obj
                    .get("source_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // SPEC-002: Get current_stage
                meta.current_stage = obj
                    .get("current_stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // SPEC-002: Get stage_progress
                meta.stage_progress = obj
                    .get("stage_progress")
                    .and_then(|v| v.as_f64())
                    .map(|n| n as f32);

                // SPEC-002: Get stage_message
                meta.stage_message = obj
                    .get("stage_message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // SPEC-002: Get pdf_id (linked PDF document for viewing)
                meta.pdf_id = obj
                    .get("pdf_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                doc_metadata.insert(id.to_string(), meta);
            }
        }
    }

    // Filter documents by tenant context
    let filter_workspace_id = tenant_ctx.workspace_id.clone();
    let filter_tenant_id = tenant_ctx.tenant_id.clone();

    // SECURITY: STRICT tenant filtering - both tenant_id AND workspace_id must match
    // This matches the strict filtering in entities.rs and relationships.rs (commit d11edba8)
    let matches_tenant_context = |meta: &DocMetadata| -> bool {
        // Both must match exactly (None is already handled by early return above)
        meta.workspace_id.as_ref() == filter_workspace_id.as_ref()
            && meta.tenant_id.as_ref() == filter_tenant_id.as_ref()
    };

    // Build document list from BOTH:
    // 1. Documents with chunks (processed)
    // 2. Documents with metadata but no chunks yet (pending/processing)
    let mut documents: Vec<DocumentSummary> = doc_chunks
        .into_iter()
        .filter_map(|(id, chunk_count)| {
            let meta = doc_metadata.remove(&id).unwrap_or_default();
            // Filter by tenant context
            if !matches_tenant_context(&meta) {
                return None;
            }
            Some(DocumentSummary {
                id,
                title: meta.title,
                file_name: meta.file_name,
                content_summary: meta.content_summary,
                content_length: meta.content_length,
                chunk_count,
                entity_count: meta.entity_count,
                status: meta.status,
                error_message: meta.error_message,
                track_id: meta.track_id,
                created_at: meta.created_at,
                updated_at: meta.updated_at,
                cost_usd: meta.cost_usd,
                input_tokens: meta.input_tokens,
                output_tokens: meta.output_tokens,
                total_tokens: meta.total_tokens,
                llm_model: meta.llm_model,
                embedding_model: meta.embedding_model,
                // SPEC-002: Unified Ingestion Pipeline fields
                source_type: meta.source_type,
                current_stage: meta.current_stage,
                stage_progress: meta.stage_progress,
                stage_message: meta.stage_message,
                pdf_id: meta.pdf_id,
            })
        })
        .collect();

    // Add documents that have metadata but no chunks yet (pending/processing)
    for (id, meta) in doc_metadata {
        // Filter by tenant context
        if !matches_tenant_context(&meta) {
            continue;
        }
        documents.push(DocumentSummary {
            id,
            title: meta.title,
            file_name: meta.file_name,
            content_summary: meta.content_summary,
            content_length: meta.content_length,
            chunk_count: 0,
            entity_count: meta.entity_count,
            status: meta.status,
            error_message: meta.error_message,
            track_id: meta.track_id,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            cost_usd: meta.cost_usd,
            input_tokens: meta.input_tokens,
            output_tokens: meta.output_tokens,
            total_tokens: meta.total_tokens,
            llm_model: meta.llm_model,
            embedding_model: meta.embedding_model,
            // SPEC-002: Unified Ingestion Pipeline fields
            source_type: meta.source_type,
            current_stage: meta.current_stage,
            stage_progress: meta.stage_progress,
            stage_message: meta.stage_message,
            pdf_id: meta.pdf_id,
        });
    }

    // Sort by created_at descending (newest first)
    documents.sort_by(|a, b| {
        b.created_at
            .as_deref()
            .unwrap_or("")
            .cmp(a.created_at.as_deref().unwrap_or(""))
    });

    // Calculate status counts for all documents
    let status_counts = StatusCounts {
        pending: documents
            .iter()
            .filter(|d| d.status.as_deref() == Some("pending"))
            .count(),
        processing: documents
            .iter()
            .filter(|d| d.status.as_deref() == Some("processing"))
            .count(),
        completed: documents
            .iter()
            .filter(|d| {
                d.status.is_none()
                    || d.status.as_deref() == Some("completed")
                    || d.status.as_deref() == Some("indexed")
            })
            .count(),
        // FIX-5: Track partial_failure status
        partial_failure: documents
            .iter()
            .filter(|d| d.status.as_deref() == Some("partial_failure"))
            .count(),
        failed: documents
            .iter()
            .filter(|d| d.status.as_deref() == Some("failed"))
            .count(),
        cancelled: documents
            .iter()
            .filter(|d| d.status.as_deref() == Some("cancelled"))
            .count(),
    };

    let total = documents.len();
    let page_size = 20usize;
    let total_pages = (total + page_size - 1) / page_size.max(1);
    let page = 1usize;
    let has_more = page < total_pages;

    Ok(Json(ListDocumentsResponse {
        total,
        documents,
        page,
        page_size,
        total_pages,
        has_more,
        status_counts,
    }))
}

/// Get a document by ID.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}",
    tag = "Documents",
    params(
        ("document_id" = String, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Document found", body = DocumentDetailResponse),
        (status = 404, description = "Document not found"),
        (status = 403, description = "Access denied - document belongs to different tenant")
    )
)]
pub async fn get_document(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    axum::extract::Path(document_id): axum::extract::Path<String>,
) -> ApiResult<Json<DocumentDetailResponse>> {
    debug!(
        document_id = %document_id,
        tenant_id = ?tenant_ctx.tenant_id,
        workspace_id = ?tenant_ctx.workspace_id,
        "Getting document by ID with tenant context"
    );

    // Fetch document metadata
    let metadata_key = format!("{}-metadata", document_id);
    debug!(metadata_key = %metadata_key, "Looking up metadata key");
    let metadata_values = state
        .kv_storage
        .get_by_ids(std::slice::from_ref(&metadata_key))
        .await?;
    debug!(
        metadata_count = metadata_values.len(),
        "Metadata values retrieved"
    );

    let metadata = metadata_values.into_iter().next();
    debug!(has_metadata = metadata.is_some(), "Metadata value present");

    // Check if document exists by metadata or chunks
    let keys = state.kv_storage.keys().await?;
    debug!(total_keys = keys.len(), "Total keys in storage");
    let matching_keys: Vec<_> = keys
        .iter()
        .filter(|k| k.contains(&document_id))
        .cloned()
        .collect();
    debug!(matching_keys = ?matching_keys, "Keys matching document ID");
    let chunk_count = keys
        .iter()
        .filter(|k| k.starts_with(&format!("{}-chunk-", document_id)))
        .count();

    // Document must have either metadata or chunks
    if metadata.is_none() && chunk_count == 0 {
        return Err(ApiError::NotFound(format!(
            "Document {} not found",
            document_id
        )));
    }

    // Parse metadata if available
    let meta_obj = metadata.as_ref().and_then(|v| v.as_object());

    // Check tenant context (multi-tenancy)
    if let Some(obj) = meta_obj {
        let doc_tenant_id = obj.get("tenant_id").and_then(|v| v.as_str());
        let doc_workspace_id = obj.get("workspace_id").and_then(|v| v.as_str());

        // Verify tenant access
        if let Some(ref filter_tid) = tenant_ctx.tenant_id {
            if let Some(doc_tid) = doc_tenant_id {
                if doc_tid != filter_tid {
                    return Err(ApiError::Forbidden);
                }
            }
        }

        // Verify workspace access
        if let Some(ref filter_ws) = tenant_ctx.workspace_id {
            if let Some(doc_ws) = doc_workspace_id {
                if doc_ws != filter_ws {
                    return Err(ApiError::Forbidden);
                }
            }
        }
    }

    // Fetch document content
    let content_key = format!("{}-content", document_id);
    let content_values = state.kv_storage.get_by_ids(&[content_key]).await?;
    let content = content_values.into_iter().next().and_then(|v| {
        v.get("content")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    });

    // SPEC-040: Async fallback PDF vision model lookup for backward compatibility.
    // WHY: Documents processed before pdf_vision_model was written to KV metadata JSON
    // don't have that field. We query the pdf_documents table as fallback using the
    // pdf_id that IS stored in all document metadata records.
    let (fallback_pdf_vision_model, fallback_pdf_extraction_method): (
        Option<String>,
        Option<String>,
    ) = {
        let needs_fallback = meta_obj
            .and_then(|obj| obj.get("pdf_vision_model"))
            .is_none();
        let pdf_uuid_opt = if needs_fallback {
            meta_obj
                .and_then(|obj| obj.get("pdf_id"))
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
        } else {
            None
        };
        if let Some(pdf_uuid) = pdf_uuid_opt {
            #[cfg(feature = "postgres")]
            {
                if let Some(ref pool) = state.pg_pool {
                    match sqlx::query_as::<_, (Option<String>, Option<String>)>(
                        "SELECT vision_model, extraction_method FROM pdf_documents WHERE pdf_id = $1",
                    )
                    .bind(pdf_uuid)
                    .fetch_optional(pool)
                    .await
                    {
                        Ok(Some((vision_model, extraction_method))) => (vision_model, extraction_method),
                        _ => (None, None),
                    }
                } else {
                    (None, None)
                }
            }
            #[cfg(not(feature = "postgres"))]
            {
                let _ = pdf_uuid;
                (None, None)
            }
        } else {
            (None, None)
        }
    };

    // Build response from metadata
    let (
        title,
        file_name,
        content_summary,
        content_length,
        content_hash,
        entity_count,
        relationship_count,
        status,
        error_message,
        source_type,
        mime_type,
        file_size,
        track_id,
        tenant_id,
        workspace_id,
        created_at,
        updated_at,
        processed_at,
        lineage,
        custom_metadata,
        pdf_id,
    ) = if let Some(obj) = meta_obj {
        // Build lineage information from stored metadata
        let lineage = {
            let llm_model = obj
                .get("llm_model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let embedding_model = obj
                .get("embedding_model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let embedding_dimensions = obj
                .get("embedding_dimensions")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            let keywords = obj.get("keywords").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });
            let entity_types = obj
                .get("entity_types")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                });
            let relationship_types = obj
                .get("relationship_types")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                });
            let chunking_strategy = obj
                .get("chunking_strategy")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let avg_chunk_size = obj
                .get("avg_chunk_size")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            let processing_duration_ms = obj.get("processing_duration_ms").and_then(|v| v.as_u64());

            // Token usage and cost fields
            let input_tokens = obj
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            let output_tokens = obj
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            let total_tokens = obj
                .get("total_tokens")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            let cost_usd = obj.get("cost_usd").and_then(|v| v.as_f64());

            // SPEC-040: PDF extraction lineage fields
            // WHY: vision_model and extraction_method are stored in metadata JSON by the PDF
            // processor so the document detail view can show what model was used for extraction.
            // For documents processed before this field was added, fall back to the values
            // looked up from the pdf_documents table (fallback_pdf_vision_model).
            let pdf_vision_model = obj
                .get("pdf_vision_model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| fallback_pdf_vision_model.clone());
            let pdf_extraction_method = obj
                .get("pdf_extraction_method")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| fallback_pdf_extraction_method.clone());

            // Only include lineage if we have at least one field
            if llm_model.is_some()
                || embedding_model.is_some()
                || keywords.is_some()
                || entity_types.is_some()
                || relationship_types.is_some()
                || chunking_strategy.is_some()
                || processing_duration_ms.is_some()
                || input_tokens.is_some()
                || cost_usd.is_some()
                || pdf_vision_model.is_some()
            {
                Some(DocumentLineage {
                    llm_model,
                    embedding_model,
                    embedding_dimensions,
                    keywords,
                    entity_types,
                    relationship_types,
                    chunking_strategy,
                    avg_chunk_size,
                    processing_duration_ms,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    cost_usd,
                    pdf_vision_model,
                    pdf_extraction_method,
                })
            } else {
                None
            }
        };

        (
            obj.get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("file_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    obj.get("title")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                }),
            obj.get("content_summary")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("content_length")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            obj.get("content_hash")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("entity_count")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            obj.get("relationship_count")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            obj.get("status")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "completed".to_string()),
            obj.get("error_message")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("source_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("mime_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("file_size")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            obj.get("track_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("tenant_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("workspace_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("created_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("updated_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            obj.get("processed_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            lineage,
            obj.get("custom_metadata").cloned(),
            // OODA-50: Extract pdf_id from metadata for PDF viewer
            obj.get("pdf_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        )
    } else {
        // Fallback for documents without metadata (legacy)
        (
            None,                    // title
            None,                    // file_name
            None,                    // content_summary
            None,                    // content_length
            None,                    // content_hash
            None,                    // entity_count
            None,                    // relationship_count
            "completed".to_string(), // status
            None,                    // error_message
            None,                    // source_type
            None,                    // mime_type
            None,                    // file_size
            None,                    // track_id
            None,                    // tenant_id
            None,                    // workspace_id
            None,                    // created_at
            None,                    // updated_at
            None,                    // processed_at
            None,                    // lineage
            None,                    // custom_metadata
            None,                    // pdf_id
        )
    };

    Ok(Json(DocumentDetailResponse {
        id: document_id,
        title,
        file_name,
        content,
        content_summary,
        content_length,
        content_hash,
        chunk_count,
        entity_count,
        relationship_count,
        status,
        error_message,
        source_type,
        mime_type,
        file_size,
        track_id,
        tenant_id,
        workspace_id,
        created_at,
        updated_at,
        processed_at,
        lineage,
        metadata: custom_metadata,
        // OODA-50: Use pdf_id from metadata for PDF viewer
        pdf_id,
    }))
}


/// Get track status by track ID.
///
/// Returns all documents uploaded with a specific track_id, along with status summary.
#[utoipa::path(
    get,
    path = "/api/v1/documents/track/{track_id}",
    tag = "Documents",
    params(
        ("track_id" = String, Path, description = "Track ID for the batch")
    ),
    responses(
        (status = 200, description = "Track status retrieved", body = TrackStatusResponse),
        (status = 404, description = "Track not found")
    )
)]
pub async fn get_track_status(
    State(state): State<AppState>,
    axum::extract::Path(track_id): axum::extract::Path<String>,
) -> ApiResult<Json<TrackStatusResponse>> {
    let keys = state.kv_storage.keys().await?;

    // Find all metadata keys
    let metadata_keys: Vec<String> = keys
        .iter()
        .filter(|k| k.ends_with("-metadata"))
        .cloned()
        .collect();

    // Fetch all metadata
    let metadata_values = state.kv_storage.get_by_ids(&metadata_keys).await?;

    // Group chunks by document
    let mut doc_chunks: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for key in &keys {
        if let Some(doc_id) = key.split("-chunk-").next() {
            if !doc_id.ends_with("-metadata") && !doc_id.ends_with("-content") {
                *doc_chunks.entry(doc_id.to_string()).or_default() += 1;
            }
        }
    }

    // Filter documents by track_id
    let mut track_docs: Vec<DocumentSummary> = Vec::new();
    let mut created_times: Vec<String> = Vec::new();

    for value in metadata_values {
        if let Some(obj) = value.as_object() {
            let doc_track_id = obj.get("track_id").and_then(|v| v.as_str()).unwrap_or("");

            if doc_track_id == track_id {
                let id = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let chunk_count = doc_chunks.get(&id).copied().unwrap_or(0);

                if let Some(created_at) = obj.get("created_at").and_then(|v| v.as_str()) {
                    created_times.push(created_at.to_string());
                }

                track_docs.push(DocumentSummary {
                    id,
                    title: obj.get("title").and_then(|v| v.as_str()).map(String::from),
                    file_name: obj
                        .get("file_name")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    content_summary: obj
                        .get("content_summary")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    content_length: obj
                        .get("content_length")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize),
                    chunk_count,
                    entity_count: obj
                        .get("entity_count")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize),
                    status: obj.get("status").and_then(|v| v.as_str()).map(String::from),
                    error_message: obj
                        .get("error_message")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    track_id: Some(track_id.clone()),
                    created_at: obj
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    updated_at: obj
                        .get("updated_at")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    cost_usd: obj.get("cost_usd").and_then(|v| v.as_f64()),
                    input_tokens: obj
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize),
                    output_tokens: obj
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize),
                    total_tokens: obj
                        .get("total_tokens")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize),
                    llm_model: obj
                        .get("llm_model")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    embedding_model: obj
                        .get("embedding_model")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    // SPEC-002: Unified Ingestion Pipeline fields
                    source_type: obj
                        .get("source_type")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    current_stage: obj
                        .get("current_stage")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    stage_progress: obj
                        .get("stage_progress")
                        .and_then(|v| v.as_f64())
                        .map(|n| n as f32),
                    stage_message: obj
                        .get("stage_message")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    pdf_id: obj.get("pdf_id").and_then(|v| v.as_str()).map(String::from),
                });
            }
        }
    }

    // Calculate status summary (handle empty track gracefully - documents may still be processing)
    let status_summary = StatusCounts {
        pending: track_docs
            .iter()
            .filter(|d| d.status.as_deref() == Some("pending"))
            .count(),
        processing: track_docs
            .iter()
            .filter(|d| d.status.as_deref() == Some("processing"))
            .count(),
        completed: track_docs
            .iter()
            .filter(|d| {
                d.status.is_none()
                    || d.status.as_deref() == Some("completed")
                    || d.status.as_deref() == Some("indexed")
            })
            .count(),
        // FIX-5: Track partial_failure status
        partial_failure: track_docs
            .iter()
            .filter(|d| d.status.as_deref() == Some("partial_failure"))
            .count(),
        failed: track_docs
            .iter()
            .filter(|d| d.status.as_deref() == Some("failed"))
            .count(),
        cancelled: track_docs
            .iter()
            .filter(|d| d.status.as_deref() == Some("cancelled"))
            .count(),
    };

    // Find earliest created_at
    created_times.sort();
    let created_at = created_times.first().cloned();

    // Check if complete (no pending or processing)
    let is_complete = status_summary.pending == 0 && status_summary.processing == 0;

    // Build latest message
    let latest_message = if !is_complete {
        Some(format!(
            "Processing {}/{} documents...",
            status_summary.completed + status_summary.failed,
            track_docs.len()
        ))
    } else if status_summary.failed > 0 {
        Some(format!("Completed with {} errors", status_summary.failed))
    } else {
        Some("All documents processed successfully".to_string())
    };

    Ok(Json(TrackStatusResponse {
        track_id,
        created_at,
        documents: track_docs.clone(),
        total_count: track_docs.len(),
        status_summary,
        is_complete,
        latest_message,
    }))
}

// ============================================
// GAP-014: Document Scan API
// ============================================

/// Scan a directory and queue documents for processing.
///
/// SECURITY (OODA-248): Path traversal protection.
/// User-provided paths are validated against allowed directories.
#[utoipa::path(
    post,
    path = "/api/v1/documents/scan",
    tag = "Documents",
    request_body = ScanDirectoryRequest,
    responses(
        (status = 200, description = "Directory scanned successfully", body = ScanDirectoryResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Path not allowed"),
        (status = 404, description = "Directory not found")
    )
)]
pub async fn scan_directory(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Json(request): Json<ScanDirectoryRequest>,
) -> ApiResult<Json<ScanDirectoryResponse>> {
    debug!(
        "scan_directory called with tenant context: tenant_id={:?}, workspace_id={:?}",
        tenant_ctx.tenant_id, tenant_ctx.workspace_id
    );

    // SECURITY (OODA-248): Validate path against allowed directories.
    // WHY: Prevents directory traversal attacks (e.g., ../../../etc/passwd).
    let validated_path =
        crate::path_validation::validate_path(&request.path, &state.path_validation_config)?;

    let base_path = &validated_path.canonical;

    // Path is already validated to exist by validate_path
    if !base_path.is_dir() {
        return Err(ApiError::BadRequest(format!(
            "Path is not a directory: {}",
            request.path
        )));
    }

    // Generate track ID
    let track_id = request.track_id.unwrap_or_else(|| {
        format!(
            "scan_{}_{}",
            Utc::now().format("%Y%m%d_%H%M%S"),
            &Uuid::new_v4().to_string()[..8]
        )
    });

    let mut queued_files = Vec::new();
    let mut skipped_files = Vec::new();
    let mut files_found = 0;

    // Collect files to process
    let entries = collect_files(base_path, request.recursive, request.max_files)?;

    for entry in entries {
        files_found += 1;

        let file_path = entry.path();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Check extension filter
        if !request.extensions.is_empty() {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if !request
                    .extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(ext))
                {
                    skipped_files.push(SkippedFile {
                        path: file_path.display().to_string(),
                        reason: format!("Extension .{} not in filter list", ext),
                    });
                    continue;
                }
            } else {
                skipped_files.push(SkippedFile {
                    path: file_path.display().to_string(),
                    reason: "No extension".to_string(),
                });
                continue;
            }
        }

        // Try to read file content
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                skipped_files.push(SkippedFile {
                    path: file_path.display().to_string(),
                    reason: format!("Failed to read: {}", e),
                });
                continue;
            }
        };

        if content.trim().is_empty() {
            skipped_files.push(SkippedFile {
                path: file_path.display().to_string(),
                reason: "Empty file".to_string(),
            });
            continue;
        }

        // Check size limit
        if content.len() > state.config.max_document_size {
            skipped_files.push(SkippedFile {
                path: file_path.display().to_string(),
                reason: format!(
                    "Exceeds max size ({} > {})",
                    content.len(),
                    state.config.max_document_size
                ),
            });
            continue;
        }

        // Generate document ID
        let document_id = Uuid::new_v4().to_string();

        // Generate content summary
        let content_summary = crate::validation::generate_content_summary(&content);

        // Store document metadata
        let doc_metadata_key = format!("{}-metadata", document_id);
        let doc_metadata = serde_json::json!({
            "id": document_id,
            "title": file_name,
            "file_path": file_path.display().to_string(),
            "content_summary": content_summary,
            "content_length": content.len(),
            "track_id": track_id,
            "created_at": Utc::now().to_rfc3339(),
            "status": "pending",
        });
        state
            .kv_storage
            .upsert(&[(doc_metadata_key, doc_metadata)])
            .await?;

        // Store document content
        let doc_content_key = format!("{}-content", document_id);
        let doc_content = serde_json::json!({
            "content": content,
        });
        state
            .kv_storage
            .upsert(&[(doc_content_key, doc_content)])
            .await?;

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
                text: content,
                file_source: file_path.display().to_string(),
                workspace_id: workspace_id.clone(),
                metadata: Some(serde_json::json!({
                    "document_id": document_id,
                    "title": file_name,
                    "track_id": track_id,
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

            state
                .task_storage
                .create_task(&task)
                .await
                .map_err(|e| ApiError::Internal(format!("Failed to create task: {}", e)))?;

            state
                .task_queue
                .send(task)
                .await
                .map_err(|e| ApiError::Internal(format!("Failed to queue task: {}", e)))?;
        }

        queued_files.push(file_path.display().to_string());
    }

    Ok(Json(ScanDirectoryResponse {
        track_id,
        files_found,
        files_queued: queued_files.len(),
        files_skipped: skipped_files.len(),
        queued_files,
        skipped_files,
    }))
}

/// Collect files from a directory.
fn collect_files(
    path: &std::path::Path,
    recursive: bool,
    max_files: usize,
) -> Result<Vec<std::fs::DirEntry>, ApiError> {
    let mut files = Vec::new();

    fn visit_dir(
        dir: &std::path::Path,
        recursive: bool,
        max_files: usize,
        files: &mut Vec<std::fs::DirEntry>,
    ) -> Result<(), ApiError> {
        if files.len() >= max_files {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).map_err(|e| {
            ApiError::Internal(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;

        for entry in entries {
            if files.len() >= max_files {
                break;
            }

            let entry = entry.map_err(|e| {
                ApiError::Internal(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            if path.is_file() {
                files.push(entry);
            } else if path.is_dir() && recursive {
                visit_dir(&path, recursive, max_files, files)?;
            }
        }

        Ok(())
    }

    visit_dir(path, recursive, max_files, &mut files)?;
    Ok(files)
}
