//! Bulk operation handlers for conversations.
//!
//! Implements import, bulk-delete, bulk-archive, and bulk-move.

use axum::{extract::State, Json};

use crate::error::ApiResult;
use crate::handlers::conversations_types::*;
use crate::middleware::TenantContext;
use crate::state::AppState;

/// Import conversations from localStorage.
#[utoipa::path(
    post,
    path = "/api/v1/conversations/import",
    request_body = ImportConversationsRequest,
    responses(
        (status = 200, description = "Import result", body = ImportConversationsResponse),
    ),
    tags = ["conversations"]
)]
pub async fn import_conversations(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Json(request): Json<ImportConversationsRequest>,
) -> ApiResult<Json<ImportConversationsResponse>> {
    let identity =
        super::super::postgres_user_bootstrap::resolve_conversation_identity(&state, &tenant_ctx)
            .await?;

    let result = state
        .conversation_service
        .import_conversations(identity.tenant_id, identity.user_id, request.conversations)
        .await?;

    Ok(Json(ImportConversationsResponse {
        imported: result.imported,
        failed: result.failed,
        errors: result
            .errors
            .into_iter()
            .map(|e| ImportErrorResponse {
                id: e.id,
                error: e.error,
            })
            .collect(),
    }))
}

/// Bulk delete conversations.
#[utoipa::path(
    post,
    path = "/api/v1/conversations/bulk/delete",
    request_body = BulkOperationRequest,
    responses(
        (status = 200, description = "Bulk delete result", body = BulkOperationResponse),
    ),
    tags = ["conversations"]
)]
pub async fn bulk_delete_conversations(
    State(state): State<AppState>,
    _tenant_ctx: TenantContext,
    Json(request): Json<BulkOperationRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    let affected = state
        .conversation_service
        .bulk_delete(request.conversation_ids)
        .await?;

    Ok(Json(BulkOperationResponse { affected }))
}

/// Bulk archive/unarchive conversations.
#[utoipa::path(
    post,
    path = "/api/v1/conversations/bulk/archive",
    request_body = BulkArchiveRequest,
    responses(
        (status = 200, description = "Bulk archive result", body = BulkOperationResponse),
    ),
    tags = ["conversations"]
)]
pub async fn bulk_archive_conversations(
    State(state): State<AppState>,
    _tenant_ctx: TenantContext,
    Json(request): Json<BulkArchiveRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    let affected = state
        .conversation_service
        .bulk_archive(request.conversation_ids, request.archive)
        .await?;

    Ok(Json(BulkOperationResponse { affected }))
}

/// Bulk move conversations to folder.
#[utoipa::path(
    post,
    path = "/api/v1/conversations/bulk/move",
    request_body = BulkMoveRequest,
    responses(
        (status = 200, description = "Bulk move result", body = BulkOperationResponse),
    ),
    tags = ["conversations"]
)]
pub async fn bulk_move_conversations(
    State(state): State<AppState>,
    _tenant_ctx: TenantContext,
    Json(request): Json<BulkMoveRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    let affected = state
        .conversation_service
        .bulk_move_to_folder(request.conversation_ids, request.folder_id)
        .await?;

    Ok(Json(BulkOperationResponse { affected }))
}
