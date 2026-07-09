//! In-memory original document storage for tests.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::original_storage::*;

use super::lock::map_lock_err;

#[derive(Debug, Default)]
pub struct MemoryOriginalStorage {
    originals: RwLock<HashMap<Uuid, DocumentOriginal>>,
}

impl MemoryOriginalStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DocumentOriginalStorage for MemoryOriginalStorage {
    async fn store_original(&self, request: StoreOriginalRequest) -> Result<()> {
        let document_id = request.document_id;
        let record = DocumentOriginal {
            document_id,
            workspace_id: request.workspace_id,
            filename: request.filename,
            content_type: request.content_type,
            file_size_bytes: request.original_data.len() as i64,
            original_data: request.original_data,
            created_at: Utc::now(),
        };
        self.originals
            .write()
            .map_err(map_lock_err)?
            .insert(document_id, record);
        Ok(())
    }

    async fn get_original(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Option<DocumentOriginal>> {
        let guard = self.originals.read().map_err(map_lock_err)?;
        Ok(guard.get(document_id).and_then(|record| {
            if record.workspace_id == *workspace_id {
                Some(record.clone())
            } else {
                None
            }
        }))
    }

    async fn delete_original(&self, workspace_id: &Uuid, document_id: &Uuid) -> Result<bool> {
        let mut guard = self.originals.write().map_err(map_lock_err)?;
        if let Some(record) = guard.get(document_id) {
            if record.workspace_id != *workspace_id {
                return Err(StorageError::NotFound(format!(
                    "Original not found for document {}",
                    document_id
                )));
            }
            guard.remove(document_id);
            return Ok(true);
        }
        Ok(false)
    }
}
