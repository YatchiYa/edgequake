//! PostgreSQL ChunkRepository — relational chunk authority writer (SPEC-091 W1).

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageError;
use crate::traits::domain::{
    Chunk, ChunkCursor, ChunkId, ChunkRepository, ChunkText, DocumentId, InsertReport, Page,
    UnitOfWork,
};

/// Postgres adapter for `ChunkRepository::insert_batch` (idempotent via UNIQUE constraint).
pub struct PostgresChunkRepository {
    pool: PgPool,
}

/// One row to insert into `chunks` (engine backfill + repository share this).
pub(crate) struct ChunkInsertRow {
    pub document_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub chunk_index: i32,
    pub content: String,
    pub start_offset: Option<i32>,
    pub end_offset: Option<i32>,
    pub token_count: Option<i32>,
    pub metadata: serde_json::Value,
}

/// LAW-D7: single round trip per batch via `unnest` (mirrors vector upsert in
/// `storage_impl.rs`). Ids are DB-generated (`gen_random_uuid()`); the returned
/// vec holds the ids of rows actually inserted (conflict-skips omitted), so
/// callers can seed `chunk_serving_state` in the same transaction.
pub(crate) async fn insert_chunks_batch<'e, E>(
    executor: E,
    rows: &[ChunkInsertRow],
) -> Result<Vec<Uuid>, StorageError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let document_ids: Vec<Uuid> = rows.iter().map(|r| r.document_id).collect();
    let tenant_ids: Vec<Option<Uuid>> = rows.iter().map(|r| r.tenant_id).collect();
    let workspace_ids: Vec<Option<Uuid>> = rows.iter().map(|r| r.workspace_id).collect();
    let chunk_indexes: Vec<i32> = rows.iter().map(|r| r.chunk_index).collect();
    let contents: Vec<&str> = rows.iter().map(|r| r.content.as_str()).collect();
    let start_offsets: Vec<Option<i32>> = rows.iter().map(|r| r.start_offset).collect();
    let end_offsets: Vec<Option<i32>> = rows.iter().map(|r| r.end_offset).collect();
    let token_counts: Vec<Option<i32>> = rows.iter().map(|r| r.token_count).collect();
    let metadatas: Vec<&serde_json::Value> = rows.iter().map(|r| &r.metadata).collect();

    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO chunks (
            document_id, tenant_id, workspace_id, chunk_index,
            content, start_offset, end_offset, token_count, metadata
        )
        SELECT * FROM unnest(
            $1::uuid[], $2::uuid[], $3::uuid[], $4::int[],
            $5::text[], $6::int[], $7::int[], $8::int[], $9::jsonb[]
        )
        ON CONFLICT (document_id, chunk_index) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&document_ids)
    .bind(&tenant_ids)
    .bind(&workspace_ids)
    .bind(&chunk_indexes)
    .bind(&contents)
    .bind(&start_offsets)
    .bind(&end_offsets)
    .bind(&token_counts)
    .bind(&metadatas)
    .fetch_all(executor)
    .await
    .map_err(|e| StorageError::Database(format!("chunks batch insert failed: {e}")))
}

/// Upsert `chunk_serving_state` rows (migration 109). Used by the W1 write path
/// (mark `ready` after vectors+graph persisted) and the W1 backfill (legacy
/// chunks are already fully projected → `ready`).
pub(crate) async fn upsert_serving_states<'e, E>(
    executor: E,
    chunk_ids: &[Uuid],
    state: &str,
) -> Result<(), StorageError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    if chunk_ids.is_empty() {
        return Ok(());
    }
    sqlx::query(
        r#"
        INSERT INTO public.chunk_serving_state (chunk_id, state)
        SELECT id, $2 FROM unnest($1::uuid[]) AS id
        ON CONFLICT (chunk_id) DO UPDATE
        SET state = EXCLUDED.state, updated_at = now()
        "#,
    )
    .bind(chunk_ids)
    .bind(state)
    .execute(executor)
    .await
    .map_err(|e| StorageError::Database(format!("chunk_serving_state upsert failed: {e}")))?;
    Ok(())
}

impl PostgresChunkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Insert a minimal `documents` parent (write path only).
///
/// Status is `processing`: the row is a spine placeholder created at ingest
/// time, later enriched by the document lifecycle (status/title/content).
/// `ON CONFLICT (id) DO NOTHING` protects authoritative rows (PDF path,
/// backfill-adopted rows). Shared by the chunk batch writer and the typed
/// `ingestion_dedup` reservation (FK precondition).
pub(crate) async fn ensure_document_parent(
    pool: &PgPool,
    document_id: Uuid,
    tenant_id: Option<Uuid>,
    workspace_id: Option<Uuid>,
) -> Result<(), StorageError> {
    sqlx::query(
        r#"
        INSERT INTO public.documents (id, tenant_id, workspace_id, content, status)
        VALUES ($1, $2, $3, '', 'processing')
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(document_id)
    .bind(tenant_id)
    .bind(workspace_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ensure document parent failed: {e}")))?;
    Ok(())
}

/// SPEC-091 Wave B3: admission-time `documents` row carrying workspace
/// membership (SSOT for wsdoc reads). Unlike `ensure_document_parent`, a
/// pre-existing minimal row (NULL tenant/workspace, e.g. created by the chunk
/// writer) is repaired rather than left memberless — never overwrites an
/// already-set scope.
///
/// `title` is written at admission so list reads do not surface the schema
/// DEFAULT `'Untitled'` while KV still holds the real filename (markdown path).
/// Empty title falls back to `"Untitled"`. On conflict, placeholder titles
/// (`''` / `'Untitled'`) are repaired without clobbering a deliberate rename.
pub async fn ensure_admission_document_row(
    pool: &PgPool,
    document_id: Uuid,
    tenant_id: Option<Uuid>,
    workspace_id: Option<Uuid>,
    title: &str,
) -> Result<(), StorageError> {
    ensure_admission_document_row_with_track(
        pool,
        document_id,
        tenant_id,
        workspace_id,
        title,
        None,
    )
    .await
}

/// SPEC-091 Doc 23 / LAW-KVH4: admission stamps typed `documents.track_id`
/// when the progress identity is known (insert-* task id).
pub async fn ensure_admission_document_row_with_track(
    pool: &PgPool,
    document_id: Uuid,
    tenant_id: Option<Uuid>,
    workspace_id: Option<Uuid>,
    title: &str,
    track_id: Option<&str>,
) -> Result<(), StorageError> {
    let title = if title.trim().is_empty() {
        "Untitled"
    } else {
        title
    };
    sqlx::query(
        r#"
        INSERT INTO public.documents (id, tenant_id, workspace_id, title, content, status, track_id)
        VALUES ($1, $2, $3, $4, '', 'processing', $5)
        ON CONFLICT (id) DO UPDATE SET
            tenant_id = COALESCE(documents.tenant_id, EXCLUDED.tenant_id),
            workspace_id = COALESCE(documents.workspace_id, EXCLUDED.workspace_id),
            title = CASE
                WHEN documents.title IN ('', 'Untitled') THEN EXCLUDED.title
                ELSE documents.title
            END,
            track_id = COALESCE(EXCLUDED.track_id, documents.track_id)
        "#,
    )
    .bind(document_id)
    .bind(tenant_id)
    .bind(workspace_id)
    .bind(title)
    .bind(track_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ensure admission document row failed: {e}")))?;
    Ok(())
}

/// Batch variant of `ensure_document_parent` — one round trip per batch via
/// `unnest`.
async fn ensure_document_parents(pool: &PgPool, chunks: &[Chunk]) -> Result<(), StorageError> {
    let mut seen = std::collections::HashSet::new();
    let mut ids: Vec<Uuid> = Vec::new();
    let mut tenants: Vec<Option<Uuid>> = Vec::new();
    let mut workspaces: Vec<Option<Uuid>> = Vec::new();
    for c in chunks {
        if seen.insert(c.document_id.0) {
            ids.push(c.document_id.0);
            tenants.push(c.tenant_id.map(|t| t.0));
            workspaces.push(c.workspace_id.map(|w| w.0));
        }
    }
    sqlx::query(
        r#"
        INSERT INTO public.documents (id, tenant_id, workspace_id, content, status)
        SELECT id, tenant_id, workspace_id, '', 'processing'
        FROM unnest($1::uuid[], $2::uuid[], $3::uuid[]) AS t(id, tenant_id, workspace_id)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(&ids)
    .bind(&tenants)
    .bind(&workspaces)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ensure document parents failed: {e}")))?;
    Ok(())
}

#[async_trait]
impl ChunkRepository for PostgresChunkRepository {
    /// LAW-D7: single round trip per batch via the shared `insert_chunks_batch`.
    ///
    /// Write-path precondition: `chunks.document_id` FK references `documents`.
    /// Text-path admissions never create a `documents` row, so the writer
    /// ensures minimal parents first (`ON CONFLICT DO NOTHING` — a later
    /// authoritative upsert may enrich title/status; the backfill job keeps
    /// its own skip-orphan guard and does NOT adopt parents).
    async fn insert_batch(
        &self,
        _tx: &mut UnitOfWork,
        chunks: &[Chunk],
    ) -> Result<InsertReport, StorageError> {
        if chunks.is_empty() {
            return Ok(InsertReport::default());
        }

        ensure_document_parents(&self.pool, chunks).await?;

        let rows: Vec<ChunkInsertRow> = chunks
            .iter()
            .map(|c| ChunkInsertRow {
                document_id: c.document_id.0,
                tenant_id: c.tenant_id.map(|t| t.0),
                workspace_id: c.workspace_id.map(|w| w.0),
                chunk_index: c.chunk_index,
                content: c.content.clone(),
                start_offset: c.start_offset,
                end_offset: c.end_offset,
                token_count: c.token_count,
                metadata: c.metadata.clone(),
            })
            .collect();

        let inserted = insert_chunks_batch(&self.pool, &rows).await?.len() as u64;

        Ok(InsertReport {
            inserted,
            skipped: chunks.len() as u64 - inserted,
        })
    }

    async fn load_texts(&self, ids: &[ChunkId]) -> Result<Vec<ChunkText>, StorageError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let uuids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, content FROM chunks WHERE id = ANY($1)",
        )
        .bind(&uuids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(id, content)| ChunkText {
                id: ChunkId::new(id),
                content,
            })
            .collect())
    }

    async fn load_for_document(&self, document_id: DocumentId) -> Result<Vec<Chunk>, StorageError> {
        let rows = sqlx::query_as::<_, ChunkRow>(
            r#"
            SELECT id, document_id, tenant_id, workspace_id, chunk_index,
                   content, start_offset, end_offset, token_count, metadata
            FROM chunks
            WHERE document_id = $1
            ORDER BY chunk_index
            "#,
        )
        .bind(document_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(rows.into_iter().map(ChunkRow::into_chunk).collect())
    }

    async fn load_one(
        &self,
        document_id: DocumentId,
        chunk_index: i32,
    ) -> Result<Option<Chunk>, StorageError> {
        let row = sqlx::query_as::<_, ChunkRow>(
            r#"
            SELECT id, document_id, tenant_id, workspace_id, chunk_index,
                   content, start_offset, end_offset, token_count, metadata
            FROM chunks
            WHERE document_id = $1 AND chunk_index = $2
            "#,
        )
        .bind(document_id.0)
        .bind(chunk_index)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(row.map(ChunkRow::into_chunk))
    }

    async fn count_for_document(&self, document_id: DocumentId) -> Result<u64, StorageError> {
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM chunks WHERE document_id = $1")
            .bind(document_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(count.max(0) as u64)
    }

    async fn scan_from(
        &self,
        cursor: Option<ChunkCursor>,
        limit: u32,
    ) -> Result<Page<Chunk>, StorageError> {
        let limit = i64::from(limit.clamp(1, 5_000));
        let rows = if let Some(cur) = cursor {
            sqlx::query_as::<_, ChunkRow>(
                r#"
                SELECT id, document_id, tenant_id, workspace_id, chunk_index,
                       content, start_offset, end_offset, token_count, metadata
                FROM chunks
                WHERE (document_id, chunk_index) > ($1, $2)
                ORDER BY document_id, chunk_index
                LIMIT $3
                "#,
            )
            .bind(cur.document_id.0)
            .bind(cur.chunk_index)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, ChunkRow>(
                r#"
                SELECT id, document_id, tenant_id, workspace_id, chunk_index,
                       content, start_offset, end_offset, token_count, metadata
                FROM chunks
                ORDER BY document_id, chunk_index
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| StorageError::Database(e.to_string()))?;

        let full_page = rows.len() as i64 == limit;
        let items: Vec<Chunk> = rows.into_iter().map(ChunkRow::into_chunk).collect();
        let next_cursor = if full_page {
            items.last().map(|c| ChunkCursor {
                document_id: c.document_id,
                chunk_index: c.chunk_index,
            })
        } else {
            None
        };
        Ok(Page { items, next_cursor })
    }

    async fn delete_for_document(
        &self,
        _tx: &mut UnitOfWork,
        document_id: DocumentId,
    ) -> Result<u64, StorageError> {
        let result = sqlx::query("DELETE FROM chunks WHERE document_id = $1")
            .bind(document_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(result.rows_affected())
    }

    /// W4: upsert serving state for every chunk of a document (write path
    /// calls this with `ready` only after vectors + graph merge succeeded).
    async fn set_serving_state(
        &self,
        document_id: DocumentId,
        state: &str,
    ) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO public.chunk_serving_state (chunk_id, state)
            SELECT id, $2 FROM chunks WHERE document_id = $1
            ON CONFLICT (chunk_id) DO UPDATE
            SET state = EXCLUDED.state, updated_at = now()
            "#,
        )
        .bind(document_id.0)
        .bind(state)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("set_serving_state failed: {e}")))?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct ChunkRow {
    id: Uuid,
    document_id: Uuid,
    tenant_id: Option<Uuid>,
    workspace_id: Option<Uuid>,
    chunk_index: i32,
    content: String,
    start_offset: Option<i32>,
    end_offset: Option<i32>,
    token_count: Option<i32>,
    metadata: serde_json::Value,
}

impl ChunkRow {
    fn into_chunk(self) -> Chunk {
        use crate::traits::domain::{TenantId, WorkspaceId};
        Chunk {
            id: ChunkId::new(self.id),
            document_id: DocumentId(self.document_id),
            tenant_id: self.tenant_id.map(TenantId),
            workspace_id: self.workspace_id.map(WorkspaceId),
            chunk_index: self.chunk_index,
            content: self.content,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            token_count: self.token_count,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::domain::{TenantId, WorkspaceId};

    #[test]
    fn contract_spec091_single_chunk_writer_identity() {
        let doc_id = Uuid::new_v4();
        let chunk = Chunk {
            id: ChunkId::new(Uuid::new_v4()),
            document_id: DocumentId(doc_id),
            tenant_id: Some(TenantId(Uuid::new_v4())),
            workspace_id: Some(WorkspaceId(Uuid::new_v4())),
            chunk_index: 3,
            content: "spec-091".into(),
            start_offset: Some(0),
            end_offset: Some(8),
            token_count: Some(2),
            metadata: serde_json::json!({"legacy_key": "doc-chunk-3"}),
        };
        assert_eq!(chunk.document_id.0, doc_id);
        assert_eq!(chunk.chunk_index, 3);
    }
}
