//! SPEC-091 IW2 — typed fleet embedding port (entity/relationship/report).
//! SPEC-098 — mirror report with miss evidence (LAW-098-4).

use async_trait::async_trait;

use crate::embedding_family::EmbeddingFamily;
use crate::error::StorageError;

use super::types::{
    EmbeddingCapabilities, FleetEmbeddingRow, ModelId, ScoredFleet, UpsertReport, VectorQuery,
    WorkspaceId,
};

/// Result of mirroring a legacy vector batch into typed fleet tables.
///
/// `eligible` counts rows that had parseable workspace + legacy id.
/// `resolved` counts FK hits that were (or would be) upserted.
/// Typed callers fail closed when `resolved < eligible` (LAW-098-4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MirrorLegacyReport {
    pub resolved: u64,
    pub eligible: u64,
    /// Sample of legacy ids that were eligible but FK-missed (capped by caller).
    pub misses: Vec<String>,
    /// Sample of legacy ids with missing/invalid workspace_id metadata.
    pub invalid_workspace: Vec<String>,
    /// SPEC-120: lid writes absorbed because another FK already owns the stamp.
    pub absorbed_legacy_collisions: u64,
}

impl MirrorLegacyReport {
    pub const SAMPLE_CAP: usize = 5;

    pub fn push_miss(&mut self, id: &str) {
        if self.misses.len() < Self::SAMPLE_CAP {
            self.misses.push(id.to_string());
        }
    }

    pub fn push_invalid_workspace(&mut self, id: &str) {
        if self.invalid_workspace.len() < Self::SAMPLE_CAP {
            self.invalid_workspace.push(id.to_string());
        }
    }

    /// True when every eligible row resolved (or there were no eligible rows).
    pub fn is_complete(&self) -> bool {
        self.eligible == 0 || self.resolved == self.eligible
    }
}

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
    ///
    /// SPEC-130: `known_relationship_ids` maps legacy `SRC->TGT:TYPE` →
    /// `relationships.id` from the same-session sink; when present, skips name
    /// re-resolve for those keys.
    async fn mirror_legacy_batch(
        &self,
        rows: &[(String, Vec<f32>, serde_json::Value)],
        count_as_entities: bool,
        known_relationship_ids: Option<&std::collections::HashMap<String, uuid::Uuid>>,
    ) -> Result<MirrorLegacyReport, StorageError> {
        let _ = (rows, count_as_entities, known_relationship_ids);
        Ok(MirrorLegacyReport::default())
    }
}
