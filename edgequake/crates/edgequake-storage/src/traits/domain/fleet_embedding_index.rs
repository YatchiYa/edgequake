//! SPEC-091 IW2 — typed fleet embedding port (entity/relationship/report).

use async_trait::async_trait;

use crate::embedding_family::EmbeddingFamily;
use crate::error::StorageError;

use super::types::{
    EmbeddingCapabilities, FleetEmbeddingRow, ModelId, ScoredFleet, UpsertReport, VectorQuery,
    WorkspaceId,
};

#[async_trait]
pub trait FleetEmbeddingIndex: Send + Sync {
    fn capabilities(&self, family: EmbeddingFamily) -> EmbeddingCapabilities;

    async fn upsert_batch(
        &self,
        family: EmbeddingFamily,
        _model: ModelId,
        rows: &[FleetEmbeddingRow],
    ) -> Result<UpsertReport, StorageError>;

    async fn search(
        &self,
        family: EmbeddingFamily,
        req: &VectorQuery,
    ) -> Result<Vec<ScoredFleet>, StorageError>;

    async fn delete_for_workspace(
        &self,
        family: EmbeddingFamily,
        workspace: WorkspaceId,
    ) -> Result<u64, StorageError>;

    /// Dual-write hook: mirror freshly upserted legacy vector rows into typed
    /// fleet tables. Default no-op for non-Postgres adapters.
    async fn mirror_legacy_batch(
        &self,
        rows: &[(String, Vec<f32>, serde_json::Value)],
        count_as_entities: bool,
    ) -> Result<u64, StorageError> {
        let _ = (rows, count_as_entities);
        Ok(0)
    }
}
