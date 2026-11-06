//! Extractor decorator: every extractor's output is canonicalized the same way.

use std::sync::Arc;

use async_trait::async_trait;

use super::compiled::CompiledOntology;
use crate::chunker::TextChunk;
use crate::error::Result;
use crate::extractor::{EntityExtractor, ExtractionResult};

/// Wraps any extractor (LLM, gleaning, SOTA, decision) and applies
/// [`CompiledOntology::canonicalize`] to each result.
pub struct OntologyExtractor {
    inner: Arc<dyn EntityExtractor>,
    ontology: Arc<CompiledOntology>,
}

impl OntologyExtractor {
    pub fn new(inner: Arc<dyn EntityExtractor>, ontology: Arc<CompiledOntology>) -> Self {
        Self { inner, ontology }
    }

    /// `inner` unchanged without an ontology, wrapped with one.
    pub fn wrap(
        inner: Arc<dyn EntityExtractor>,
        ontology: Option<&Arc<CompiledOntology>>,
    ) -> Arc<dyn EntityExtractor> {
        match ontology {
            Some(o) => Arc::new(Self::new(inner, o.clone())),
            None => inner,
        }
    }
}

#[async_trait]
impl EntityExtractor for OntologyExtractor {
    async fn extract(&self, chunk: &TextChunk) -> Result<ExtractionResult> {
        let mut result = self.inner.extract(chunk).await?;
        let stats = self.ontology.canonicalize(&mut result);
        if stats != Default::default() {
            tracing::debug!(
                chunk_id = %result.source_chunk_id,
                renamed = stats.renamed,
                retyped = stats.retyped,
                merged = stats.merged,
                self_loops_dropped = stats.self_loops_dropped,
                "Ontology canonicalized extraction"
            );
        }
        Ok(result)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }
}
