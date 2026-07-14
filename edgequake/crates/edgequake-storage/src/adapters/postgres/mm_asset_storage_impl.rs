//! PostgreSQL implementation of document multimodal asset storage.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::mm_asset_storage::*;

pub struct PostgresMmAssetStorage {
    pool: PgPool,
}

impl PostgresMmAssetStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DocumentMmAssetStorage for PostgresMmAssetStorage {
    async fn store_asset(&self, request: StoreMmAssetRequest) -> Result<()> {
        let path = normalize_mm_asset_path(&request.asset_path)?;
        let asset_id = normalize_mm_asset_id(&request.asset_id)?;
        validate_mm_asset_data(&request.asset_data)?;
        sqlx::query(
            r#"
            INSERT INTO document_mm_assets (
                document_id, workspace_id, asset_id, asset_path, content_type,
                file_size_bytes, asset_data, asset_kind, page_num
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (document_id, asset_path) DO UPDATE SET
                asset_id = EXCLUDED.asset_id,
                content_type = EXCLUDED.content_type,
                file_size_bytes = EXCLUDED.file_size_bytes,
                asset_data = EXCLUDED.asset_data,
                asset_kind = EXCLUDED.asset_kind,
                page_num = EXCLUDED.page_num
            "#,
        )
        .bind(request.document_id)
        .bind(request.workspace_id)
        .bind(&asset_id)
        .bind(&path)
        .bind(&request.content_type)
        .bind(request.asset_data.len() as i64)
        .bind(&request.asset_data)
        .bind(&request.asset_kind)
        .bind(request.page_num)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to store mm-asset: {e}")))?;
        Ok(())
    }

    async fn get_asset(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        asset_path: &str,
    ) -> Result<Option<DocumentMmAsset>> {
        let path = normalize_mm_asset_path(asset_path)?;
        let row = sqlx::query_as::<_, MmAssetRow>(
            r#"
            SELECT document_id, workspace_id, asset_id, asset_path, content_type,
                   file_size_bytes, asset_data, asset_kind, page_num, created_at
            FROM document_mm_assets
            WHERE document_id = $1 AND workspace_id = $2 AND asset_path = $3
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .bind(&path)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to get mm-asset: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn get_asset_by_id(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        asset_id: &str,
    ) -> Result<Option<DocumentMmAsset>> {
        let id = normalize_mm_asset_id(asset_id)?;
        let row = sqlx::query_as::<_, MmAssetRow>(
            r#"
            SELECT document_id, workspace_id, asset_id, asset_path, content_type,
                   file_size_bytes, asset_data, asset_kind, page_num, created_at
            FROM document_mm_assets
            WHERE document_id = $1 AND workspace_id = $2 AND asset_id = $3
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to get mm-asset by id: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn list_asset_paths(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT asset_path FROM document_mm_assets
            WHERE document_id = $1 AND workspace_id = $2
            ORDER BY asset_path
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to list mm-assets: {e}")))?;
        Ok(rows)
    }

    async fn list_asset_summaries(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<DocumentMmAssetSummary>> {
        let rows = sqlx::query_as::<_, MmAssetSummaryRow>(
            r#"
            SELECT asset_id, asset_path, content_type, file_size_bytes, asset_kind, page_num
            FROM document_mm_assets
            WHERE document_id = $1 AND workspace_id = $2
            ORDER BY page_num NULLS LAST, asset_path
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to list mm-asset summaries: {e}")))?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_assets_for_document(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM document_mm_assets WHERE document_id = $1 AND workspace_id = $2",
        )
        .bind(document_id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("Failed to delete mm-assets: {e}")))?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct MmAssetSummaryRow {
    asset_id: String,
    asset_path: String,
    content_type: String,
    file_size_bytes: i64,
    asset_kind: String,
    page_num: Option<i32>,
}

impl From<MmAssetSummaryRow> for DocumentMmAssetSummary {
    fn from(row: MmAssetSummaryRow) -> Self {
        Self {
            asset_id: row.asset_id,
            asset_path: row.asset_path,
            content_type: row.content_type,
            file_size_bytes: row.file_size_bytes,
            asset_kind: row.asset_kind,
            page_num: row.page_num,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MmAssetRow {
    document_id: Uuid,
    workspace_id: Uuid,
    asset_id: String,
    asset_path: String,
    content_type: String,
    file_size_bytes: i64,
    asset_data: Vec<u8>,
    asset_kind: String,
    page_num: Option<i32>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<MmAssetRow> for DocumentMmAsset {
    fn from(row: MmAssetRow) -> Self {
        Self {
            document_id: row.document_id,
            workspace_id: row.workspace_id,
            asset_id: row.asset_id,
            asset_path: row.asset_path,
            content_type: row.content_type,
            file_size_bytes: row.file_size_bytes,
            asset_data: row.asset_data,
            asset_kind: row.asset_kind,
            page_num: row.page_num,
            created_at: row.created_at,
        }
    }
}
