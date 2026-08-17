//! SPEC-091 RM2 — typed `chunks.content_tsv` lexical search (migration 136).

/// Search typed chunks by tsquery under an optional workspace filter.
/// Returns (chunk_id, document_id, rank) ordered by rank desc.
#[cfg(feature = "postgres")]
pub async fn search_chunks_fts(
    pool: &sqlx::PgPool,
    query: &str,
    workspace_id: Option<uuid::Uuid>,
    limit: i64,
) -> Result<Vec<(uuid::Uuid, uuid::Uuid, f32)>, crate::error::StorageError> {
    let limit = limit.clamp(1, 200);
    let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, f32)>(
        r#"
        SELECT c.id, c.document_id,
               ts_rank(c.content_tsv, plainto_tsquery('english', $1))::real AS rank
        FROM public.chunks c
        JOIN public.documents d ON d.id = c.document_id
        WHERE c.content_tsv @@ plainto_tsquery('english', $1)
          AND ($2::uuid IS NULL
               OR d.workspace_id = $2
               OR (d.workspace_id IS NULL AND d.metadata->>'workspace_id' = $3))
        ORDER BY rank DESC
        LIMIT $4
        "#,
    )
    .bind(query)
    .bind(workspace_id)
    .bind(workspace_id.map(|u| u.to_string()))
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| crate::error::StorageError::Database(format!("chunk FTS: {e}")))?;
    Ok(rows)
}
