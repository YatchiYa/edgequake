//! PostgreSQL implementation of document original storage.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::original_storage::*;

pub struct PostgresOriginalStorage {
    pool: PgPool,
}

impl PostgresOriginalStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DocumentOriginalStorage for PostgresOriginalStorage {
    async fn store_original(&self, request: StoreOriginalRequest) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO document_originals (
                document_id, workspace_id, filename, content_type,
                file_size_bytes, original_data
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (document_id) DO UPDATE SET
                filename = EXCLUDED.filename,
                content_type = EXCLUDED.content_type,
                file_size_bytes = EXCLUDED.file_size_bytes,
                original_data = EXCLUDED.original_data
            "#,
        )
        .bind(request.document_id)
        .bind(request.workspace_id)
        .bind(&request.filename)
        .bind(&request.content_type)
        .bind(request.original_data.len() as i64)
        .bind(&request.original_data)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to store original: {e}")))?;
        Ok(())
    }

    async fn get_original(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Option<DocumentOriginal>> {
        let row = sqlx::query_as::<_, OriginalRow>(
            r#"
            SELECT document_id, workspace_id, filename, content_type,
                   file_size_bytes, original_data, created_at
            FROM document_originals
            WHERE document_id = $1 AND workspace_id = $2
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to get original: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn delete_original(&self, workspace_id: &Uuid, document_id: &Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM document_originals WHERE document_id = $1 AND workspace_id = $2",
        )
        .bind(document_id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to delete original: {e}")))?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct OriginalRow {
    document_id: Uuid,
    workspace_id: Uuid,
    filename: String,
    content_type: String,
    file_size_bytes: i64,
    original_data: Vec<u8>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<OriginalRow> for DocumentOriginal {
    fn from(row: OriginalRow) -> Self {
        Self {
            document_id: row.document_id,
            workspace_id: row.workspace_id,
            filename: row.filename,
            content_type: row.content_type,
            file_size_bytes: row.file_size_bytes,
            original_data: row.original_data,
            created_at: row.created_at,
        }
    }
}
