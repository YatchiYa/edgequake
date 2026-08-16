//! Contract test for P-G9 (RC-14) / 064: query embedding cache.
//!
//! `CachingEmbeddingProvider` memoizes per-text for both `embed_one` and batch
//! `embed` (same keys). `embed_one` is implemented as a 1-text `embed`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use edgequake_llm::traits::EmbeddingProvider;
use edgequake_llm::MockProvider;
use edgequake_query::cache::CachingEmbeddingProvider;

/// Wraps an embedding provider and counts inner `embed` RTTs.
struct CountingEmbedding {
    inner: Arc<dyn EmbeddingProvider>,
    embed_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl EmbeddingProvider for CountingEmbedding {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn model(&self) -> &str {
        self.inner.model()
    }
    fn dimension(&self) -> usize {
        self.inner.dimension()
    }
    fn max_tokens(&self) -> usize {
        self.inner.max_tokens()
    }
    async fn embed(&self, texts: &[String]) -> edgequake_llm::Result<Vec<Vec<f32>>> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed(texts).await
    }
}

#[tokio::test]
async fn repeated_query_embeddings_skip_inner_provider() {
    let mock: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider::default());
    let embed_calls = Arc::new(AtomicUsize::new(0));
    let counting = Arc::new(CountingEmbedding {
        inner: mock,
        embed_calls: embed_calls.clone(),
    }) as Arc<dyn EmbeddingProvider>;

    let cached = Arc::new(CachingEmbeddingProvider::with_defaults(counting));

    for _ in 0..3 {
        let _ = cached.embed_one("what is GraphRAG?").await.unwrap();
    }

    assert_eq!(
        embed_calls.load(Ordering::SeqCst),
        1,
        "repeated identical queries must hit the cache, not the inner provider"
    );
    assert_eq!(cached.hits(), 2, "two cache hits expected");
    assert_eq!(cached.misses(), 1, "one cache miss expected (first call)");
}

#[tokio::test]
async fn batch_embed_shares_per_text_cache_with_embed_one() {
    let mock: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider::default());
    let embed_calls = Arc::new(AtomicUsize::new(0));
    let counting = Arc::new(CountingEmbedding {
        inner: mock,
        embed_calls: embed_calls.clone(),
    }) as Arc<dyn EmbeddingProvider>;

    let cached = Arc::new(CachingEmbeddingProvider::with_defaults(counting));

    let texts = vec!["chunk a".to_string(), "chunk b".to_string()];
    let out = cached.embed(&texts).await.unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(embed_calls.load(Ordering::SeqCst), 1);
    assert_eq!(cached.misses(), 2);
    assert_eq!(cached.hits(), 0);

    let _ = cached.embed_one("chunk a").await.unwrap();
    assert_eq!(
        embed_calls.load(Ordering::SeqCst),
        1,
        "embed_one must reuse the batch-populated per-text cache"
    );
    assert_eq!(cached.hits(), 1);
}

#[tokio::test]
async fn distinct_queries_each_miss_the_cache() {
    let mock: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider::default());
    let cached = Arc::new(CachingEmbeddingProvider::with_defaults(
        mock as Arc<dyn EmbeddingProvider>,
    ));

    let _ = cached.embed_one("query one").await.unwrap();
    let _ = cached.embed_one("query two").await.unwrap();
    let _ = cached.embed_one("query three").await.unwrap();

    assert_eq!(cached.misses(), 3, "three distinct queries must all miss");
    assert_eq!(cached.hits(), 0);
}
