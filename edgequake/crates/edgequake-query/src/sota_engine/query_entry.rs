
use crate::context::QueryContext;
use crate::error::{QueryError, Result};
use crate::keywords::{ExtractedKeywords, QueryIntent};
use crate::modes::QueryMode;
use crate::truncation::balance_context;

use edgequake_storage::traits::VectorStorage;

use super::{QueryEmbeddings, SOTAQueryEngine};

impl SOTAQueryEngine {
    /// Execute a query with full SOTA pipeline.
    ///
    /// # WHY: 5-Stage Query Pipeline
    ///
    /// The query flow follows LightRAG's proven architecture:
    ///
    /// 1. **Keyword Extraction** - Extract high/low-level keywords using LLM
    ///    - WHY high-level: Relationships (e.g., "partnership", "acquired")
    ///    - WHY low-level: Entities (e.g., "Apple", "Microsoft")
    ///    - WHY caching: Same queries reuse extraction results (24h TTL)
    ///
    /// 2. **Keyword Validation** - Check keywords exist in knowledge graph
    ///    - WHY: Non-existent keywords dilute embedding computation
    ///    - Example: "STLA Medium" not in graph → drop it
    ///
    /// 3. **Mode Selection** - Choose retrieval strategy
    ///    - Local: Entities + 1-hop neighbors (specific questions)
    ///    - Global: Relationships + community summaries (broad themes)
    ///    - Hybrid: Both local + global (best quality, higher cost)
    ///    - Naive: Chunks only (keyword search fallback)
    ///
    /// 4. **Vector Retrieval** - Semantic search with mode-specific embedding
    ///    - WHY different embeddings: low_level → entity search, high_level → relationship search
    ///
    /// 5. **Token Budgeting** - Fit context within LLM limits
    ///    - WHY: LLM context windows are limited; we prioritize high-scoring content
    ///
    /// @implements FEAT0109 (SOTA Query Engine)
    pub async fn query(
        &self,
        request: crate::engine::QueryRequest,
    ) -> Result<crate::engine::QueryResponse> {
        let start = std::time::Instant::now();
        let mut stats = crate::engine::QueryStats::default();

        // Step 1: Extract keywords (with caching)
        let raw_keywords = if self.config.use_keyword_extraction {
            let kw_start = std::time::Instant::now();
            let kw = self
                .keyword_extractor
                .extract_extended(&request.query)
                .await?;
            tracing::debug!(
                query = %request.query,
                high_level = ?kw.high_level,
                low_level = ?kw.low_level,
                intent = %kw.query_intent,
                "Extracted keywords"
            );
            stats.embedding_time_ms += kw_start.elapsed().as_millis() as u64;
            kw
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        // WHY: Drop keywords with no graph matches to prevent embedding dilution
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, "Selected query mode");

        // Step 3: Compute embeddings
        let embed_start = std::time::Instant::now();
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, self.embedding_provider.as_ref())
                .await?;
        stats.embedding_time_ms += embed_start.elapsed().as_millis() as u64;

        // Step 4: Mode-specific retrieval
        let retrieval_start = std::time::Instant::now();
        let context = match mode {
            QueryMode::Local => {
                self.query_local(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive(&embeddings, request.tenant_id(), request.workspace_id())
                    .await?
            }
        };
        stats.retrieval_time_ms = retrieval_start.elapsed().as_millis() as u64;
        stats.context_tokens = context.token_count;

        // Step 4.5: Rerank chunks for improved precision
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let rerank_start = std::time::Instant::now();
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
            let rerank_time = rerank_start.elapsed().as_millis() as u64;
            tracing::debug!(rerank_time_ms = rerank_time, "Reranking completed");
            // Include rerank time in retrieval
            stats.retrieval_time_ms += rerank_time;
        }

        // Step 4.6: Sort entities by degree for importance-based ranking
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Generate answer (if not context-only)
        let (answer, generated_tokens) = if request.context_only {
            (String::new(), 0)
        } else if request.prompt_only {
            (self.build_prompt(&request.query, &final_context), 0)
        } else {
            let gen_start = std::time::Instant::now();
            let result = self.generate_answer(&request.query, &final_context).await?;
            stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
            result
        };

        stats.generated_tokens = generated_tokens;
        stats.total_time_ms = start.elapsed().as_millis() as u64;

        Ok(crate::engine::QueryResponse {
            answer,
            context: final_context,
            mode,
            stats,
        })
    }

    /// Execute a query with a workspace-specific embedding provider override.
    ///
    /// This method is used when the workspace has a different embedding configuration
    /// than the default engine provider. The override provider is used ONLY for
    /// computing query embeddings, not for document ingestion.
    ///
    /// @implements SPEC-032: Workspace-specific embedding in query process
    ///
    /// # Arguments
    ///
    /// * `request` - The query request
    /// * `embedding_provider` - The workspace-specific embedding provider
    ///
    /// # Returns
    ///
    /// Query response with answer, context, and stats.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let workspace_provider = ProviderFactory::create_embedding_provider(
    ///     "ollama", "embeddinggemma:latest", 768,
    /// )?;
    /// let response = engine.query_with_embedding_provider(request, workspace_provider).await?;
    /// ```
    pub async fn query_with_embedding_provider(
        &self,
        request: crate::engine::QueryRequest,
        embedding_provider: std::sync::Arc<dyn crate::EmbeddingProvider>,
    ) -> Result<crate::engine::QueryResponse> {
        let start = std::time::Instant::now();
        let mut stats = crate::engine::QueryStats::default();

        // Step 1: Extract keywords (with caching)
        let raw_keywords = if self.config.use_keyword_extraction {
            let kw_start = std::time::Instant::now();
            let kw = self
                .keyword_extractor
                .extract_extended(&request.query)
                .await?;
            tracing::debug!(
                query = %request.query,
                high_level = ?kw.high_level,
                low_level = ?kw.low_level,
                intent = %kw.query_intent,
                "Extracted keywords (workspace embedding)"
            );
            stats.embedding_time_ms += kw_start.elapsed().as_millis() as u64;
            kw
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, "Selected query mode (workspace embedding)");

        // Step 3: Compute embeddings using WORKSPACE-SPECIFIC provider
        let embed_start = std::time::Instant::now();
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, embedding_provider.as_ref())
                .await?;
        stats.embedding_time_ms += embed_start.elapsed().as_millis() as u64;

        // Step 4: Mode-specific retrieval (same as query method)
        let retrieval_start = std::time::Instant::now();
        let context = match mode {
            QueryMode::Local => {
                self.query_local(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive(&embeddings, request.tenant_id(), request.workspace_id())
                    .await?
            }
        };
        stats.retrieval_time_ms = retrieval_start.elapsed().as_millis() as u64;
        stats.context_tokens = context.token_count;

        // Step 4.5: Rerank chunks
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let rerank_start = std::time::Instant::now();
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
            let rerank_time = rerank_start.elapsed().as_millis() as u64;
            stats.retrieval_time_ms += rerank_time;
        }

        // Step 4.6: Sort entities by degree
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Generate answer
        let (answer, generated_tokens) = if request.context_only {
            (String::new(), 0)
        } else if request.prompt_only {
            (self.build_prompt(&request.query, &final_context), 0)
        } else {
            let gen_start = std::time::Instant::now();
            let result = self.generate_answer(&request.query, &final_context).await?;
            stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
            result
        };

        stats.generated_tokens = generated_tokens;
        stats.total_time_ms = start.elapsed().as_millis() as u64;

        Ok(crate::engine::QueryResponse {
            answer,
            context: final_context,
            mode,
            stats,
        })
    }

    /// Execute a query with workspace-specific vector storage and embedding provider.
    ///
    /// SPEC-033: Full workspace isolation for vector storage.
    ///
    /// This method enables complete workspace isolation by using:
    /// - Workspace-specific embedding provider (for computing query embeddings)
    /// - Workspace-specific vector storage (for similarity search)
    ///
    /// WHY: Different workspaces may use different embedding models with different
    /// dimensions (e.g., OpenAI 1536 vs Ollama 768). The vector storage must match
    /// the embedding dimension for correct similarity search.
    ///
    /// # Arguments
    ///
    /// * `request` - The query request
    /// * `embedding_provider` - The workspace-specific embedding provider
    /// * `vector_storage` - The workspace-specific vector storage
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ws_embedding = ProviderFactory::create_embedding_provider("ollama", "nomic-embed-text", 768)?;
    /// let ws_vector = registry.get_or_create(workspace_config).await?;
    /// let response = engine.query_with_workspace_config(request, ws_embedding, ws_vector).await?;
    /// ```
    pub async fn query_with_workspace_config(
        &self,
        request: crate::engine::QueryRequest,
        embedding_provider: std::sync::Arc<dyn crate::EmbeddingProvider>,
        vector_storage: std::sync::Arc<dyn VectorStorage>,
    ) -> Result<crate::engine::QueryResponse> {
        let start = std::time::Instant::now();
        let mut stats = crate::engine::QueryStats::default();

        // Step 1: Extract keywords (with caching)
        let raw_keywords = if self.config.use_keyword_extraction {
            let kw_start = std::time::Instant::now();
            let kw = self
                .keyword_extractor
                .extract_extended(&request.query)
                .await?;
            tracing::debug!(
                query = %request.query,
                high_level = ?kw.high_level,
                low_level = ?kw.low_level,
                intent = %kw.query_intent,
                "Extracted keywords (workspace config)"
            );
            stats.embedding_time_ms += kw_start.elapsed().as_millis() as u64;
            kw
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, "Selected query mode (workspace config)");

        // Step 3: Compute embeddings using WORKSPACE-SPECIFIC embedding provider
        let embed_start = std::time::Instant::now();
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, embedding_provider.as_ref())
                .await?;
        stats.embedding_time_ms += embed_start.elapsed().as_millis() as u64;

        // Step 4: Mode-specific retrieval using WORKSPACE-SPECIFIC vector storage
        let retrieval_start = std::time::Instant::now();
        let context = match mode {
            QueryMode::Local => {
                self.query_local_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive_with_vector_storage(
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
        };
        stats.retrieval_time_ms = retrieval_start.elapsed().as_millis() as u64;
        stats.context_tokens = context.token_count;

        tracing::debug!(
            chunks_from_retrieval = context.chunks.len(),
            entities_from_retrieval = context.entities.len(),
            "OODA-231: Context returned from mode-specific retrieval (query_with_workspace_config)"
        );

        // Step 4.5: Rerank chunks
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        tracing::debug!(
            chunks_before_rerank = context.chunks.len(),
            should_rerank = should_rerank,
            has_reranker = self.reranker.is_some(),
            "OODA-231: Before reranking step (query_with_workspace_config)"
        );
        if should_rerank && self.reranker.is_some() {
            let rerank_start = std::time::Instant::now();
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
            let rerank_time = rerank_start.elapsed().as_millis() as u64;
            stats.retrieval_time_ms += rerank_time;
        }
        tracing::debug!(
            chunks_after_rerank = context.chunks.len(),
            "OODA-231: After reranking step (query_with_workspace_config)"
        );

        // Step 4.6: Sort entities by degree
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Generate answer
        let (answer, generated_tokens) = if request.context_only {
            (String::new(), 0)
        } else if request.prompt_only {
            (self.build_prompt(&request.query, &final_context), 0)
        } else {
            let gen_start = std::time::Instant::now();
            let result = self.generate_answer(&request.query, &final_context).await?;
            stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
            result
        };

        stats.generated_tokens = generated_tokens;
        stats.total_time_ms = start.elapsed().as_millis() as u64;

        Ok(crate::engine::QueryResponse {
            answer,
            context: final_context,
            mode,
            stats,
        })
    }

    /// Execute a query with full workspace configuration AND optional LLM override.
    ///
    /// This method combines workspace-specific embedding/vector storage for retrieval
    /// with an optional LLM provider override for answer generation. This is the
    /// recommended method for chat-style interfaces where users can select a different
    /// LLM model while still using workspace-specific embeddings.
    ///
    /// @implements SPEC-032: Workspace-specific embedding in query process
    /// @implements SPEC-033: Workspace vector isolation
    /// @implements OODA-228: Fix dimension mismatch in chat handler
    ///
    /// # Arguments
    ///
    /// * `request` - The query request
    /// * `embedding_provider` - The workspace-specific embedding provider
    /// * `vector_storage` - The workspace-specific vector storage
    /// * `llm_provider` - Optional LLM provider override for answer generation
    ///
    /// # Returns
    ///
    /// Query response using workspace embeddings and optionally custom LLM.
    pub async fn query_with_full_config(
        &self,
        request: crate::engine::QueryRequest,
        embedding_provider: std::sync::Arc<dyn crate::EmbeddingProvider>,
        vector_storage: std::sync::Arc<dyn VectorStorage>,
        llm_provider: Option<std::sync::Arc<dyn crate::LLMProvider>>,
    ) -> Result<crate::engine::QueryResponse> {
        let start = std::time::Instant::now();
        let mut stats = crate::engine::QueryStats::default();

        // Step 1: Extract keywords (with caching)
        // WHY: Use extract_with_llm_override when user selected a specific LLM provider.
        // This ensures keyword extraction uses the SAME LLM as answer generation.
        // Without this, keyword extraction would use the server default (often Ollama)
        // while answer generation uses the user's choice (e.g., OpenAI GPT-4).
        // This bug caused inconsistent behavior and unexpected costs.
        let raw_keywords = if self.config.use_keyword_extraction {
            let kw_start = std::time::Instant::now();
            let kw = self
                .keyword_extractor
                .extract_with_llm_override(&request.query, llm_provider.clone())
                .await?;
            tracing::debug!(
                query = %request.query,
                high_level = ?kw.high_level,
                low_level = ?kw.low_level,
                intent = %kw.query_intent,
                has_llm_override = llm_provider.is_some(),
                "Extracted keywords (full config)"
            );
            stats.embedding_time_ms += kw_start.elapsed().as_millis() as u64;
            kw
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, "Selected query mode (full config)");

        // Step 3: Compute embeddings using WORKSPACE-SPECIFIC embedding provider
        let embed_start = std::time::Instant::now();
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, embedding_provider.as_ref())
                .await?;
        stats.embedding_time_ms += embed_start.elapsed().as_millis() as u64;

        // Step 4: Mode-specific retrieval using WORKSPACE-SPECIFIC vector storage
        let retrieval_start = std::time::Instant::now();
        let context = match mode {
            QueryMode::Local => {
                self.query_local_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive_with_vector_storage(
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
        };
        stats.retrieval_time_ms = retrieval_start.elapsed().as_millis() as u64;
        stats.context_tokens = context.token_count;

        // Step 4.5: Rerank chunks
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let rerank_start = std::time::Instant::now();
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
            let rerank_time = rerank_start.elapsed().as_millis() as u64;
            stats.retrieval_time_ms += rerank_time;
        }

        // Step 4.6: Sort entities by degree
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Generate answer using OVERRIDE LLM or default
        let (answer, generated_tokens) = if request.context_only {
            (String::new(), 0)
        } else if request.prompt_only {
            (self.build_prompt(&request.query, &final_context), 0)
        } else {
            let gen_start = std::time::Instant::now();
            let result = if let Some(ref llm) = llm_provider {
                // Use override LLM provider
                self.generate_answer_with_provider(&request.query, &final_context, Some(llm))
                    .await?
            } else {
                // Use default LLM provider
                self.generate_answer(&request.query, &final_context).await?
            };
            stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
            result
        };

        stats.generated_tokens = generated_tokens;
        stats.total_time_ms = start.elapsed().as_millis() as u64;

        Ok(crate::engine::QueryResponse {
            answer,
            context: final_context,
            mode,
            stats,
        })
    }

    /// Execute a query streaming with full config (workspace embedding + storage + optional LLM override).
    ///
    /// This is the streaming equivalent of `query_with_full_config`. It returns context first,
    /// then streams tokens from the answer generation.
    ///
    /// @implements SPEC-032: Workspace-specific embedding in query process
    /// @implements SPEC-033: Workspace vector isolation
    /// @implements OODA-228: Fix dimension mismatch in chat handler (streaming variant)
    ///
    /// # Returns
    ///
    /// Tuple of (QueryContext, QueryMode, Token stream)
    pub async fn query_stream_with_full_config(
        &self,
        request: crate::engine::QueryRequest,
        embedding_provider: std::sync::Arc<dyn crate::EmbeddingProvider>,
        vector_storage: std::sync::Arc<dyn VectorStorage>,
        llm_provider: Option<std::sync::Arc<dyn crate::LLMProvider>>,
    ) -> Result<(
        QueryContext,
        QueryMode,
        futures::stream::BoxStream<'static, Result<String>>,
    )> {
        use futures::StreamExt;

        // Step 1: Extract keywords (with caching)
        // WHY: Use extract_with_llm_override when user selected a specific LLM provider.
        // This ensures keyword extraction uses the SAME LLM as answer generation.
        // Without this, keyword extraction would use the server default (often Ollama)
        // while answer generation uses the user's choice (e.g., OpenAI GPT-4).
        let raw_keywords = if self.config.use_keyword_extraction {
            self.keyword_extractor
                .extract_with_llm_override(&request.query, llm_provider.clone())
                .await?
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, "Selected query mode (stream full config)");

        // Step 3: Compute embeddings using WORKSPACE-SPECIFIC embedding provider
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, embedding_provider.as_ref())
                .await?;

        // Step 4: Mode-specific retrieval using WORKSPACE-SPECIFIC vector storage
        let context = match mode {
            QueryMode::Local => {
                self.query_local_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix_with_vector_storage(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive_with_vector_storage(
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                    &vector_storage,
                )
                .await?
            }
        };

        // Step 4.5: Rerank chunks
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
        }

        // Step 4.6: Sort entities by degree
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Handle empty context
        if final_context.is_empty() {
            return Ok((
                final_context,
                mode,
                futures::stream::once(async {
                    Ok("I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.".to_string())
                })
                .boxed(),
            ));
        }

        // Step 7: Build prompt and stream using LLM override or default
        let prompt = self.build_prompt(&request.query, &final_context);

        // Determine which LLM provider to use for streaming
        let llm_to_use = llm_provider
            .clone()
            .or_else(|| Some(self.llm_provider.clone()));

        let stream = if let Some(ref llm) = llm_to_use {
            // Check if provider supports streaming
            if llm.supports_streaming() {
                tracing::debug!("Using streaming mode for LLM provider (full config)");
                llm.stream(&prompt)
                    .await
                    .map(|stream| stream.map(|res| res.map_err(QueryError::from)).boxed())
                    .map_err(QueryError::from)?
            } else {
                // Fallback to non-streaming and wrap in a stream
                tracing::warn!(
                    provider = llm.name(),
                    "Provider doesn't support streaming (full config), falling back to non-streaming mode"
                );

                let prompt_clone = prompt.clone();
                let llm_clone = llm.clone();

                let response = llm_clone
                    .complete(&prompt_clone)
                    .await
                    .map_err(QueryError::from)?;

                futures::stream::once(async move { Ok(response.content) }).boxed()
            }
        } else {
            return Err(QueryError::ConfigError(
                "No LLM provider available for streaming".to_string(),
            ));
        };

        tracing::debug!("Using full config for streaming response (embedding + vector storage + optional LLM override)");

        Ok((final_context, mode, stream))
    }

    /// Execute a query with an LLM provider override.
    ///
    /// This method is used when the user selects a different LLM provider/model
    /// in the query interface. The override provider is used ONLY for generating
    /// the answer, not for keyword extraction.
    ///
    /// @implements SPEC-032: Provider selection at query time
    ///
    /// # Arguments
    ///
    /// * `request` - The query request (may contain llm_provider/llm_model hints)
    /// * `llm_provider` - The LLM provider to use for answer generation
    ///
    /// # Returns
    ///
    /// Query response with answer generated using the override provider.
    pub async fn query_with_llm_provider(
        &self,
        request: crate::engine::QueryRequest,
        llm_provider: std::sync::Arc<dyn crate::LLMProvider>,
    ) -> Result<crate::engine::QueryResponse> {
        let start = std::time::Instant::now();
        let mut stats = crate::engine::QueryStats::default();

        // Step 1: Extract keywords (with caching)
        let raw_keywords = if self.config.use_keyword_extraction {
            let kw_start = std::time::Instant::now();
            let kw = self
                .keyword_extractor
                .extract_extended(&request.query)
                .await?;
            tracing::debug!(
                query = %request.query,
                high_level = ?kw.high_level,
                low_level = ?kw.low_level,
                intent = %kw.query_intent,
                "Extracted keywords (LLM override)"
            );
            stats.embedding_time_ms += kw_start.elapsed().as_millis() as u64;
            kw
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, "Selected query mode (LLM override)");

        // Step 3: Compute embeddings (uses default embedding provider)
        let embed_start = std::time::Instant::now();
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, self.embedding_provider.as_ref())
                .await?;
        stats.embedding_time_ms += embed_start.elapsed().as_millis() as u64;

        // Step 4: Mode-specific retrieval
        let retrieval_start = std::time::Instant::now();
        let context = match mode {
            QueryMode::Local => {
                self.query_local(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive(&embeddings, request.tenant_id(), request.workspace_id())
                    .await?
            }
        };
        stats.retrieval_time_ms = retrieval_start.elapsed().as_millis() as u64;
        stats.context_tokens = context.token_count;

        // Step 4.5: Rerank chunks for improved precision
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let rerank_start = std::time::Instant::now();
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
            let rerank_time = rerank_start.elapsed().as_millis() as u64;
            tracing::debug!(
                rerank_time_ms = rerank_time,
                "Reranking completed (LLM override)"
            );
            stats.retrieval_time_ms += rerank_time;
        }

        // Step 4.6: Sort entities by degree for importance-based ranking
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Generate answer using OVERRIDE LLM provider
        let (answer, generated_tokens) = if request.context_only {
            (String::new(), 0)
        } else if request.prompt_only {
            (self.build_prompt(&request.query, &final_context), 0)
        } else {
            let gen_start = std::time::Instant::now();
            // SPEC-032: Use the override LLM provider
            let result = self
                .generate_answer_with_provider(&request.query, &final_context, Some(&llm_provider))
                .await?;
            stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
            result
        };

        stats.generated_tokens = generated_tokens;
        stats.total_time_ms = start.elapsed().as_millis() as u64;

        Ok(crate::engine::QueryResponse {
            answer,
            context: final_context,
            mode,
            stats,
        })
    }

    /// Execute a streaming query with full SOTA pipeline.
    ///
    /// This method applies all SOTA enhancements (keyword extraction, adaptive mode,
    /// mode-specific retrieval) and then streams the LLM response.
    pub async fn query_stream(
        &self,
        request: crate::engine::QueryRequest,
    ) -> Result<futures::stream::BoxStream<'static, Result<String>>> {
        use futures::StreamExt;

        // Step 1: Extract keywords (with caching)
        // WHY: These methods (query, query_stream) don't have an LLM override parameter.
        // They always use the engine's default LLM provider (self.llm_provider).
        // Pass None to extract_with_llm_override to use the default LLM.
        // For workspace-specific LLM selection, use query_with_full_config or query_stream_with_full_config.
        let raw_keywords = if self.config.use_keyword_extraction {
            self.keyword_extractor
                .extract_with_llm_override(&request.query, None)
                .await?
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        // WHY: Drop keywords with no graph matches to prevent embedding dilution
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        tracing::debug!(mode = %mode, streaming = true, "Selected query mode for streaming");

        // Step 3: Compute embeddings
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, self.embedding_provider.as_ref())
                .await?;

        // Step 4: Mode-specific retrieval
        let context = match mode {
            QueryMode::Local => {
                self.query_local(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive(&embeddings, request.tenant_id(), request.workspace_id())
                    .await?
            }
        };

        // Step 4.5: Rerank chunks for improved precision (streaming version)
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
            tracing::debug!(streaming = true, "Reranking completed for streaming query");
        }

        // Step 4.6: Sort entities by degree for importance-based ranking
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        // Step 6: Handle empty context
        if final_context.is_empty() {
            return Ok(futures::stream::once(async {
                Ok("I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.".to_string())
            }).boxed());
        }

        // Step 7: Build prompt and stream response
        let prompt = self.build_prompt(&request.query, &final_context);

        // Check if provider supports streaming
        if self.llm_provider.supports_streaming() {
            self.llm_provider
                .stream(&prompt)
                .await
                .map(|stream| stream.map(|res| res.map_err(QueryError::from)).boxed())
                .map_err(QueryError::from)
        } else {
            // Fallback: Use non-streaming and convert to single-chunk stream
            tracing::warn!(
                provider = self.llm_provider.name(),
                "Provider doesn't support streaming, falling back to non-streaming mode"
            );

            let response = self
                .llm_provider
                .complete(&prompt)
                .await
                .map_err(QueryError::from)?;
            Ok(futures::stream::once(async move { Ok(response.content) }).boxed())
        }
    }

    /// Execute a streaming query and return both context and stream.
    ///
    /// This is the preferred method for UI scenarios where sources need to be
    /// displayed alongside the streaming response.
    ///
    /// Returns:
    /// - QueryContext: The retrieved entities, relationships, and chunks
    /// - QueryMode: The mode used for retrieval
    /// - BoxStream: The LLM response stream
    ///
    /// # Streaming Fallback
    ///
    /// If the default LLM provider doesn't support streaming, this method will
    /// fall back to non-streaming mode and convert the full response into a
    /// single-chunk stream. This ensures compatibility with all providers.
    ///
    /// Returns:
    /// - QueryContext: The retrieved entities, relationships, and chunks
    /// - QueryMode: The mode used for retrieval
    /// - BoxStream: The LLM response stream
    pub async fn query_stream_with_context(
        &self,
        request: crate::engine::QueryRequest,
    ) -> Result<(
        QueryContext,
        QueryMode,
        futures::stream::BoxStream<'static, Result<String>>,
    )> {
        use futures::StreamExt;

        // Step 1: Get context (this handles keywords, mode selection, retrieval, truncation)
        let (context, mode) = self.get_context(&request).await?;

        // Step 2: Handle empty context
        if context.is_empty() {
            return Ok((
                context,
                mode,
                futures::stream::once(async {
                    Ok("I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.".to_string())
                })
                .boxed(),
            ));
        }

        // Step 3: Build prompt and get stream
        let prompt = self.build_prompt(&request.query, &context);

        // Check if provider supports streaming
        let stream = if self.llm_provider.supports_streaming() {
            // Use streaming mode
            self.llm_provider
                .stream(&prompt)
                .await
                .map(|stream| stream.map(|res| res.map_err(QueryError::from)).boxed())
                .map_err(QueryError::from)?
        } else {
            // Fallback: Use non-streaming and convert to single-chunk stream
            tracing::warn!(
                provider = self.llm_provider.name(),
                "Provider doesn't support streaming, falling back to non-streaming mode"
            );

            let response = self
                .llm_provider
                .complete(&prompt)
                .await
                .map_err(QueryError::from)?;
            futures::stream::once(async move { Ok(response.content) }).boxed()
        };

        Ok((context, mode, stream))
    }

    /// Execute a streaming query with an LLM provider override.
    ///
    /// This method is used when the user selects a different LLM provider/model
    /// in the query interface. The override provider is used for streaming the answer.
    ///
    /// @implements SPEC-032: Provider selection at query time (streaming)
    ///
    /// # Arguments
    ///
    /// * `request` - The query request
    /// * `llm_provider` - The LLM provider to use for streaming the answer
    ///
    /// # Returns
    ///
    /// - QueryContext: The retrieved entities, relationships, and chunks
    /// - QueryMode: The mode used for retrieval
    /// - BoxStream: The LLM response stream using the override provider
    ///
    /// # Streaming Fallback
    ///
    /// If the provider doesn't support streaming (`supports_streaming() == false`),
    /// this method will fall back to non-streaming mode and convert the full response
    /// into a single-chunk stream. This ensures compatibility with all providers.
    pub async fn query_stream_with_context_and_llm(
        &self,
        request: crate::engine::QueryRequest,
        llm_provider: std::sync::Arc<dyn crate::LLMProvider>,
    ) -> Result<(
        QueryContext,
        QueryMode,
        futures::stream::BoxStream<'static, Result<String>>,
    )> {
        use futures::StreamExt;

        // Step 1: Get context (this handles keywords, mode selection, retrieval, truncation)
        let (context, mode) = self.get_context(&request).await?;

        // Step 2: Handle empty context
        if context.is_empty() {
            return Ok((
                context,
                mode,
                futures::stream::once(async {
                    Ok("I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.".to_string())
                })
                .boxed(),
            ));
        }

        // Step 3: Build prompt and get stream using OVERRIDE LLM provider
        let prompt = self.build_prompt(&request.query, &context);

        // SPEC-032: Check if provider supports streaming
        // If not, fall back to non-streaming mode
        let stream = if llm_provider.supports_streaming() {
            // Use streaming mode
            tracing::debug!("Using streaming mode for LLM provider override");
            llm_provider
                .stream(&prompt)
                .await
                .map(|stream| stream.map(|res| res.map_err(QueryError::from)).boxed())
                .map_err(QueryError::from)?
        } else {
            // Fallback: Use non-streaming and convert to single-chunk stream
            tracing::warn!(
                provider = llm_provider.name(),
                "Provider doesn't support streaming, falling back to non-streaming mode"
            );

            // Clone prompt for the async block
            let prompt_clone = prompt.clone();
            let llm_clone = llm_provider.clone();

            // Use non-streaming completion and wrap in a stream
            let response = llm_clone
                .complete(&prompt_clone)
                .await
                .map_err(QueryError::from)?;

            // Return as a single-chunk stream
            futures::stream::once(async move { Ok(response.content) }).boxed()
        };

        tracing::debug!("Using LLM provider override for response");

        Ok((context, mode, stream))
    }

    /// Get the retrieved context without generating an answer.
    ///
    /// Useful for streaming scenarios where context is sent first.
    pub async fn get_context(
        &self,
        request: &crate::engine::QueryRequest,
    ) -> Result<(QueryContext, QueryMode)> {
        // Step 1: Extract keywords (with caching)
        // WHY: These methods (query, query_stream) don't have an LLM override parameter.
        // They always use the engine's default LLM provider (self.llm_provider).
        // Pass None to extract_with_llm_override to use the default LLM.
        // For workspace-specific LLM selection, use query_with_full_config or query_stream_with_full_config.
        let raw_keywords = if self.config.use_keyword_extraction {
            self.keyword_extractor
                .extract_with_llm_override(&request.query, None)
                .await?
        } else {
            ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory)
        };

        // Step 1.5: Validate keywords against knowledge graph
        // WHY: Drop keywords with no graph matches to prevent embedding dilution
        let keywords = self.validate_keywords(&raw_keywords).await;

        // Step 2: Determine query mode
        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        // Step 3: Compute embeddings
        let embeddings =
            QueryEmbeddings::compute(&request.query, &keywords, self.embedding_provider.as_ref())
                .await?;

        // Step 4: Mode-specific retrieval
        let context = match mode {
            QueryMode::Local => {
                self.query_local(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Global => {
                self.query_global(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Hybrid => {
                self.query_hybrid(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Mix => {
                self.query_mix(
                    &keywords,
                    &embeddings,
                    request.tenant_id(),
                    request.workspace_id(),
                )
                .await?
            }
            QueryMode::Naive => {
                self.query_naive(&embeddings, request.tenant_id(), request.workspace_id())
                    .await?
            }
        };

        // Step 4.5: Rerank chunks for improved precision
        let mut context = context;
        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                )
                .await;
            context.chunks = reranked_chunks;
        }

        // Step 4.6: Sort entities by degree for importance-based ranking
        self.sort_entities_by_degree(&mut context.entities);

        // Step 5: Apply truncation
        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &self.config.truncation,
            self.tokenizer.as_ref(),
        );

        let mut final_context = context;
        final_context.entities = truncated_entities;
        final_context.relationships = truncated_relationships;
        final_context.chunks = truncated_chunks;

        Ok((final_context, mode))
    }
}
