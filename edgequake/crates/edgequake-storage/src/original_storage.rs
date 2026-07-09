//! Original document binary storage for non-PDF uploads.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Result, StorageError};

/// Stored original upload bytes linked to a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOriginal {
    pub document_id: Uuid,
    pub workspace_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub original_data: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// Request to persist original upload bytes.
#[derive(Debug, Clone)]
pub struct StoreOriginalRequest {
    pub document_id: Uuid,
    pub workspace_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub original_data: Vec<u8>,
}

/// Storage trait for non-PDF original uploads.
#[async_trait]
pub trait DocumentOriginalStorage: Send + Sync {
    async fn store_original(&self, request: StoreOriginalRequest) -> Result<()>;

    async fn get_original(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Option<DocumentOriginal>>;

    async fn delete_original(&self, workspace_id: &Uuid, document_id: &Uuid) -> Result<bool>;
}

/// Validate original payload size before persistence.
pub fn validate_original_data(data: &[u8], max_bytes: usize) -> Result<()> {
    if data.is_empty() {
        return Err(StorageError::InvalidData(
            "Original upload data cannot be empty".into(),
        ));
    }
    if data.len() > max_bytes {
        return Err(StorageError::InvalidData(format!(
            "Original upload exceeds max size ({} > {})",
            data.len(),
            max_bytes
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_original() {
        assert!(validate_original_data(&[], 1024).is_err());
    }
}
