//! SPEC-091 domain ports — storage-agnostic boundaries (LD-05).

mod chunk_repository;
mod document_repository;
mod embedding_index;
mod fleet_embedding_index;
mod migration_ledger;
mod types;

pub use chunk_repository::ChunkRepository;
pub use document_repository::DocumentRepository;
pub use embedding_index::EmbeddingIndex;
pub use fleet_embedding_index::{FleetEmbeddingIndex, MirrorLegacyReport};
pub use migration_ledger::{MigrationJobSnapshot, MigrationLedger};
pub use types::{
    Chunk, ChunkCursor, ChunkId, ChunkText, DocumentId, EmbeddingCapabilities, EmbeddingRow,
    FleetEmbeddingKey, FleetEmbeddingRow, InsertReport, ModelId, Page, ScoredChunk, ScoredFleet,
    TenantId, UnitOfWork, UpsertReport, VectorQuery, WorkspaceId,
};
