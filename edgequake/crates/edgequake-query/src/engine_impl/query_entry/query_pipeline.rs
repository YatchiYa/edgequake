//! Shared query pipeline phases (SPEC-017 P1-01).
//!
//! Single prepare → retrieve → finalize implementation used by all non-stream
//! query entry points. Eliminates triplicated ~600 LOC blocks across basic,
//! workspace, and LLM-override paths.

use std::sync::Arc;
use std::time::Instant;

use edgequake_storage::traits::VectorStorage;

use crate::context::QueryContext;
use crate::error::Result;
use crate::keywords::{ExtractedKeywords, QueryIntent};
use crate::modes::QueryMode;
use crate::truncation::{balance_context, truncation_config_for_intent};
use crate::types::{QueryRequest, QueryResponse, QueryStats};
use crate::{EmbeddingProvider, LLMProvider};

use super::super::{QueryEmbeddings, QueryEngine};

/// Provider bundle for one pipeline run.
pub(crate) struct QueryProviders<'a> {
    pub embedding: &'a dyn EmbeddingProvider,
    pub vector_storage: Option<&'a Arc<dyn VectorStorage>>,
    pub keyword_llm: Option<Arc<dyn LLMProvider>>,
    pub answer_llm: Option<Arc<dyn LLMProvider>>,
}

/// State after prepare phase (keywords, embeddings, mode).
pub(crate) struct PreparedQuery {
    pub keywords: ExtractedKeywords,
    pub embeddings: QueryEmbeddings,
    pub mode: QueryMode,
    /// Pure embedding wall (embed_one + keyword-level embeds). 059: not keyword LLM.
    pub embedding_time_ms: u64,
    /// Keyword extraction wall (LLM / heuristic). 059 C1b stage honesty.
    pub keyword_time_ms: u64,
    /// Effective chunk cap (`max_results` API override or config default).
    pub max_chunks: usize,
}

impl QueryEngine {
    /// Run the full non-streaming query pipeline with explicit providers.
    #[tracing::instrument(
        name = "query_pipeline",
        skip(self, request, providers),
        err(Debug, level = "warn"),
        fields(
            mode = ?request.mode,
            context_only = request.context_only,
            query_len = request.query.len(),
            error.code = tracing::field::Empty,
            error.message = tracing::field::Empty,
        )
    )]
    pub(crate) async fn run_query_pipeline(
        &self,
        request: QueryRequest,
        providers: QueryProviders<'_>,
    ) -> Result<QueryResponse> {
        let start = Instant::now();
        let mut stats = QueryStats::default();

        if request.mode.is_some_and(|m| m.is_bypass()) {
            return self
                .pipeline_finalize(
                    request,
                    QueryContext::default(),
                    QueryMode::Bypass,
                    &mut stats,
                    &providers,
                    start,
                )
                .await;
        }

        let prepared = self.pipeline_prepare(&request, &providers).await?;
        stats.embedding_time_ms = prepared.embedding_time_ms;
        stats.keyword_time_ms = prepared.keyword_time_ms;
        stats.keyword_cache_hit = self
            .keyword_cache_hit_flag
            .load(std::sync::atomic::Ordering::Relaxed);

        if request.context_only {
            if let Some(cache) = &self.result_cache {
                if let Some(cached) = cache.get(&request, prepared.mode) {
                    stats.retrieval_time_ms = 0;
                    return self
                        .pipeline_finalize(
                            request,
                            cached,
                            prepared.mode,
                            &mut stats,
                            &providers,
                            start,
                        )
                        .await;
                }
                cache.record_miss();
            }
        }

        let retrieval_start = Instant::now();
        let mut context = self
            .pipeline_retrieve(&prepared, &request, &providers)
            .await?;
        // 021: use LLM keyword-extractor intent for truncation (not flaky heuristic).
        attach_query_intent_metadata(&mut context, prepared.keywords.query_intent);
        attach_keywords_metadata(&mut context, &prepared.keywords);
        stats.retrieval_time_ms = retrieval_start.elapsed().as_millis() as u64;
        stats.context_tokens = context.token_count;
        stats.absorb_arm_metadata(&context);

        if request.context_only {
            if let Some(cache) = &self.result_cache {
                cache.put(&request, prepared.mode, context.clone());
            }
        }

        self.pipeline_finalize(
            request,
            context,
            prepared.mode,
            &mut stats,
            &providers,
            start,
        )
        .await
    }

    /// Retrieve context only (streaming / inspection paths).
    pub(crate) async fn run_context_pipeline(
        &self,
        request: &QueryRequest,
        providers: QueryProviders<'_>,
    ) -> Result<(QueryContext, QueryMode)> {
        let prepared = self.pipeline_prepare(request, &providers).await?;
        let mut context = self
            .pipeline_retrieve(&prepared, request, &providers)
            .await?;
        attach_query_intent_metadata(&mut context, prepared.keywords.query_intent);
        attach_keywords_metadata(&mut context, &prepared.keywords);
        Ok((context, prepared.mode))
    }

    /// Post-retrieval enrichment: filter, rerank, sort, truncate (no LLM generation).
    ///
    /// DRY: shared by streaming (`enrich_retrieved_context`) and non-stream finalize.
    pub(crate) async fn enrich_retrieved_context(
        &self,
        request: &QueryRequest,
        context: QueryContext,
    ) -> QueryContext {
        self.postprocess_retrieved_context(request, context, None)
            .await
    }

    /// Single post-retrieval pipeline (SPEC-046 / SOLID — one place for prune+balance).
    ///
    /// When `stats` is `Some`, records `rerank_time_ms`.
    pub(crate) async fn postprocess_retrieved_context(
        &self,
        request: &QueryRequest,
        mut context: QueryContext,
        mut stats: Option<&mut QueryStats>,
    ) -> QueryContext {
        crate::context_filter::filter_context_by_document_ids(
            &mut context,
            request.allowed_document_ids.as_deref(),
        );

        // SPEC-001 Phase 1: env-gated relevancy prune.
        // - rrf: sync keep-m on fusion scores (also applied in Mix fuse)
        // - cosine: query↔chunk embed cosine rescore (postprocess only)
        let relevancy_cfg = crate::relevancy_prune::RelevancyPruneConfig::from_env();
        if relevancy_cfg.uses_query_embed_cosine() {
            let prune_start = Instant::now();
            context = self
                .apply_query_embed_cosine_prune(&request.query, context, &relevancy_cfg)
                .await;
            if let Some(s) = stats.as_mut() {
                // Fold into rerank timing when present; otherwise leave unset.
                let ms = prune_start.elapsed().as_millis() as u64;
                s.rerank_time_ms = Some(s.rerank_time_ms.unwrap_or(0).saturating_add(ms));
            }
        } else if relevancy_cfg.enabled() {
            context = crate::relevancy_prune::apply_relevancy_prune(context, &relevancy_cfg);
        }

        // 042/048: materialize topic CONTENT chunks from KV into Mix before CE (CE_GAP).
        // 048: optional `TOPIC_MATERIALIZE_TYPES` scopes by request question_type.
        if crate::topic_entity_admit::topic_materialize_enabled()
            && crate::topic_entity_admit::topic_materialize_types_allow(request.question_type())
        {
            let topic_ids = crate::topic_entity_admit::topic_chunk_ids_from_context(&context);
            let max_n = crate::topic_entity_admit::topic_materialize_max();
            let max_mix = request
                .rerank_top_k
                .unwrap_or(self.config.rerank_top_k)
                .max(1);
            let _ = crate::topic_entity_admit::materialize_topic_chunks_into_mix(
                self.kv_storage.as_deref(),
                &mut context.chunks,
                &topic_ids,
                max_n,
                max_mix,
                &request.query,
            )
            .await;
        }

        // 026/027b: stash Mix first-stage (post-filter/prune, pre-CE) for L2 dual-list.
        let mix_pre_ce = if crate::l2_sources_union::l2_sources_union_enabled()
            || crate::l2_bm25_union::l2_bm25_union_enabled()
        {
            Some(context.chunks.clone())
        } else {
            None
        };

        let intent = intent_for_truncation(request, &context);

        // 035: Fact BM25 first-stage for protect_first (CE still ranks; no dual-list).
        if crate::intent_rerank::use_bm25_protect_for_intent(intent) && !context.chunks.is_empty() {
            let n = context.chunks.len();
            let bm25_mix =
                crate::l2_bm25_union::bm25_order_chunks(&request.query, &context.chunks, n).await;
            if !bm25_mix.is_empty() {
                tracing::debug!(
                    mix = n,
                    bm25 = bm25_mix.len(),
                    "035 Fact protect: BM25-reorder Mix before CE"
                );
                context.chunks = bm25_mix;
            }
        }

        // 027: Fact→BM25 when gated (intent from keyword metadata / heuristic).
        let prefer_bm25 = crate::intent_rerank::use_bm25_for_intent(intent);
        // 036: Exploratory coverage protect override (else global RERANK_PROTECT_FIRST).
        let protect_first = crate::intent_rerank::protect_first_for_intent(intent);

        // 039/042: CE id-protect for topic-admit / materialized ids.
        let topic_protect_ids = if crate::topic_entity_admit::topic_survival_enabled() {
            crate::topic_entity_admit::topic_chunk_ids_from_context(&context)
        } else {
            Vec::new()
        };

        let should_rerank = request.enable_rerank.unwrap_or(self.config.enable_rerank);
        if should_rerank && self.reranker.is_some() {
            let rerank_start = Instant::now();
            let reranked_chunks = self
                .rerank_chunks(
                    &request.query,
                    context.chunks,
                    request.enable_rerank,
                    request.rerank_top_k,
                    prefer_bm25,
                    protect_first,
                    &topic_protect_ids,
                )
                .await;
            context.chunks = reranked_chunks;
            if let Some(s) = stats.as_mut() {
                s.rerank_time_ms = Some(rerank_start.elapsed().as_millis() as u64);
            }
        }

        crate::entity_rank::rank_entities_for_prompt(
            &mut context.entities,
            self.config.entity_rank,
        );
        tracing::debug!(
            entity_rank = self.config.entity_rank.as_str(),
            entity_count = context.entities.len(),
            top_score = context.entities.first().map(|e| e.score).unwrap_or(0.0),
            top_degree = context.entities.first().map(|e| e.degree).unwrap_or(0),
            "Ranked entities for prompt"
        );

        if self.config.path_prune.enabled() {
            context.relationships = crate::path_prune::prune_relationships_for_query(
                std::mem::take(&mut context.relationships),
                &self.config.path_prune,
                &request.query,
            );
            if self.config.path_prune.prune_orphan_entities {
                context.entities = crate::path_prune::prune_orphan_entities(
                    std::mem::take(&mut context.entities),
                    &context.relationships,
                    &self.config.path_prune,
                );
            }
        }

        // 022 P1: graph-walk context compression (after path prune, before truncation).
        if crate::graph_walk_compress::graph_walk_compress_enabled() {
            let kw = keywords_from_context_metadata(&context);
            context = crate::graph_walk_compress::apply_graph_walk_compress(context, &kw);
        }

        let pre_e = context.entities.len();
        let pre_r = context.relationships.len();
        let pre_c = context.chunks.len();

        // 021: prefer LLM `query_intent` from keyword extraction (stashed on context
        // metadata). Heuristic is fallback only when keywords were skipped / cache miss.
        let intent = intent_for_truncation(request, &context);
        let trunc_cfg = truncation_config_for_intent(&self.config.truncation, intent);
        tracing::debug!(
            intent = %intent,
            min_chunk_budget_ratio = trunc_cfg.min_chunk_budget_ratio,
            "Truncation budget from query intent"
        );

        // 040/042: topic-admit / materialized ids pack first under greedy truncate.
        if crate::topic_entity_admit::topic_trunc_protect_enabled()
            || crate::topic_entity_admit::topic_materialize_enabled()
        {
            let topic_ids = crate::topic_entity_admit::topic_chunk_ids_from_context(&context);
            let max_n = if crate::topic_entity_admit::topic_materialize_enabled() {
                crate::topic_entity_admit::topic_materialize_max()
            } else {
                crate::topic_entity_admit::topic_trunc_protect_max()
            };
            crate::topic_entity_admit::prefer_topic_chunks_for_trunc(
                &mut context.chunks,
                &topic_ids,
                max_n,
            );
        }

        let (truncated_entities, truncated_relationships, truncated_chunks) = balance_context(
            context.entities.clone(),
            context.relationships.clone(),
            context.chunks.clone(),
            &trunc_cfg,
            self.tokenizer.as_ref(),
        );

        context.entities = truncated_entities;
        context.relationships = truncated_relationships;
        context.chunks = truncated_chunks;
        if context.entities.len() < pre_e
            || context.relationships.len() < pre_r
            || context.chunks.len() < pre_c
        {
            context.is_truncated = true;
        }
        // 026 / 027b: L2 dual-list — prompt stays `context.chunks` (CE).
        if let Some(mix) = mix_pre_ce {
            let citation = if crate::l2_bm25_union::l2_bm25_union_enabled() {
                let k = crate::l2_bm25_union::l2_bm25_mix_top_k();
                let mode = crate::l2_bm25_union::l2_bm25_mode();
                // 080 R6: Acc prompt list == Fact citations (LightRAG one-list).
                if matches!(mode, crate::l2_bm25_union::L2Bm25Mode::Unified) {
                    tracing::debug!(
                        chunks = context.chunks.len(),
                        "080 L2 Unified: citation_chunks = Acc prompt chunks"
                    );
                    context.chunks.clone()
                } else if matches!(mode, crate::l2_bm25_union::L2Bm25Mode::FactReplace) {
                    // FactReplace: BM25 L2 only for factual intents; else CE sources.
                    // LLM/metadata intent only — heuristic OR was Acc/ctx toxic
                    // (T034914Z: Fact ER 1.0 but ctx 0.44 from over-routing).
                    let intent = intent_for_truncation(request, &context);
                    let is_fact = matches!(intent, QueryIntent::Factual);
                    if !is_fact {
                        tracing::debug!(?intent, "027d L2 FactReplace: non-fact → CE sources");
                        context.chunks.clone()
                    } else {
                        let bm25_mix =
                            crate::l2_bm25_union::bm25_order_chunks(&request.query, &mix, k).await;
                        let citation = crate::l2_bm25_union::union_bm25_ce_chunks(
                            &bm25_mix,
                            &context.chunks,
                            crate::l2_bm25_union::L2Bm25Mode::Replace,
                        );
                        tracing::debug!(
                            bm25_mix = bm25_mix.len(),
                            citation = citation.len(),
                            "027d L2 FactReplace: factual → BM25 sources"
                        );
                        citation
                    }
                } else {
                    let bm25_mix =
                        crate::l2_bm25_union::bm25_order_chunks(&request.query, &mix, k).await;
                    let citation = crate::l2_bm25_union::union_bm25_ce_chunks(
                        &bm25_mix,
                        &context.chunks,
                        mode,
                    );
                    tracing::debug!(
                        mix_pre_ce = mix.len(),
                        bm25_mix = bm25_mix.len(),
                        ce_final = context.chunks.len(),
                        citation = citation.len(),
                        ?mode,
                        "027b L2 sources BM25∪CE"
                    );
                    citation
                }
            } else {
                let k = crate::l2_sources_union::l2_sources_mix_top_k();
                let citation =
                    crate::l2_sources_union::union_mix_ce_chunks(&mix, &context.chunks, k);
                tracing::debug!(
                    mix_pre_ce = mix.len(),
                    ce_final = context.chunks.len(),
                    citation = citation.len(),
                    mix_top_k = k,
                    "026 L2 sources union Mix∪CE"
                );
                citation
            };
            context.citation_chunks = Some(citation);
        }
        // X-20: stamp stable citation_id before any prompt format / API sources emit.
        context.ensure_stable_citation_ids();
        if let Some(s) = stats.as_mut() {
            s.context_truncated = context.is_truncated;
            s.context_empty = context.chunks.is_empty()
                && context.entities.is_empty()
                && context.relationships.is_empty();
        }
        context
    }

    async fn pipeline_prepare(
        &self,
        request: &QueryRequest,
        providers: &QueryProviders<'_>,
    ) -> Result<PreparedQuery> {
        let keyword_query = crate::conversation_context::query_with_conversation_context(
            &request.query,
            &request.conversation_history,
            crate::conversation_context::DEFAULT_CONVERSATION_TURN_LIMIT,
        );

        // 059: time keyword vs embed futures separately (they run in parallel).
        // 060: KEYWORD_MODE=heuristic skips keyword LLM (product latency; Acc keeps llm).
        // 083: request hl/ll override skips keyword LLM (LightRAG QueryParam law).
        let keyword_llm = providers.keyword_llm.clone();
        let keyword_mode = crate::keywords::keyword_mode_from_env();
        let keyword_override = request.keyword_override_lists();
        let (raw_keywords_result, query_vec_result) = tokio::join!(
            async {
                let t0 = Instant::now();
                let result = if !self.config.use_keyword_extraction {
                    Ok(ExtractedKeywords::new(
                        vec![],
                        vec![],
                        QueryIntent::Exploratory,
                    ))
                } else if let Some((hl, ll)) = keyword_override {
                    tracing::info!(
                        keyword_source = "request_override",
                        hl_count = hl.len(),
                        ll_count = ll.len(),
                        "083 hl/ll override — skip keyword LLM"
                    );
                    Ok(ExtractedKeywords::new(
                        hl,
                        ll,
                        QueryIntent::classify_heuristic(&keyword_query),
                    ))
                } else if matches!(keyword_mode, crate::keywords::KeywordMode::Heuristic) {
                    tracing::debug!("060 KEYWORD_MODE=heuristic — skip keyword LLM");
                    Ok(crate::keywords::heuristic_extracted_keywords(
                        &keyword_query,
                    ))
                } else if let Some(llm) = keyword_llm {
                    self.keyword_extractor
                        .extract_with_llm_override(&keyword_query, Some(llm))
                        .await
                } else {
                    self.keyword_extractor
                        .extract_extended(&keyword_query)
                        .await
                };
                (result, t0.elapsed().as_millis() as u64)
            },
            async {
                let t0 = Instant::now();
                // Skip embed_one when keywords are disabled — compute_with_query_vec
                // batch-embeds three levels (MockProvider / LightRAG parity).
                // D-38: embed the question only; conversation history stays in
                // keyword_query for prompt/keyword extraction, not the vector.
                // X-10: L2-normalize via Embedding::normalize after embed_one.
                let result = if self.config.use_keyword_extraction {
                    match providers.embedding.embed_one(&request.query).await {
                        Ok(mut vector) => {
                            // X-10: L2-normalize (same math as Embedding::normalize;
                            // local to avoid query→core dependency cycle).
                            let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                            if norm > 0.0 {
                                vector.iter_mut().for_each(|x| *x /= norm);
                            }
                            Ok(vector)
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(vec![])
                };
                (result, t0.elapsed().as_millis() as u64)
            }
        );

        let (raw_keywords, keyword_time_ms) = {
            let (r, ms) = raw_keywords_result;
            (r?, ms)
        };
        let (query_vec, mut embedding_time_ms) = {
            let (r, ms) = query_vec_result;
            (r.map_err(crate::error::QueryError::from)?, ms)
        };

        let tenant = request.tenant_id();
        let workspace = request.workspace_id();
        let keywords = self
            .validate_keywords(&raw_keywords, tenant.as_deref(), workspace.as_deref())
            .await;

        let mode = if let Some(m) = request.mode {
            m
        } else if self.config.use_adaptive_mode {
            keywords.query_intent.recommended_mode()
        } else {
            self.config.default_mode
        };

        let embed_start = Instant::now();
        let embeddings = QueryEmbeddings::compute_with_query_vec(
            &request.query,
            query_vec,
            &keywords,
            providers.embedding,
        )
        .await?;
        embedding_time_ms += embed_start.elapsed().as_millis() as u64;

        Ok(PreparedQuery {
            keywords,
            embeddings,
            mode,
            embedding_time_ms,
            keyword_time_ms,
            max_chunks: request.max_results.unwrap_or(self.config.max_chunks),
        })
    }

    async fn pipeline_retrieve(
        &self,
        prepared: &PreparedQuery,
        request: &QueryRequest,
        providers: &QueryProviders<'_>,
    ) -> Result<QueryContext> {
        use edgequake_observability::{
            query_preview, record_rag_retrieval_outcome, with_rag_retrieval_span, RagRetrievalAttrs,
        };

        let tenant = request.tenant_id();
        let workspace = request.workspace_id();
        let mode = prepared.mode;
        let keywords = &prepared.keywords;
        let embeddings = &prepared.embeddings;

        let max_chunks = prepared.max_chunks;
        // SPEC-031: pass allowed_document_ids into every mode for Tier 1 pre-filter
        let allowed_doc_ids = request.allowed_document_ids.as_deref();

        // OPS-24: single-mode paths get a top-level retrieval span; Mix/Hybrid
        // arms are spanned inside `run_arm_timed` (avoid double-nesting).
        let needs_mode_span = matches!(
            mode,
            QueryMode::Local | QueryMode::Global | QueryMode::Naive
        );

        let retrieve = async {
            match providers.vector_storage {
                Some(vector_storage) => match mode {
                    QueryMode::Local => {
                        self.query_local_with_vector_storage(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            allowed_doc_ids,
                            vector_storage,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Global => {
                        self.query_global_with_vector_storage(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            allowed_doc_ids,
                            vector_storage,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Hybrid => {
                        self.query_hybrid_with_vector_storage(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            allowed_doc_ids,
                            vector_storage,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Mix => {
                        self.query_mix_with_vector_storage(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            allowed_doc_ids,
                            vector_storage,
                            request.mix_weights.as_ref(),
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Naive => {
                        self.query_naive_with_vector_storage(
                            &request.query,
                            embeddings,
                            tenant,
                            workspace,
                            allowed_doc_ids,
                            vector_storage,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Bypass => Ok(QueryContext::default()),
                },
                None => match mode {
                    QueryMode::Local => {
                        self.query_local(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Global => {
                        self.query_global(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Hybrid => {
                        self.query_hybrid(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Mix => {
                        self.query_mix(
                            &request.query,
                            keywords,
                            embeddings,
                            tenant,
                            workspace,
                            request.mix_weights.as_ref(),
                            max_chunks,
                        )
                        .await
                    }
                    QueryMode::Naive => {
                        self.query_naive(&request.query, embeddings, tenant, workspace, max_chunks)
                            .await
                    }
                    QueryMode::Bypass => Ok(QueryContext::default()),
                },
            }
        };

        if needs_mode_span {
            with_rag_retrieval_span(
                RagRetrievalAttrs {
                    data_source_id: Some("edgequake"),
                    top_k: Some(max_chunks),
                    arm: Some(mode.as_str()),
                    mode: Some(mode.as_str()),
                    query_preview: Some(query_preview(&request.query, 64)),
                },
                async {
                    let ctx = retrieve.await?;
                    record_rag_retrieval_outcome(
                        ctx.chunks.is_empty() && ctx.entities.is_empty(),
                        false,
                        None,
                    );
                    Ok(ctx)
                },
            )
            .await
        } else {
            retrieve.await
        }
    }

    async fn pipeline_finalize(
        &self,
        request: QueryRequest,
        context: QueryContext,
        mode: QueryMode,
        stats: &mut QueryStats,
        providers: &QueryProviders<'_>,
        pipeline_start: Instant,
    ) -> Result<QueryResponse> {
        let final_context = self
            .postprocess_retrieved_context(&request, context, Some(stats))
            .await;

        let question_type = request.question_type();
        let (answer, generated_tokens) = if request.context_only {
            (String::new(), 0)
        } else if request.prompt_only {
            (
                self.build_prompt(
                    &request.query,
                    &final_context,
                    request.system_prompt.as_deref(),
                    &request.conversation_history,
                    question_type,
                    Some(request.response_type_or_default()),
                ),
                0,
            )
        } else if mode.is_bypass() {
            // P-G8 / RC-13: Bypass = direct LLM, no RAG template, no apology.
            // `final_context` is intentionally empty for Bypass (the retrieval
            // step is skipped in `run_query_pipeline`); the RAG `generate_answer`
            // would misinterpret that emptiness as a retrieval miss and return
            // the apology string. Use the dedicated direct-LLM path instead.
            let gen_start = Instant::now();
            let result = self
                .generate_bypass_answer(
                    &request.query,
                    providers.answer_llm.as_ref(),
                    request.system_prompt.as_deref(),
                    request.images.as_deref(),
                )
                .await?;
            stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
            result
        } else {
            let gen_start = Instant::now();
            let has_images = request
                .images
                .as_ref()
                .map(|i| !i.is_empty())
                .unwrap_or(false);
            let prompt = self.build_prompt(
                &request.query,
                &final_context,
                request.system_prompt.as_deref(),
                &request.conversation_history,
                question_type,
                Some(request.response_type_or_default()),
            );
            let mode_str = mode.as_str();
            let cache_key = crate::cache::llm_cache_storage_key(
                mode_str,
                crate::cache::LlmCacheType::Query,
                &crate::cache::hash_query_prompt_with_effort(
                    &prompt,
                    request.reasoning_effort.as_deref(),
                ),
            );

            // SPEC-103 answer cache. Active when explicitly wired or env allows.
            // Skip vision / empty context (no poison empty answers — LAW edge case).
            let answer_active =
                self.answer_cache.is_some() || crate::cache::answer_cache_enabled_from_env();
            if answer_active && !has_images && !final_context.is_empty() {
                if let Some(cache) = self.llm_response_cache.as_ref() {
                    if let Some(cached) = cache.get_return(&cache_key).await {
                        stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
                        stats.ttft_ms = Some(0);
                        stats.answer_cache_hit = true;
                        let tokens = cached.split_whitespace().count();
                        (cached, tokens)
                    } else {
                        let result = self
                            .generate_rag_answer(&request, &final_context, providers, question_type)
                            .await?;
                        stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
                        if !result.0.is_empty() {
                            cache
                                .set_return(
                                    &cache_key,
                                    crate::cache::LlmCacheType::Query,
                                    &result.0,
                                    Some(&prompt),
                                )
                                .await;
                        }
                        result
                    }
                } else if let Some(cache) = self.answer_cache.as_ref() {
                    // Legacy sync L1 path (tests that only call with_answer_cache pre-103).
                    let legacy_key = crate::cache::answer_cache_key(&prompt);
                    if let Some(cached) = cache.get(&legacy_key) {
                        stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
                        stats.ttft_ms = Some(0);
                        stats.answer_cache_hit = true;
                        let tokens = cached.split_whitespace().count();
                        (cached, tokens)
                    } else {
                        let result = self
                            .generate_rag_answer(&request, &final_context, providers, question_type)
                            .await?;
                        stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
                        if !result.0.is_empty() {
                            cache.set(&legacy_key, &result.0);
                        }
                        result
                    }
                } else {
                    let result = self
                        .generate_rag_answer(&request, &final_context, providers, question_type)
                        .await?;
                    stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
                    result
                }
            } else {
                let result = self
                    .generate_rag_answer(&request, &final_context, providers, question_type)
                    .await?;
                stats.generation_time_ms = gen_start.elapsed().as_millis() as u64;
                result
            }
        };

        stats.generated_tokens = generated_tokens;
        stats.reasoning_effort = request.reasoning_effort.clone();
        stats.total_time_ms = pipeline_start.elapsed().as_millis() as u64;

        // SPEC-046 OPS-P3.22: optional LLM-judge faithfulness (opt-in).
        // Falls back to heuristic sampler (OPS-P2.20) when judge off / fails.
        if let Some(score) = crate::eval::maybe_score_faithfulness_llm(
            crate::eval::faithfulness_judge_enabled_from_env(),
            providers.answer_llm.as_deref(),
            &answer,
            &final_context,
        )
        .await
        {
            stats.faithfulness_score = Some(score);
        } else if let Some(score) = crate::eval::maybe_score_faithfulness(
            crate::eval::faithfulness_sample_rate_from_env(),
            &request.query,
            &answer,
            &final_context,
        ) {
            stats.faithfulness_score = Some(score);
        }

        let explain = Some(crate::types::ExplainTrace::from_stats(&mode, stats));
        Ok(QueryResponse {
            answer,
            context: final_context,
            mode,
            stats: stats.clone(),
            explain,
        })
    }
}

const META_QUERY_INTENT: &str = "query_intent";
const META_KEYWORDS: &str = "extracted_keywords";

/// Stash LLM (or mock) keyword-extractor intent for postprocess truncation.
fn attach_query_intent_metadata(context: &mut QueryContext, intent: QueryIntent) {
    context.metadata.insert(
        META_QUERY_INTENT.to_string(),
        serde_json::Value::String(intent.to_string()),
    );
}

fn attach_keywords_metadata(context: &mut QueryContext, keywords: &ExtractedKeywords) {
    context.metadata.insert(
        META_KEYWORDS.to_string(),
        serde_json::json!({
            "high_level": keywords.high_level,
            "low_level": keywords.low_level,
        }),
    );
}

impl QueryEngine {
    /// RAG generate with optional answer-LLM override (DRY for answer-cache path).
    async fn generate_rag_answer(
        &self,
        request: &QueryRequest,
        context: &QueryContext,
        providers: &QueryProviders<'_>,
        question_type: Option<&str>,
    ) -> Result<(String, usize)> {
        let response_type = Some(request.response_type_or_default());
        if let Some(ref llm) = providers.answer_llm {
            self.generate_answer_with_provider(
                &request.query,
                context,
                Some(llm),
                request.system_prompt.as_deref(),
                request.images.as_deref(),
                &request.conversation_history,
                question_type,
                response_type,
                request.reasoning_effort.as_deref(),
            )
            .await
        } else {
            self.generate_answer_with_provider(
                &request.query,
                context,
                None,
                request.system_prompt.as_deref(),
                request.images.as_deref(),
                &request.conversation_history,
                question_type,
                response_type,
                request.reasoning_effort.as_deref(),
            )
            .await
        }
    }
}

fn keywords_from_context_metadata(context: &QueryContext) -> Vec<String> {
    let Some(v) = context.metadata.get(META_KEYWORDS) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for key in ["high_level", "low_level"] {
        if let Some(arr) = v.get(key).and_then(|x| x.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    out.push(s.to_string());
                }
            }
        }
    }
    out
}

/// Resolve truncation intent: metadata (LLM) → heuristic fallback.
fn intent_for_truncation(request: &QueryRequest, context: &QueryContext) -> QueryIntent {
    if let Some(v) = context.metadata.get(META_QUERY_INTENT) {
        if let Some(s) = v.as_str() {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return QueryIntent::from_str_loose(trimmed);
            }
        }
    }
    QueryIntent::classify_heuristic(&request.query)
}

#[cfg(test)]
mod intent_truncation_tests {
    use super::*;
    use crate::context::QueryContext;

    #[test]
    fn prefers_metadata_llm_intent_over_heuristic() {
        let mut ctx = QueryContext::new();
        // Heuristic would call this Factual ("What are …"), LLM says exploratory.
        attach_query_intent_metadata(&mut ctx, QueryIntent::Exploratory);
        let req = QueryRequest::new("What are the main diagnostic methods for AML?");
        assert_eq!(intent_for_truncation(&req, &ctx), QueryIntent::Exploratory);
    }

    #[test]
    fn falls_back_to_heuristic_without_metadata() {
        let ctx = QueryContext::new();
        let req = QueryRequest::new("What is machine learning?");
        assert_eq!(intent_for_truncation(&req, &ctx), QueryIntent::Factual);
    }
}

#[cfg(test)]
mod keyword_override_tests {
    use super::*;
    use crate::keywords::{KeywordExtractor, Keywords};
    use async_trait::async_trait;
    use edgequake_llm::MockProvider;
    use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountingKeywordExtractor {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl KeywordExtractor for CountingKeywordExtractor {
        async fn extract(&self, _query: &str) -> crate::error::Result<Keywords> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Keywords::new(vec!["from_extractor".into()], vec![]))
        }

        async fn extract_extended(&self, query: &str) -> crate::error::Result<ExtractedKeywords> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(ExtractedKeywords::new(
                vec!["from_extractor".into()],
                vec![],
                QueryIntent::classify_heuristic(query),
            ))
        }

        async fn extract_with_llm_override(
            &self,
            query: &str,
            _llm_override: Option<Arc<dyn LLMProvider>>,
        ) -> crate::error::Result<ExtractedKeywords> {
            self.extract_extended(query).await
        }
    }

    #[tokio::test]
    async fn hl_ll_override_skips_keyword_extractor() {
        let vector = Arc::new(MemoryVectorStorage::new("kw-override", 1536));
        let graph = Arc::new(MemoryGraphStorage::new("kw-override"));
        let embed: Arc<dyn crate::EmbeddingProvider> = Arc::new(MockProvider::default());
        let llm: Arc<dyn LLMProvider> = Arc::new(MockProvider::default());
        let counter = Arc::new(CountingKeywordExtractor {
            calls: AtomicUsize::new(0),
        });
        let engine = QueryEngine::with_mock_keywords(
            crate::QueryEngineConfig::default(),
            vector,
            graph,
            embed,
            llm,
        )
        .with_keyword_extractor(counter.clone());

        let request = QueryRequest::new("staging for NSCLC")
            .context_only()
            .with_hl_keywords(vec!["staging".into(), "NSCLC".into()])
            .with_ll_keywords(vec!["TNM".into()]);

        let response = engine.query(request).await.expect("query");
        assert_eq!(
            counter.calls.load(Ordering::SeqCst),
            0,
            "keyword extractor must not run when hl/ll override present"
        );
        let kw = response
            .context
            .metadata
            .get("extracted_keywords")
            .expect("keywords metadata");
        assert_eq!(
            kw.get("high_level")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(2)
        );
        assert_eq!(
            kw.get("low_level")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str()),
            Some("TNM")
        );
    }
}
