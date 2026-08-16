//! Query entry points using workspace-specific vector storage.
//!
//! Contains: `query_with_workspace_config`, `query_with_full_config`,
//!           `query_with_vector_storage`.

use std::sync::Arc;

use crate::error::Result;
use edgequake_storage::traits::VectorStorage;

use super::super::QueryEngine;
use super::query_pipeline::QueryProviders;

impl QueryEngine {
    /// Execute a query with workspace-specific vector storage (default embedding).
    ///
    /// @implements SPEC-033 (Workspace vector isolation)
    pub async fn query_with_vector_storage(
        &self,
        request: crate::types::QueryRequest,
        vector_storage: Arc<dyn VectorStorage>,
    ) -> Result<crate::types::QueryResponse> {
        self.query_with_workspace_config(request, self.embedding_provider.clone(), vector_storage)
            .await
    }

    /// Execute a query with workspace-specific embedding and vector storage.
    ///
    /// @implements SPEC-033: Full workspace isolation for vector storage.
    pub async fn query_with_workspace_config(
        &self,
        request: crate::types::QueryRequest,
        embedding_provider: Arc<dyn crate::EmbeddingProvider>,
        vector_storage: Arc<dyn VectorStorage>,
    ) -> Result<crate::types::QueryResponse> {
        self.query_with_role_llms(request, embedding_provider, vector_storage, None, None)
            .await
    }

    /// Execute a query with full workspace configuration AND optional LLM overrides.
    ///
    /// When `keyword_llm` is set (SPEC-046 / LightRAG KEYWORD role), keyword
    /// extraction uses that provider; answer generation uses `answer_llm`.
    ///
    /// @implements SPEC-032, SPEC-033, OODA-228, SPEC-046 EQ-046-13
    pub async fn query_with_full_config(
        &self,
        request: crate::types::QueryRequest,
        embedding_provider: Arc<dyn crate::EmbeddingProvider>,
        vector_storage: Arc<dyn VectorStorage>,
        llm_provider: Option<Arc<dyn crate::LLMProvider>>,
    ) -> Result<crate::types::QueryResponse> {
        self.query_with_role_llms(
            request,
            embedding_provider,
            vector_storage,
            llm_provider.clone(),
            llm_provider,
        )
        .await
    }

    /// Execute with separate Keyword vs Query role LLMs (SPEC-046 EQ-046-13).
    pub async fn query_with_role_llms(
        &self,
        request: crate::types::QueryRequest,
        embedding_provider: Arc<dyn crate::EmbeddingProvider>,
        vector_storage: Arc<dyn VectorStorage>,
        keyword_llm: Option<Arc<dyn crate::LLMProvider>>,
        answer_llm: Option<Arc<dyn crate::LLMProvider>>,
    ) -> Result<crate::types::QueryResponse> {
        let embedding = self.cached_embedding_for(embedding_provider);
        self.run_query_pipeline(
            request,
            QueryProviders {
                embedding: embedding.as_ref(),
                vector_storage: Some(&vector_storage),
                keyword_llm,
                answer_llm,
            },
        )
        .await
    }
}
