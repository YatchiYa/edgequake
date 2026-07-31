//! SPEC-091 MigrationLedger port stub — job/batch progress surfaces.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::StorageError;

/// High-level migration job snapshot (mirrors edgequake_migration_job).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationJobSnapshot {
    pub step_id: String,
    pub state: String,
    pub processed_count: i64,
    pub estimated_total: Option<i64>,
}

/// Read-only migration progress port (stub).
#[async_trait]
pub trait MigrationLedger: Send + Sync {
    async fn list_jobs(&self) -> Result<Vec<MigrationJobSnapshot>, StorageError> {
        Ok(Vec::new())
    }
}
