//! Query-embedding cache (P-G9 / RC-14).
//!
//! WHY (First Principles): every non-Bypass query re-embeds the query string on
//! every request. For a popular query (or repeated `context_only` inspection),
//! that is a redundant, deterministic LLM/embedding-provider round-trip. The
//! embedding of a given text under a given model is a pure function of
//! (model, text), so it is safe to memoize.
//!
//! Design (DIP / DRY):
//! - `CachingEmbeddingProvider` wraps any `EmbeddingProvider` and memoizes
//!   results keyed by `hash(model || text)` for both `embed_one` and per-text
//!   slots inside batch `embed` (064 / LightRAG query batch law).
//! - Batch `embed` fills missing texts in one inner round-trip, then stores each
//!   vector so a later `embed_one` / `embed` hit skips the network.
//! - LRU + TTL eviction (mirrors `keywords::cache::InMemoryKeywordCache`).
//! - `embedding_version` is part of the key (E27): changing the embedding model
//!   or its configuration invalidates the whole cache without a manual clear.
//! - Thread-safe via `RwLock<HashMap>`; no `async` lock held across an await
//!   on a miss (the inner provider call runs with no lock held).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use edgequake_llm::traits::EmbeddingProvider;

/// A hashed cache key bundling the embedding version (model identity) and the
/// text. Two equal texts under different models must NOT collide (E27).
fn cache_key(version: &str, text: &str) -> String {
    // Suffix the version so a model swap cannot serve a stale vector. Use a
    // delimiter unlikely to appear in either field.
    format!("{version}\u{1f}\u{1e}{text}")
}

struct Entry {
    embedding: Vec<f32>,
    expires_at: Option<Instant>,
    accessed_at: Instant,
}

/// LRU + TTL embedding cache wrapping any `EmbeddingProvider`.
///
/// Query path: `embed_one` and batch `embed` share the same per-text keys so
/// keyword-level batches after a speculative `embed_one(query)` stay warm.
pub struct CachingEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    /// Bumped into the key so a model/config change invalidates everything.
    version: String,
    max_size: usize,
    ttl: Duration,
    cache: RwLock<HashMap<String, Entry>>,
    hits: RwLock<u64>,
    misses: RwLock<u64>,
}

impl CachingEmbeddingProvider {
    /// Wrap `inner` with an embedding cache of `max_size` entries and `ttl`.
    /// `version` is folded into the cache key (e.g. `"{name}/{model}/{dim}"`);
    /// change it to invalidate.
    pub fn new(inner: Arc<dyn EmbeddingProvider>, max_size: usize, ttl: Duration) -> Self {
        let version = format!("{}/{}", inner.name(), inner.model());
        Self {
            inner,
            version,
            max_size,
            ttl,
            cache: RwLock::new(HashMap::new()),
            hits: RwLock::new(0),
            misses: RwLock::new(0),
        }
    }

    /// Wrap with defaults: 10_000 entries, 1h TTL (plan-19 P-G9).
    pub fn with_defaults(inner: Arc<dyn EmbeddingProvider>) -> Self {
        Self::new(inner, 10_000, Duration::from_secs(3600))
    }

    /// Cache hit count (for tests / observability).
    pub fn hits(&self) -> u64 {
        *self.hits.read().unwrap()
    }

    /// Cache miss count.
    pub fn misses(&self) -> u64 {
        *self.misses.read().unwrap()
    }

    fn evict_if_needed(&self) {
        let mut cache = self.cache.write().unwrap();
        let now = Instant::now();
        // Drop expired entries first.
        cache.retain(|_, e| e.expires_at.map(|exp| exp > now).unwrap_or(true));
        // Then LRU-evict until under capacity.
        while cache.len() >= self.max_size {
            let oldest = cache
                .iter()
                .min_by_key(|(_, e)| e.accessed_at)
                .map(|(k, _)| k.clone());
            match oldest {
                Some(k) => {
                    cache.remove(&k);
                }
                None => break,
            }
        }
    }
}

#[async_trait]
impl EmbeddingProvider for CachingEmbeddingProvider {
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

    fn max_batch_size(&self) -> usize {
        self.inner.max_batch_size()
    }

    async fn embed(&self, texts: &[String]) -> edgequake_llm::Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let now = Instant::now();
        let mut out: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
        let mut missing: Vec<(usize, String)> = Vec::new();

        {
            let cache = self.cache.read().unwrap();
            for (i, text) in texts.iter().enumerate() {
                let key = cache_key(&self.version, text);
                if let Some(entry) = cache.get(&key) {
                    let live = entry.expires_at.map(|exp| exp > now).unwrap_or(true);
                    if live {
                        *self.hits.write().unwrap() += 1;
                        out[i] = Some(entry.embedding.clone());
                        continue;
                    }
                }
                *self.misses.write().unwrap() += 1;
                missing.push((i, text.clone()));
            }
        }

        if !missing.is_empty() {
            let miss_texts: Vec<String> = missing.iter().map(|(_, t)| t.clone()).collect();
            let vectors = self.inner.embed(&miss_texts).await?;
            if vectors.len() != miss_texts.len() {
                return Err(edgequake_llm::error::LlmError::ProviderError(format!(
                    "embed batch size mismatch: expected {} got {}",
                    miss_texts.len(),
                    vectors.len()
                )));
            }
            self.evict_if_needed();
            let mut cache = self.cache.write().unwrap();
            let store_at = Instant::now();
            for ((i, text), embedding) in missing.into_iter().zip(vectors) {
                let key = cache_key(&self.version, &text);
                cache.insert(
                    key,
                    Entry {
                        embedding: embedding.clone(),
                        expires_at: Some(store_at + self.ttl),
                        accessed_at: store_at,
                    },
                );
                out[i] = Some(embedding);
            }
        }

        Ok(out
            .into_iter()
            .map(|v| v.expect("embed cache slot filled"))
            .collect())
    }

    // embed_one is the hot query path — memoize it (same keys as batch).
    async fn embed_one(&self, text: &str) -> edgequake_llm::Result<Vec<f32>> {
        let mut batch = self.embed(&[text.to_string()]).await?;
        Ok(batch
            .pop()
            .expect("embed_one batch always returns one vector"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_llm::MockProvider;

    #[tokio::test]
    async fn repeated_embed_one_hits_cache() {
        let mock = Arc::new(MockProvider::default());
        let cached = Arc::new(CachingEmbeddingProvider::with_defaults(
            mock as Arc<dyn EmbeddingProvider>,
        ));

        // Two identical embed_one calls: the second must be a cache hit.
        let a = cached.embed_one("hello world").await.unwrap();
        let b = cached.embed_one("hello world").await.unwrap();
        assert_eq!(a, b, "cached embedding must equal the first call");
        assert_eq!(cached.hits(), 1, "second call must hit the cache");
        assert_eq!(cached.misses(), 1, "first call must miss");
    }

    #[tokio::test]
    async fn different_texts_do_not_collide() {
        let mock = Arc::new(MockProvider::default());
        let cached = Arc::new(CachingEmbeddingProvider::with_defaults(
            mock as Arc<dyn EmbeddingProvider>,
        ));

        let _ = cached.embed_one("alpha").await.unwrap();
        let _ = cached.embed_one("beta").await.unwrap();
        // Two distinct texts → two misses, zero hits.
        assert_eq!(cached.misses(), 2);
        assert_eq!(cached.hits(), 0);
    }

    #[tokio::test]
    async fn embed_batch_populates_and_shares_cache_with_embed_one() {
        let mock = Arc::new(MockProvider::default());
        let cached = Arc::new(CachingEmbeddingProvider::with_defaults(
            mock as Arc<dyn EmbeddingProvider>,
        ));

        let texts = vec!["x".to_string(), "y".to_string()];
        let out = cached.embed(&texts).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(cached.misses(), 2);
        // Second batch: both texts hit.
        let out2 = cached.embed(&texts).await.unwrap();
        assert_eq!(out, out2);
        assert_eq!(cached.hits(), 2);
        // embed_one reuses the same key.
        let _ = cached.embed_one("x").await.unwrap();
        assert_eq!(cached.hits(), 3);
    }

    #[tokio::test]
    async fn ttl_expiration_causes_re_embed() {
        let mock = Arc::new(MockProvider::default());
        let cached = Arc::new(CachingEmbeddingProvider::new(
            mock as Arc<dyn EmbeddingProvider>,
            10,
            Duration::from_millis(1),
        ));

        let _ = cached.embed_one("ephemeral").await.unwrap();
        // Wait past the TTL.
        tokio::time::sleep(Duration::from_millis(10)).await;
        let _ = cached.embed_one("ephemeral").await.unwrap();
        // Both calls miss (the second because the entry expired).
        assert_eq!(cached.misses(), 2);
        assert_eq!(cached.hits(), 0);
    }
}
