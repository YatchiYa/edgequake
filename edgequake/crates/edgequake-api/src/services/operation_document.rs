//! Relational document projection for operation/task detail (SPEC-120).
//!
//! First principles: queue SSOT is Postgres `tasks`; list/detail chrome for the
//! subject document is the denormalized `documents` row (stage mirror), not KV.

use crate::handlers::tasks_types::OperationDocumentProjection;

/// Load a document projection for operation detail from Postgres.
#[cfg(feature = "postgres")]
pub async fn load_operation_document_projection(
    pool: &sqlx::PgPool,
    document_id: &str,
) -> Option<OperationDocumentProjection> {
    let Ok(uuid) = uuid::Uuid::parse_str(document_id) else {
        return None;
    };
    let row = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<f64>,
            Option<serde_json::Value>,
            Option<i32>,
            Option<String>,
        ),
    >(
        r#"
        SELECT
            id,
            title,
            status,
            metadata->>'current_stage' AS current_stage,
            metadata->>'stage_message' AS stage_message,
            CASE
              WHEN jsonb_typeof(metadata->'stage_progress') = 'number'
                THEN (metadata->>'stage_progress')::double precision
              ELSE NULL
            END AS stage_progress,
            metadata->'progress_counts' AS progress_counts,
            entity_count,
            track_id
        FROM public.documents
        WHERE id = $1
        "#,
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    Some(OperationDocumentProjection {
        id: row.0.to_string(),
        title: row.1,
        status: row.2,
        current_stage: row.3,
        stage_message: row.4,
        stage_progress: row.5,
        progress_counts: row.6.as_ref().and_then(crate::services::progress_counts_from_value),
        entity_count: row.7.map(i64::from),
        track_id: row.8,
    })
}
