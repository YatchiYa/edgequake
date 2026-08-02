//! Keyword caching for efficient repeated queries.
//!
//! Provides multi-level caching:
//! - In-memory LRU cache (fast, limited size)
//! - Durable L2 via SPEC-103 [`crate::cache::LlmResponseCache`] → `public.llm_cache`

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

use super::extractor::ExtractedKeywords;
use crate::cache::llm_response_cache::{LlmCacheType, SharedLlmResponseCache};
use crate::error::Result;

/// Trait for keyword cache implementations.
#[async_trait]
pub trait KeywordCache: Send + Sync {
    /// Get cached keywords by key.
    async fn get(&self, key: &str) -> Result<Option<ExtractedKeywords>>;

    /// Store keywords with optional TTL.
    async fn set(
        &self,
        key: &str,
        keywords: &ExtractedKeywords,
        ttl: Option<Duration>,
    ) -> Result<()>;

    /// Delete cached keywords.
    async fn delete(&self, key: &str) -> Result<()>;

    /// Clear all cached keywords.
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics.
    async fn stats(&self) -> CacheStats;
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

impl CacheStats {
    /// Calculate hit rate as percentage.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// In-memory LRU cache for keywords.
///
/// Fast but limited to single instance. Use as L1 cache.
pub struct InMemoryKeywordCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    max_size: usize,
    stats: RwLock<CacheStats>,
}

struct CacheEntry {
    keywords: ExtractedKeywords,
    expires_at: Option<std::time::Instant>,
    accessed_at: std::time::Instant,
}

impl InMemoryKeywordCache {
    /// Create a new in-memory cache.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Evict expired and LRU entries if over capacity.
    fn evict_if_needed(&self) {
        let mut cache = self.cache.write().unwrap();

        // First, remove expired entries
        let now = std::time::Instant::now();
        cache.retain(|_, entry| entry.expires_at.map(|exp| exp > now).unwrap_or(true));

        // If still over capacity, remove LRU entries
        while cache.len() >= self.max_size {
            // Find oldest entry
            let oldest_key = cache
                .iter()
                .min_by_key(|(_, entry)| entry.accessed_at)
                .map(|(key, _)| key.clone());

            if let Some(key) = oldest_key {
                cache.remove(&key);
            } else {
                break;
            }
        }
    }
}

#[async_trait]
impl KeywordCache for InMemoryKeywordCache {
    async fn get(&self, key: &str) -> Result<Option<ExtractedKeywords>> {
        let now = std::time::Instant::now();

        // Check if entry exists and not expired
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(key) {
                // Check expiration
                if let Some(expires_at) = entry.expires_at {
                    if expires_at <= now {
                        // Expired, will be cleaned up later
                        let mut stats = self.stats.write().unwrap();
                        stats.misses += 1;
                        return Ok(None);
                    }
                }

                // Hit! Update stats
                let keywords = entry.keywords.clone();
                drop(cache);

                // Update access time
                if let Ok(mut cache) = self.cache.write() {
                    if let Some(entry) = cache.get_mut(key) {
                        entry.accessed_at = now;
                    }
                }

                let mut stats = self.stats.write().unwrap();
                stats.hits += 1;
                return Ok(Some(keywords));
            }
        }

        // Miss
        let mut stats = self.stats.write().unwrap();
        stats.misses += 1;
        Ok(None)
    }

    async fn set(
        &self,
        key: &str,
        keywords: &ExtractedKeywords,
        ttl: Option<Duration>,
    ) -> Result<()> {
        self.evict_if_needed();

        let now = std::time::Instant::now();
        let entry = CacheEntry {
            keywords: keywords.clone(),
            expires_at: ttl.map(|d| now + d),
            accessed_at: now,
        };

        let mut cache = self.cache.write().unwrap();
        cache.insert(key.to_string(), entry);

        // Update size stat
        let mut stats = self.stats.write().unwrap();
        stats.size = cache.len();

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);

        let mut stats = self.stats.write().unwrap();
        stats.size = cache.len();

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        let mut stats = self.stats.write().unwrap();
        stats.size = 0;

        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }
}

impl Default for InMemoryKeywordCache {
    fn default() -> Self {
        Self::new(1000) // Default 1000 entries
    }
}

/// L1 memory + optional L2 durable LLM response cache (SPEC-103).
///
/// Keys are already LR-shaped storage keys (`{mode}:keywords:{hash}-cache`).
pub struct TieredKeywordCache {
    l1: InMemoryKeywordCache,
    l2: Option<SharedLlmResponseCache>,
}

impl TieredKeywordCache {
    pub fn new(l1: InMemoryKeywordCache, l2: Option<SharedLlmResponseCache>) -> Self {
        Self { l1, l2 }
    }

    pub fn memory_only(max_size: usize) -> Self {
        Self::new(InMemoryKeywordCache::new(max_size), None)
    }

    pub fn with_durable(max_size: usize, durable: SharedLlmResponseCache) -> Self {
        Self::new(InMemoryKeywordCache::new(max_size), Some(durable))
    }
}

#[async_trait]
impl KeywordCache for TieredKeywordCache {
    async fn get(&self, key: &str) -> Result<Option<ExtractedKeywords>> {
        if let Some(hit) = self.l1.get(key).await? {
            return Ok(Some(hit));
        }
        let Some(l2) = &self.l2 else {
            return Ok(None);
        };
        let Some(raw) = l2.get_return(key).await else {
            return Ok(None);
        };
        match serde_json::from_str::<ExtractedKeywords>(&raw) {
            Ok(kw) => {
                let _ = self
                    .l1
                    .set(key, &kw, Some(Duration::from_secs(24 * 60 * 60)))
                    .await;
                Ok(Some(kw))
            }
            Err(e) => {
                tracing::warn!(error = %e, key = %key, "SPEC-103 keyword L2 deserialize failed");
                Ok(None)
            }
        }
    }

    async fn set(
        &self,
        key: &str,
        keywords: &ExtractedKeywords,
        ttl: Option<Duration>,
    ) -> Result<()> {
        self.l1.set(key, keywords, ttl).await?;
        if let Some(l2) = &self.l2 {
            match serde_json::to_string(keywords) {
                Ok(raw) => {
                    l2.set_return(key, LlmCacheType::Keywords, &raw, None).await;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "SPEC-103 keyword L2 serialize failed");
                }
            }
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.l1.delete(key).await
    }

    async fn clear(&self) -> Result<()> {
        if let Some(l2) = &self.l2 {
            l2.clear_l1();
        }
        self.l1.clear().await
    }

    async fn stats(&self) -> CacheStats {
        self.l1.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_cache_basic() {
        let cache = InMemoryKeywordCache::new(10);

        let keywords = ExtractedKeywords::new(
            vec!["theme".to_string()],
            vec!["entity".to_string()],
            super::super::intent::QueryIntent::Factual,
        );

        // Set
        cache.set("key1", &keywords, None).await.unwrap();

        // Get
        let result = cache.get("key1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().high_level, vec!["theme".to_string()]);

        // Miss
        let result = cache.get("nonexistent").await.unwrap();
        assert!(result.is_none());

        // Stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_in_memory_cache_expiration() {
        let cache = InMemoryKeywordCache::new(10);

        let keywords = ExtractedKeywords::new(
            vec!["theme".to_string()],
            vec!["entity".to_string()],
            super::super::intent::QueryIntent::Factual,
        );

        // Set with very short TTL
        cache
            .set("key1", &keywords, Some(Duration::from_millis(1)))
            .await
            .unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should be expired
        let result = cache.get("key1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_eviction() {
        let cache = InMemoryKeywordCache::new(2);

        let keywords = ExtractedKeywords::new(
            vec!["theme".to_string()],
            vec!["entity".to_string()],
            super::super::intent::QueryIntent::Factual,
        );

        // Fill cache
        cache.set("key1", &keywords, None).await.unwrap();
        cache.set("key2", &keywords, None).await.unwrap();

        // Access key1 to make it more recent
        cache.get("key1").await.unwrap();

        // Add new entry, should evict key2 (LRU)
        cache.set("key3", &keywords, None).await.unwrap();

        // key1 should still exist
        assert!(cache.get("key1").await.unwrap().is_some());

        // key3 should exist
        assert!(cache.get("key3").await.unwrap().is_some());
    }
}
