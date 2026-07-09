//! Text embedding port for storage-side indexing (SPEC-046 EQ-046-11).
//!
//! First principle / DIP: storage must not depend on `edgequake-llm`. Callers
//! adapt their embedding provider to this trait at the composition root.

use async_trait::async_trait;

use crate::error::Result;

/// Minimal embedder used by community-report (and similar) vector indexing.
#[async_trait]
pub trait TextEmbedder: Send + Sync {
    /// Embed a batch of texts; output length must match input length.
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
