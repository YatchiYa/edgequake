//! Query-side caches (SPEC-021 P-G9 / RC-14 · 064 product polish).
//!
//! - `CachingEmbeddingProvider`: memoizes `embed_one` **and** per-text batch `embed`.
//! - `QueryResultCache`: memoizes `context_only` retrieval contexts.
//! - `InMemoryAnswerCache`: opt-in Mix answer LLM cache (LR `cache_type=query`).

pub mod answer_cache;
pub mod embedding_cache;
pub mod query_result_cache;

pub use answer_cache::{
    answer_cache_enabled_from_env, answer_cache_key, InMemoryAnswerCache, SharedAnswerCache,
};
pub use embedding_cache::CachingEmbeddingProvider;
pub use query_result_cache::QueryResultCache;

/// Port for invalidating cached `context_only` retrieval after ingestion (DIP).
pub trait QueryResultCacheInvalidator: Send + Sync {
    fn invalidate_query_result_cache(&self);

    /// Workspace-scoped invalidation — does not bust other workspaces' cache entries.
    fn invalidate_query_result_cache_for_workspace(&self, workspace_id: &str) {
        let _ = workspace_id;
        self.invalidate_query_result_cache();
    }
}
