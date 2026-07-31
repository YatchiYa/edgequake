//! Cross-store persistence after pipeline compute (SPEC-021 P-G2).

mod ingestion_persister;
mod relational_chunk_writer;
pub mod typed_embedding_writer;

pub use ingestion_persister::{
    build_chunk_vector_batch, persist_processing_result, ChunkVectorBuildOptions,
    DefaultIngestionPersister, IngestionPersistConfig, IngestionPersistContext,
    IngestionPersistOutput, IngestionPersistSettings, IngestionPersister,
};
pub use relational_chunk_writer::{build_relational_chunks, persist_relational_chunks};
pub use typed_embedding_writer::persist_typed_chunk_embeddings;
