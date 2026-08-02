//! Streaming query entry points (SPEC-017 P1-01).
//!
//! All streaming paths delegate context retrieval to `query_pipeline`, then stream
//! the LLM answer from the enriched context.

use std::sync::Arc;

use futures::StreamExt;

use crate::context::QueryContext;
use crate::error::{QueryError, Result};
use crate::modes::QueryMode;
use edgequake_storage::traits::VectorStorage;

use super::super::QueryEngine;
use super::query_pipeline::QueryProviders;

pub(super) use super::super::TokenStream;

impl QueryEngine {
    /// Legacy v1 streaming: tokens only (uses default providers + shared pipeline).
    pub async fn query_stream(&self, request: crate::types::QueryRequest) -> Result<TokenStream> {
        let (_, _, stream) = self.query_stream_with_context(request).await?;
        Ok(stream)
    }

    /// Streaming query returning context + token stream (default providers).
    pub async fn query_stream_with_context(
        &self,
        request: crate::types::QueryRequest,
    ) -> Result<(QueryContext, QueryMode, TokenStream)> {
        self.stream_with_providers(
            request,
            QueryProviders {
                embedding: self.embedding_provider.as_ref(),
                vector_storage: None,
                keyword_llm: None,
                answer_llm: None,
            },
        )
        .await
    }

    /// Streaming query with LLM override for keyword extraction and answer generation.
    pub async fn query_stream_with_context_and_llm(
        &self,
        request: crate::types::QueryRequest,
        llm_provider: Arc<dyn crate::LLMProvider>,
    ) -> Result<(QueryContext, QueryMode, TokenStream)> {
        let llm = llm_provider.clone();
        self.stream_with_providers(
            request,
            QueryProviders {
                embedding: self.embedding_provider.as_ref(),
                vector_storage: None,
                keyword_llm: Some(llm_provider),
                answer_llm: Some(llm),
            },
        )
        .await
    }

    /// Streaming query with full workspace embedding/vector config + optional LLM override.
    pub async fn query_stream_with_full_config(
        &self,
        request: crate::types::QueryRequest,
        embedding_provider: Arc<dyn crate::EmbeddingProvider>,
        vector_storage: Arc<dyn VectorStorage>,
        llm_provider: Option<Arc<dyn crate::LLMProvider>>,
    ) -> Result<(QueryContext, QueryMode, TokenStream)> {
        self.query_stream_with_role_llms(
            request,
            embedding_provider,
            vector_storage,
            llm_provider.clone(),
            llm_provider,
        )
        .await
    }

    /// Streaming with separate Keyword vs Query role LLMs (SPEC-046 EQ-046-13).
    pub async fn query_stream_with_role_llms(
        &self,
        request: crate::types::QueryRequest,
        embedding_provider: Arc<dyn crate::EmbeddingProvider>,
        vector_storage: Arc<dyn VectorStorage>,
        keyword_llm: Option<Arc<dyn crate::LLMProvider>>,
        answer_llm: Option<Arc<dyn crate::LLMProvider>>,
    ) -> Result<(QueryContext, QueryMode, TokenStream)> {
        self.stream_with_providers(
            request,
            QueryProviders {
                embedding: embedding_provider.as_ref(),
                vector_storage: Some(&vector_storage),
                keyword_llm,
                answer_llm,
            },
        )
        .await
    }

    async fn stream_with_providers(
        &self,
        request: crate::types::QueryRequest,
        providers: QueryProviders<'_>,
    ) -> Result<(QueryContext, QueryMode, TokenStream)> {
        let answer_llm = providers.answer_llm.clone();
        let (context, mode) = self.run_context_pipeline(&request, providers).await?;
        let context = self.enrich_retrieved_context(&request, context).await;
        self.stream_answer_from_context(&request, context, mode, answer_llm)
            .await
    }

    async fn stream_answer_from_context(
        &self,
        request: &crate::types::QueryRequest,
        context: QueryContext,
        mode: QueryMode,
        llm_override: Option<Arc<dyn crate::LLMProvider>>,
    ) -> Result<(QueryContext, QueryMode, TokenStream)> {
        // P-G11 (RC-16): vision parity. When the request carries images, use
        // the vision-capable `chat` path (the `stream` trait method cannot carry
        // images). E30: vision-LLm failure falls back to text-only stream inside
        // `stream_vision_answer`.
        if let Some(images) = request.images.as_ref().filter(|i| !i.is_empty()) {
            let stream = self
                .stream_vision_answer(
                    &request.query,
                    &context,
                    llm_override,
                    request.system_prompt.as_deref(),
                    images,
                )
                .await?;
            return Ok((context, mode, stream));
        }

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

        let response_type = Some(request.response_type_or_default());
        let prompt = self.build_prompt(
            &request.query,
            &context,
            request.system_prompt.as_deref(),
            &request.conversation_history,
            request.question_type(),
            response_type,
        );
        let system_text = self.build_system_prompt(
            &context,
            request.system_prompt.as_deref(),
            &request.conversation_history,
            request.question_type(),
            response_type,
        );
        let use_complete_blob = matches!(
            std::env::var("EDGEQUAKE_ANSWER_COMPLETE_BLOB")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes" | "on"
        );

        // SPEC-103 answer cache: stream cached answer as one chunk.
        // Warm fills happen on the non-stream generate path (LR also skips
        // caching streaming responses). Skip empty context / vision (no images here).
        let mode_str = mode.as_str();
        let cache_key = crate::cache::llm_cache_storage_key(
            mode_str,
            crate::cache::LlmCacheType::Query,
            &crate::cache::hash_query_prompt(&prompt),
        );
        let answer_cache_on = self.answer_cache.is_some()
            || crate::cache::answer_cache_enabled_from_env();
        if answer_cache_on && !context.is_empty() {
            if let Some(cache) = self.llm_response_cache.as_ref() {
                if let Some(cached) = cache.get_return(&cache_key).await {
                    return Ok((
                        context,
                        mode,
                        futures::stream::once(async move { Ok(cached) }).boxed(),
                    ));
                }
            } else if let Some(cache) = self.answer_cache.as_ref() {
                let legacy_key = crate::cache::answer_cache_key(&prompt);
                if let Some(cached) = cache.get(&legacy_key) {
                    return Ok((
                        context,
                        mode,
                        futures::stream::once(async move { Ok(cached) }).boxed(),
                    ));
                }
            }
        }

        let llm = llm_override.unwrap_or_else(|| self.llm_provider.clone());
        let llm_cache = self.llm_response_cache.clone();
        let answer_cache = self.answer_cache.clone();
        let prompt_for_cache = prompt.clone();
        let cache_key_for_write = cache_key.clone();
        let context_nonempty = answer_cache_on && !context.is_empty();

        // 083: prefer system/user chat when not COMPLETE_BLOB (even for stream entry —
        // token stream API is one-blob; chat preserves LR roles as a one-shot stream).
        let stream = if !use_complete_blob {
            use edgequake_llm::traits::ChatMessage;
            let messages = vec![
                ChatMessage::system(&system_text),
                ChatMessage::user(&request.query),
            ];
            match llm.chat(&messages, None).await {
                Ok(response) => {
                    if context_nonempty && !response.content.is_empty() {
                        if let Some(cache) = llm_cache.as_ref() {
                            cache
                                .set_return(
                                    &cache_key_for_write,
                                    crate::cache::LlmCacheType::Query,
                                    &response.content,
                                    Some(&prompt_for_cache),
                                )
                                .await;
                        } else if let Some(cache) = answer_cache.as_ref() {
                            cache.set(
                                &crate::cache::answer_cache_key(&prompt_for_cache),
                                &response.content,
                            );
                        }
                    }
                    futures::stream::once(async move { Ok(response.content) }).boxed()
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "083 stream chat failed; falling back to text stream/complete"
                    );
                    if llm.supports_streaming() {
                        llm.stream(&prompt)
                            .await
                            .map(|stream| stream.map(|res| res.map_err(QueryError::from)).boxed())
                            .map_err(QueryError::from)?
                    } else {
                        let response = llm.complete(&prompt).await.map_err(QueryError::from)?;
                        futures::stream::once(async move { Ok(response.content) }).boxed()
                    }
                }
            }
        } else if llm.supports_streaming() {
            llm.stream(&prompt)
                .await
                .map(|stream| stream.map(|res| res.map_err(QueryError::from)).boxed())
                .map_err(QueryError::from)?
        } else {
            tracing::warn!(
                provider = llm.name(),
                "Provider doesn't support streaming, falling back to non-streaming mode"
            );
            let response = llm.complete(&prompt).await.map_err(QueryError::from)?;
            if context_nonempty && !response.content.is_empty() {
                if let Some(cache) = llm_cache.as_ref() {
                    cache
                        .set_return(
                            &cache_key_for_write,
                            crate::cache::LlmCacheType::Query,
                            &response.content,
                            Some(&prompt_for_cache),
                        )
                        .await;
                } else if let Some(cache) = answer_cache.as_ref() {
                    cache.set(
                        &crate::cache::answer_cache_key(&prompt_for_cache),
                        &response.content,
                    );
                }
            }
            futures::stream::once(async move { Ok(response.content) }).boxed()
        };

        Ok((context, mode, stream))
    }
}
