//! Persist uploaded original bytes for non-PDF documents.

use serde_json::json;
use uuid::Uuid;

use edgequake_storage::{validate_original_data, StoreOriginalRequest};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

/// Whether this upload type should retain raw bytes for later download.
pub fn should_store_original(source_type: &str) -> bool {
    matches!(source_type, "image" | "file" | "markdown" | "text")
}

#[cfg(feature = "postgres")]
fn get_original_storage(
    state: &AppState,
) -> ApiResult<std::sync::Arc<dyn edgequake_storage::DocumentOriginalStorage>> {
    if state.storage.is_postgresql() {
        state
            .storage
            .validate_postgres_adapters()
            .map_err(ApiError::Internal)?;
    }
    state
        .storage
        .original_storage
        .as_ref()
        .cloned()
        .ok_or_else(|| ApiError::Internal("Original storage not initialized".into()))
}

#[cfg(not(feature = "postgres"))]
fn get_original_storage(
    _state: &AppState,
) -> ApiResult<std::sync::Arc<dyn edgequake_storage::DocumentOriginalStorage>> {
    Err(ApiError::Internal(
        "Original storage not available (postgres feature disabled)".into(),
    ))
}

async fn mark_has_original_metadata(state: &AppState, document_id: &str) -> ApiResult<()> {
    use edgequake_storage::kv_keys;

    for key in [
        kv_keys::staging_doc_metadata(document_id),
        kv_keys::doc_metadata(document_id),
    ] {
        if let Some(mut meta) = state
            .storage
            .kv_storage
            .get_by_id(&key)
            .await
            .map_err(ApiError::from)?
        {
            if let Some(obj) = meta.as_object_mut() {
                obj.insert("has_original".into(), json!(true));
                state
                    .storage
                    .kv_storage
                    .upsert(&[(key, meta)])
                    .await
                    .map_err(ApiError::from)?;
            }
        }
    }
    Ok(())
}

/// Store raw upload bytes and flag metadata for download UI.
pub async fn persist_uploaded_original(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
    filename: &str,
    content_type: &str,
    source_type: &str,
    bytes: &[u8],
) -> ApiResult<()> {
    if !should_store_original(source_type) {
        return Ok(());
    }

    validate_original_data(bytes, state.config.max_document_size)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let document_uuid = Uuid::parse_str(document_id)
        .map_err(|_| ApiError::BadRequest("Invalid document id".into()))?;
    let workspace_id = Uuid::parse_str(&tenant_ctx.workspace_id_or_default())
        .map_err(|_| ApiError::BadRequest("Invalid workspace id".into()))?;

    let storage = get_original_storage(state)?;
    storage
        .store_original(StoreOriginalRequest {
            document_id: document_uuid,
            workspace_id,
            filename: filename.to_string(),
            content_type: content_type.to_string(),
            original_data: bytes.to_vec(),
        })
        .await
        .map_err(ApiError::from)?;

    mark_has_original_metadata(state, document_id).await
}
