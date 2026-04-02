//! Knowledge injection handlers — PUT, GET, LIST, DELETE.
//!
//! @implements SPEC-0002 (Knowledge Injection for Enhanced Search)
//!
//! Injection entries are stored in KV as `injection::{workspace_id}::{injection_id}-metadata`
//! and processed through the standard pipeline with `source_type = "injection"` tagging.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

pub use super::injection_types::*;

/// Stable document ID prefix for injection artifacts.
fn injection_doc_id(workspace_id: &str, injection_id: &str) -> String {
    format!("injection::{}::{}", workspace_id, injection_id)
}

/// KV metadata key for an injection entry.
fn injection_meta_key(workspace_id: &str, injection_id: &str) -> String {
    format!("injection::{}::{}-metadata", workspace_id, injection_id)
}

// ============================================================================
// PUT /api/v1/workspaces/:workspace_id/injection  — Create or replace
// ============================================================================

/// Create or update a knowledge injection entry.
///
/// Processes the content through the standard pipeline with `source_type = "injection"` tagging.
/// Injection entities enrich the knowledge graph but are excluded from query source citations.
#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/injection",
    tag = "Knowledge Injection",
    request_body = PutInjectionRequest,
    responses(
        (status = 202, description = "Injection accepted for processing", body = PutInjectionResponse),
        (status = 400, description = "Invalid request"),
        (status = 413, description = "Content too large")
    )
)]
pub async fn put_injection(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Json(request): Json<PutInjectionRequest>,
) -> ApiResult<(StatusCode, Json<PutInjectionResponse>)> {
    let workspace_id = tenant_ctx
        .workspace_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "default".to_string());

    // Validate name
    let name = request.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError::BadRequest(
            "Name must be between 1 and 100 characters".to_string(),
        ));
    }

    // Validate content size
    if request.content.len() > MAX_INJECTION_CONTENT_BYTES {
        return Err(ApiError::BadRequest(format!(
            "Injection content exceeds {}KB limit",
            MAX_INJECTION_CONTENT_BYTES / 1024
        )));
    }

    if request.content.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Injection content cannot be empty".to_string(),
        ));
    }

    let injection_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let doc_id = injection_doc_id(&workspace_id, &injection_id);

    // Build metadata record
    let meta = serde_json::json!({
        "id": injection_id,
        "name": name,
        "content": request.content,
        "workspace_id": workspace_id,
        "source_type": "text",
        "status": "processing",
        "version": 1,
        "entity_count": 0,
        "source_document_id": doc_id,
        "created_at": now,
        "updated_at": now,
    });

    // Store in KV
    let meta_key = injection_meta_key(&workspace_id, &injection_id);
    state
        .kv_storage
        .upsert(&[(meta_key.clone(), meta.clone())])
        .await?;

    info!(
        workspace_id = %workspace_id,
        injection_id = %injection_id,
        content_len = request.content.len(),
        "Created knowledge injection entry"
    );

    // Process through pipeline in background
    let pipeline = state.pipeline.clone();
    let graph_storage = state.graph_storage.clone();
    let vector_storage = state.vector_storage.clone();
    let kv_storage = state.kv_storage.clone();
    let content = request.content.clone();
    let doc_id_clone = doc_id.clone();
    let injection_id_clone = injection_id.clone();
    let workspace_id_clone = workspace_id.clone();
    let meta_key_clone = meta_key.clone();
    let now_clone = now.clone();
    let name_clone = name.clone();
    let content_clone = content.clone();
    let tenant_id = tenant_ctx.tenant_id.map(|id| id.to_string());

    tokio::spawn(async move {
        match process_injection_pipeline(
            &pipeline,
            graph_storage,
            vector_storage,
            &doc_id_clone,
            &content,
            &workspace_id_clone,
            tenant_id,
        )
        .await
        {
            Ok((entity_count, chunk_ids)) => {
                let updated_meta = serde_json::json!({
                    "id": injection_id_clone,
                    "name": name_clone,
                    "content": content_clone,
                    "workspace_id": workspace_id_clone,
                    "source_type": "text",
                    "status": "completed",
                    "version": 1,
                    "entity_count": entity_count,
                    "chunk_ids": chunk_ids,
                    "source_document_id": doc_id_clone,
                    "created_at": now_clone,
                    "updated_at": Utc::now().to_rfc3339(),
                });
                let _ = kv_storage
                    .upsert(&[(meta_key_clone.clone(), updated_meta)])
                    .await;
                info!(
                    injection_id = %injection_id_clone,
                    entity_count,
                    "Injection processing completed"
                );
            }
            Err(e) => {
                warn!(
                    injection_id = %injection_id_clone,
                    error = %e,
                    "Injection processing failed"
                );
                let failed_meta = serde_json::json!({
                    "id": injection_id_clone,
                    "name": name_clone,
                    "content": content_clone,
                    "workspace_id": workspace_id_clone,
                    "source_type": "text",
                    "status": "failed",
                    "version": 1,
                    "entity_count": 0,
                    "source_document_id": doc_id_clone,
                    "error": e.to_string(),
                    "created_at": now_clone,
                    "updated_at": Utc::now().to_rfc3339(),
                });
                let _ = kv_storage
                    .upsert(&[(meta_key_clone.clone(), failed_meta)])
                    .await;
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(PutInjectionResponse {
            injection_id,
            workspace_id,
            version: 1,
            status: "processing".to_string(),
        }),
    ))
}

// ============================================================================
// GET /api/v1/workspaces/:workspace_id/injections  — List all
// ============================================================================

/// List all injection entries for a workspace.
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/injections",
    tag = "Knowledge Injection",
    responses(
        (status = 200, description = "Injection entries listed", body = ListInjectionsResponse)
    )
)]
pub async fn list_injections(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
) -> ApiResult<Json<ListInjectionsResponse>> {
    let workspace_id = tenant_ctx
        .workspace_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "default".to_string());

    let prefix = format!("injection::{}", workspace_id);
    let keys = state.kv_storage.keys().await?;

    let mut items = Vec::new();
    for key in keys
        .iter()
        .filter(|k| k.starts_with(&prefix) && k.ends_with("-metadata"))
    {
        if let Ok(Some(val)) = state.kv_storage.get_by_id(key).await {
            items.push(InjectionSummary {
                injection_id: val
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: val
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                entity_count: val
                    .get("entity_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                source_type: val
                    .get("source_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text")
                    .to_string(),
                created_at: val
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                updated_at: val
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }

    // Sort by created_at descending (newest first)
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = items.len();
    Ok(Json(ListInjectionsResponse { items, total }))
}

// ============================================================================
// GET /api/v1/workspaces/:workspace_id/injections/:injection_id  — Detail
// ============================================================================

/// Get a single injection entry detail.
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/injections/{injection_id}",
    tag = "Knowledge Injection",
    responses(
        (status = 200, description = "Injection detail", body = InjectionDetailResponse),
        (status = 404, description = "Injection not found")
    )
)]
pub async fn get_injection(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Path((_workspace_id_path, injection_id)): Path<(String, String)>,
) -> ApiResult<Json<InjectionDetailResponse>> {
    let workspace_id = tenant_ctx
        .workspace_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "default".to_string());

    let meta_key = injection_meta_key(&workspace_id, &injection_id);
    let val = state
        .kv_storage
        .get_by_id(&meta_key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Injection {} not found", injection_id)))?;

    Ok(Json(InjectionDetailResponse {
        injection_id: val
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        name: val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        content: val
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        version: val.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
        status: val
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        entity_count: val
            .get("entity_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        source_type: val
            .get("source_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text")
            .to_string(),
        created_at: val
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: val
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    }))
}

// ============================================================================
// DELETE /api/v1/workspaces/:workspace_id/injections/:injection_id
// ============================================================================

/// Delete an injection entry and all its artifacts.
#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{workspace_id}/injections/{injection_id}",
    tag = "Knowledge Injection",
    responses(
        (status = 200, description = "Injection deleted", body = DeleteInjectionResponse),
        (status = 404, description = "Injection not found")
    )
)]
pub async fn delete_injection(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Path((_workspace_id_path, injection_id)): Path<(String, String)>,
) -> ApiResult<Json<DeleteInjectionResponse>> {
    let workspace_id = tenant_ctx
        .workspace_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "default".to_string());

    let meta_key = injection_meta_key(&workspace_id, &injection_id);

    // Check it exists
    let _val = state
        .kv_storage
        .get_by_id(&meta_key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Injection {} not found", injection_id)))?;

    let doc_id = injection_doc_id(&workspace_id, &injection_id);

    // Delete KV entries (metadata + content + any chunks)
    let keys = state.kv_storage.keys().await?;
    let kv_ids_to_delete: Vec<String> = keys
        .into_iter()
        .filter(|k| k.starts_with(&doc_id) || *k == meta_key)
        .collect();
    if !kv_ids_to_delete.is_empty() {
        debug!(
            count = kv_ids_to_delete.len(),
            "Deleting injection KV entries"
        );
        let _ = state.kv_storage.delete(&kv_ids_to_delete).await;
    }

    // Delete vector entries using stored chunk IDs from metadata
    if let Some(chunk_ids) = _val.get("chunk_ids").and_then(|v| v.as_array()) {
        let ids: Vec<String> = chunk_ids
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if !ids.is_empty() {
            if let Err(e) = state.vector_storage.delete(&ids).await {
                warn!(error = %e, "Failed to delete injection vectors");
            }
        }
    }

    info!(
        injection_id = %injection_id,
        workspace_id = %workspace_id,
        "Deleted injection entry and artifacts"
    );

    Ok(Json(DeleteInjectionResponse {
        deleted: true,
        message: format!("Injection {} deleted", injection_id),
    }))
}

// ============================================================================
// Pipeline Processing (internal)
// ============================================================================

/// Process injection content through the standard pipeline with injection tagging.
///
/// Uses `Pipeline::process()` to chunk + extract entities, then merges into graph
/// with source_type=injection metadata so citations can be filtered.
async fn process_injection_pipeline(
    pipeline: &edgequake_pipeline::Pipeline,
    graph_storage: std::sync::Arc<dyn edgequake_storage::traits::GraphStorage>,
    vector_storage: std::sync::Arc<dyn edgequake_storage::traits::VectorStorage>,
    doc_id: &str,
    content: &str,
    workspace_id: &str,
    tenant_id: Option<String>,
) -> std::result::Result<(u32, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    use edgequake_pipeline::{KnowledgeGraphMerger, MergerConfig};

    // Process through standard pipeline
    let result = pipeline.process(doc_id, content).await?;

    let merger_config = MergerConfig::default();
    let merger = KnowledgeGraphMerger::new(merger_config, graph_storage, vector_storage.clone())
        .with_tenant_context(tenant_id, Some(workspace_id.to_string()));

    // Tag and merge entities with injection source tracking
    let mut tagged_extractions = Vec::new();
    for extraction in &result.extractions {
        let mut tagged = extraction.clone();
        for entity in &mut tagged.entities {
            entity.source_document_id = Some(doc_id.to_string());
            entity.source_file_path = Some("injection".to_string());
            if entity.source_chunk_ids.is_empty() {
                entity.source_chunk_ids = vec![format!("{}-chunk-0", doc_id)];
            }
        }
        for rel in &mut tagged.relationships {
            rel.source_document_id = Some(doc_id.to_string());
            rel.source_file_path = Some("injection".to_string());
            if rel.source_chunk_id.is_none() {
                rel.source_chunk_id = Some(format!("{}-chunk-0", doc_id));
            }
        }
        tagged_extractions.push(tagged);
    }

    let merge_stats = merger.merge(tagged_extractions).await?;
    let entity_count = (merge_stats.entities_created + merge_stats.entities_updated) as u32;

    // Store chunk embeddings in vector storage with injection metadata
    let mut stored_chunk_ids = Vec::new();
    for chunk in &result.chunks {
        if let Some(ref embedding) = chunk.embedding {
            let chunk_id = chunk.id.clone();
            let metadata = serde_json::json!({
                "type": "chunk",
                "source": "injection",
                "source_type": "injection",
                "source_document_id": doc_id,
                "source_file_path": "injection",
                "content": chunk.content.chars().take(500).collect::<String>(),
                "workspace_id": workspace_id,
            });
            if let Err(e) = vector_storage
                .upsert(&[(chunk_id.clone(), embedding.clone(), metadata)])
                .await
            {
                warn!(error = %e, "Failed to store injection chunk embedding");
            } else {
                stored_chunk_ids.push(chunk_id);
            }
        }
    }

    info!(
        entity_count,
        chunk_count = stored_chunk_ids.len(),
        "Injection pipeline processing complete"
    );
    Ok((entity_count, stored_chunk_ids))
}
