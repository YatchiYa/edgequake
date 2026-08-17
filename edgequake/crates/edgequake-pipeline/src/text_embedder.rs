//! Adapt `EmbeddingProvider` → `TextEmbedder` (SPEC-046 / DIP).
//!
//! Lives in pipeline so both API and core orchestrator share one adapter
//! without storage depending on the LLM crate.

use std::sync::Arc;

use async_trait::async_trait;
use edgequake_llm::traits::EmbeddingProvider;
use edgequake_storage::{StorageError, TextEmbedder};

/// Thin adapter so community-report indexing never imports the LLM crate.
pub struct LlmTextEmbedder {
    inner: Arc<dyn EmbeddingProvider>,
}

impl LlmTextEmbedder {
    pub fn new(inner: Arc<dyn EmbeddingProvider>) -> Self {
        Self { inner }
    }

    pub fn arc(inner: Arc<dyn EmbeddingProvider>) -> Arc<dyn TextEmbedder> {
        Arc::new(Self::new(inner))
    }
}

#[async_trait]
impl TextEmbedder for LlmTextEmbedder {
    async fn embed_texts(
        &self,
        texts: &[String],
    ) -> edgequake_storage::error::Result<Vec<Vec<f32>>> {
<<<<<<< HEAD
=======
        // SPEC-091 RM2: callers may pass already-preambled strings via
        // `contextual_chunk::embedding_input`; this adapter stays a thin DIP bridge.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        self.inner
            .embed(texts)
            .await
            .map_err(|e| StorageError::InvalidData(format!("embed texts: {e}")))
    }
}

<<<<<<< HEAD
=======
/// Apply contextual preamble when flag on (RM-AC-08). Use before embed_texts.
pub fn prepare_embedding_texts(contents: &[String], preambles: &[Option<String>]) -> Vec<String> {
    contents
        .iter()
        .enumerate()
        .map(|(i, content)| {
            let pre = preambles.get(i).and_then(|o| o.as_deref());
            crate::contextual_chunk::embedding_input(content, pre)
        })
        .collect()
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_llm::MockProvider;

    #[tokio::test]
    async fn adapts_mock_embedding_provider() {
        let emb = Arc::new(MockProvider::new()) as Arc<dyn EmbeddingProvider>;
        let adapter = LlmTextEmbedder::arc(emb);
        let out = adapter
            .embed_texts(&[String::from("hello")])
            .await
            .expect("embed");
        assert_eq!(out.len(), 1);
        assert!(!out[0].is_empty());
    }
}
