//! PDF document lineage lookups with PostgreSQL RLS envelope (SPEC-027 phase 42).

#[cfg(feature = "postgres")]
use uuid::Uuid;

#[cfg(feature = "postgres")]
use crate::error::ApiError;
#[cfg(feature = "postgres")]
use crate::services::tenant_isolation::{with_optional_pg_rls, PgIsolationScope};
#[cfg(feature = "postgres")]
use crate::state::ApiSecurityConfig;
#[cfg(feature = "postgres")]
use edgequake_storage::StorageError;

/// PDF extraction metadata for document detail lineage fallback.
#[cfg(feature = "postgres")]
#[derive(Debug, Clone)]
pub struct PdfExtractionMetadata {
    pub vision_model: Option<String>,
    pub extraction_method: Option<String>,
    pub extraction_warning: Option<String>,
}

/// Load PDF extraction fields from `pdf_documents` under tenant RLS when enabled.
#[cfg(feature = "postgres")]
pub async fn fetch_pdf_extraction_metadata(
    pool: &sqlx::PgPool,
    security: &ApiSecurityConfig,
    scope: Option<PgIsolationScope>,
    pdf_uuid: Uuid,
) -> Result<Option<PdfExtractionMetadata>, ApiError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        vision_model: Option<String>,
        extraction_method: Option<String>,
        extraction_errors: Option<serde_json::Value>,
        processing_status: String,
    }

    with_optional_pg_rls(pool, security, scope, move |conn| {
        Box::pin(async move {
            let row = sqlx::query_as::<_, Row>(
                r#"
                SELECT vision_model, extraction_method, extraction_errors, processing_status
                FROM pdf_documents
                WHERE pdf_id = $1
                "#,
            )
            .bind(pdf_uuid)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("pdf_documents lookup failed: {e}")))?;

            Ok(row.map(|r| {
                let extraction_warning = r
                    .extraction_errors
                    .and_then(|value| value.get("low_content_warning").cloned())
                    .and_then(|value| value.get("message").cloned())
                    .and_then(|value| value.as_str().map(str::to_string));
                let extraction_method = r.extraction_method.or_else(|| {
                    if r.processing_status == "completed" && r.vision_model.is_some() {
                        Some("vision".to_string())
                    } else {
                        None
                    }
                });
                PdfExtractionMetadata {
                    vision_model: r.vision_model,
                    extraction_method,
                    extraction_warning,
                }
            }))
        })
    })
    .await
}
