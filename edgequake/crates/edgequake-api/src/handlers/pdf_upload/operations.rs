use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
#[cfg(feature = "postgres")]
use crate::services::task_cancel::apply_task_row_cancel;
use crate::state::AppState;

// WHY: These imports are only used inside #[cfg(feature = "postgres")] blocks.
#[cfg(feature = "postgres")]
use super::helpers::create_pdf_processing_task;
#[cfg(feature = "postgres")]
use super::types::PdfUploadOptions;
#[cfg(feature = "postgres")]
use edgequake_storage::PdfProcessingStatus;
#[cfg(feature = "postgres")]
use tracing::info;
#[cfg(feature = "postgres")]
use uuid::Uuid;

// ============================================================================
// OODA-17: Error Recovery Endpoints
// ============================================================================

/// Response for retry/cancel operations.
///
/// OODA-17: Standard response for error recovery operations.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PdfOperationResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// The PDF ID.
    pub pdf_id: String,
    /// Human-readable message.
    pub message: String,
    /// New task ID (for retry operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

/// Retry a failed PDF processing task.
///
/// OODA-17: Re-enqueue a failed PDF for processing.
///
/// # Endpoint
///
/// `POST /api/v1/documents/pdf/{pdf_id}/retry`
///
/// # Behavior
///
/// 1. Validate PDF exists and belongs to workspace
/// 2. Check status is Failed (cannot retry Pending/Processing/Completed)
/// 3. Reset status to Pending
/// 4. Create new processing task
/// 5. Return new task ID
///
/// # Errors
///
/// - 404 if PDF not found
/// - 409 if PDF is not in Failed state
/// - 500 for internal errors
#[utoipa::path(
    post,
    path = "/api/v1/documents/pdf/{pdf_id}/retry",
    params(
        ("pdf_id" = String, Path, description = "PDF document ID")
    ),
    responses(
        (status = 200, description = "PDF retry initiated", body = PdfOperationResponse),
        (status = 404, description = "PDF not found"),
        (status = 409, description = "PDF not in retriable state"),
    ),
    security(("bearer_token" = []))
)]
#[allow(clippy::needless_return)]
pub async fn retry_pdf_processing(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(pdf_id): Path<String>,
) -> ApiResult<Json<PdfOperationResponse>> {
    // OODA-17: Retry requires postgres feature for PDF storage
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (&state, &tenant, &pdf_id);
        return Err(ApiError::Internal(
            "PDF storage requires postgres feature".to_string(),
        ));
    }

    #[cfg(feature = "postgres")]
    {
        let pdf_uuid = Uuid::parse_str(&pdf_id)
            .map_err(|_| ApiError::BadRequest("Invalid PDF ID format".to_string()))?;

        let _workspace_id = tenant
            .workspace_id_uuid()
            .ok_or_else(|| ApiError::BadRequest("Workspace ID required".to_string()))?;

        // OODA-17: Get PDF and validate state
        let pdf_storage = state
            .storage
            .pdf_storage
            .as_ref()
            .ok_or_else(|| ApiError::Internal("PDF storage not available".to_string()))?;

        let pdf = pdf_storage
            .get_pdf(&pdf_uuid)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to get PDF: {}", e)))?
            .ok_or_else(|| ApiError::NotFound(format!("PDF not found: {}", pdf_id)))?;

        // Only allow retry of failed PDFs
        if pdf.processing_status != PdfProcessingStatus::Failed {
            return Err(ApiError::Conflict(format!(
                "Cannot retry PDF with status '{}'. Only 'failed' PDFs can be retried.",
                pdf.processing_status
            )));
        }

        // OODA-17: Reset status to Pending
        pdf_storage
            .update_pdf_status(&pdf_uuid, PdfProcessingStatus::Pending)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to reset PDF status: {}", e)))?;

        // OODA-17: Create new processing task
        let options = PdfUploadOptions {
            enable_vision: true,
            vision_provider: None, // will be resolved from workspace config or server default
            vision_model: None,
            pdf_parser_backend: None,
            ..Default::default()
        };

        let enqueue = create_pdf_processing_task(
            &state,
            &tenant,
            pdf_uuid,
            &options,
            None,
            super::helpers::PdfReprocessIntent::fresh(),
            pdf.page_count,
            pdf.file_size_bytes.max(0) as u64,
        )
        .await?;

        info!(
            pdf_id = %pdf_id,
            task_id = %enqueue.track_id,
            "PDF processing retry initiated"
        );

        Ok(Json(PdfOperationResponse {
            success: true,
            pdf_id,
            message: "PDF retry initiated successfully".to_string(),
            task_id: Some(enqueue.track_id),
        }))
    }
}

/// Cancel an in-progress PDF processing task.
///
/// OODA-17: Request cancellation of an active PDF processing task.
///
/// # Endpoint
///
/// `DELETE /api/v1/documents/pdf/{pdf_id}/cancel`
///
/// # Behavior
///
/// 1. Validate PDF exists and belongs to workspace
/// 2. Check status is Processing (cannot cancel Completed/Failed)
/// 3. Request cancellation via PipelineState
/// 4. Update status to Failed with cancellation message
///
/// # Errors
///
/// - 404 if PDF not found
/// - 409 if PDF is not in cancellable state
/// - 500 for internal errors
#[utoipa::path(
    delete,
    path = "/api/v1/documents/pdf/{pdf_id}/cancel",
    params(
        ("pdf_id" = String, Path, description = "PDF document ID")
    ),
    responses(
        (status = 200, description = "PDF processing cancelled", body = PdfOperationResponse),
        (status = 404, description = "PDF not found"),
        (status = 409, description = "PDF not in cancellable state"),
    ),
    security(("bearer_token" = []))
)]
#[allow(clippy::needless_return)]
pub async fn cancel_pdf_processing(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(pdf_id): Path<String>,
) -> ApiResult<Json<PdfOperationResponse>> {
    // OODA-17: Cancel requires postgres feature for PDF storage
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (&state, &tenant, &pdf_id);
        return Err(ApiError::Internal(
            "PDF storage requires postgres feature".to_string(),
        ));
    }

    #[cfg(feature = "postgres")]
    {
        let pdf_uuid = Uuid::parse_str(&pdf_id)
            .map_err(|_| ApiError::BadRequest("Invalid PDF ID format".to_string()))?;

        let workspace_id = tenant
            .workspace_id_uuid()
            .ok_or_else(|| ApiError::BadRequest("Workspace ID required".to_string()))?;

        // OODA-17: Get PDF and validate state
        let pdf_storage = state
            .storage
            .pdf_storage
            .as_ref()
            .ok_or_else(|| ApiError::Internal("PDF storage not available".to_string()))?;

        let pdf = pdf_storage
            .get_pdf(&pdf_uuid)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to get PDF: {}", e)))?
            .ok_or_else(|| ApiError::NotFound(format!("PDF not found: {}", pdf_id)))?;

        // Allow cancel of Processing or Pending PDFs
        // WHY: Documents can get stuck in non-terminal states (e.g., "uploading",
        // "pending") after a server restart or network interruption. Previously
        // only "processing" was cancellable, leaving users unable to recover from
        // stuck states. Now Pending is also allowed so users can force-cancel
        // documents that never transitioned to processing.
        if pdf.processing_status != PdfProcessingStatus::Processing
            && pdf.processing_status != PdfProcessingStatus::Pending
        {
            return Err(ApiError::Conflict(format!(
                "Cannot cancel PDF with status '{}'. Only 'processing' or 'pending' PDFs can be cancelled.",
                pdf.processing_status
            )));
        }

        // Prefer per-task CancellationRegistry (same path as POST /tasks/{id}/cancel).
        // The legacy global pipeline_state flag is not observed by processors.
        let active_task = state
            .tasks
            .storage
            .find_active_pdf_processing_task(pdf_uuid, workspace_id)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to find PDF task: {}", e)))?;

        let mut cancelled_track_id = None;
        if let Some(task) = active_task {
            match apply_task_row_cancel(
                &state.tasks.storage,
                &state.tasks.cancellation_registry,
                &task.track_id,
            )
            .await
            {
                Ok(applied) if applied.cancelled => {
                    cancelled_track_id = Some(task.track_id.clone());
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        track_id = %task.track_id,
                        error = %e,
                        "Failed to persist cancelled PDF task status"
                    );
                }
            }
        }

        // Also flip the legacy flag for any older callers that still poll it.
        state.tasks.pipeline_state.request_cancellation().await;

        // Align PDF row with document cancel semantics (Failed = cancelled for PDF table).
        pdf_storage
            .update_pdf_status(&pdf_uuid, PdfProcessingStatus::Failed)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to update PDF status: {}", e)))?;

        info!(
            pdf_id = %pdf_id,
            track_id = ?cancelled_track_id,
            "PDF processing cancellation requested via per-task registry"
        );

        Ok(Json(PdfOperationResponse {
            success: true,
            pdf_id,
            message: "PDF processing cancellation requested".to_string(),
            task_id: cancelled_track_id,
        }))
    }
}
