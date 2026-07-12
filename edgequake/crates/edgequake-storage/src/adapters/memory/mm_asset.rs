//! In-memory multimodal asset storage for tests.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;
use crate::mm_asset_storage::*;

use super::lock::map_lock_err;

#[derive(Debug, Default)]
pub struct MemoryMmAssetStorage {
    /// Key: (document_id, asset_path)
    assets: RwLock<HashMap<(Uuid, String), DocumentMmAsset>>,
}

impl MemoryMmAssetStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DocumentMmAssetStorage for MemoryMmAssetStorage {
    async fn store_asset(&self, request: StoreMmAssetRequest) -> Result<()> {
        let path = normalize_mm_asset_path(&request.asset_path)?;
        let asset_id = normalize_mm_asset_id(&request.asset_id)?;
        validate_mm_asset_data(&request.asset_data)?;
        let record = DocumentMmAsset {
            document_id: request.document_id,
            workspace_id: request.workspace_id,
            asset_id,
            asset_path: path.clone(),
            content_type: request.content_type,
            file_size_bytes: request.asset_data.len() as i64,
            asset_data: request.asset_data,
            asset_kind: request.asset_kind,
            page_num: request.page_num,
            created_at: Utc::now(),
        };
        self.assets
            .write()
            .map_err(map_lock_err)?
            .insert((request.document_id, path), record);
        Ok(())
    }

    async fn get_asset(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        asset_path: &str,
    ) -> Result<Option<DocumentMmAsset>> {
        let path = normalize_mm_asset_path(asset_path)?;
        let guard = self.assets.read().map_err(map_lock_err)?;
        Ok(guard.get(&(*document_id, path)).and_then(|record| {
            if record.workspace_id == *workspace_id {
                Some(record.clone())
            } else {
                None
            }
        }))
    }

    async fn get_asset_by_id(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        asset_id: &str,
    ) -> Result<Option<DocumentMmAsset>> {
        let id = normalize_mm_asset_id(asset_id)?;
        let guard = self.assets.read().map_err(map_lock_err)?;
        Ok(guard.values().find_map(|record| {
            if record.document_id == *document_id
                && record.workspace_id == *workspace_id
                && record.asset_id == id
            {
                Some(record.clone())
            } else {
                None
            }
        }))
    }

    async fn list_asset_paths(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<String>> {
        let guard = self.assets.read().map_err(map_lock_err)?;
        let mut paths: Vec<String> = guard
            .values()
            .filter(|r| r.document_id == *document_id && r.workspace_id == *workspace_id)
            .map(|r| r.asset_path.clone())
            .collect();
        paths.sort();
        Ok(paths)
    }

    async fn list_asset_summaries(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<DocumentMmAssetSummary>> {
        let guard = self.assets.read().map_err(map_lock_err)?;
        let mut rows: Vec<DocumentMmAssetSummary> = guard
            .values()
            .filter(|r| r.document_id == *document_id && r.workspace_id == *workspace_id)
            .map(|r| DocumentMmAssetSummary {
                asset_id: r.asset_id.clone(),
                asset_path: r.asset_path.clone(),
                content_type: r.content_type.clone(),
                file_size_bytes: r.file_size_bytes,
                asset_kind: r.asset_kind.clone(),
                page_num: r.page_num,
            })
            .collect();
        rows.sort_by(|a, b| {
            a.page_num
                .cmp(&b.page_num)
                .then_with(|| a.asset_path.cmp(&b.asset_path))
        });
        Ok(rows)
    }

    async fn delete_assets_for_document(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<u64> {
        let mut guard = self.assets.write().map_err(map_lock_err)?;
        let before = guard.len();
        guard.retain(|(doc, _), record| {
            !(doc == document_id && record.workspace_id == *workspace_id)
        });
        Ok((before - guard.len()) as u64)
    }
}
