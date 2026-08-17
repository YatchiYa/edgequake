<<<<<<< HEAD
//! Product query-answer LLM cache (LightRAG `cache_type=query` law / 064).
//!
//! First principles: identical Mix prompt → identical answer under temperature-0
//! SUT pins. Caching skips the generate RTT for warm repeats (product UX), not
//! Acc fairness. **Default off** via `EDGEQUAKE_QUERY_ANSWER_CACHE`.
//!
//! Key = SHA-256 of the fully built RAG prompt (includes context + history +
//! system prompt). Ingest / graph change → different context → different key.
=======
//! Product query-answer LLM cache (LightRAG `cache_type=query` / SPEC-103).
//!
//! First principles: identical Mix prompt → identical answer under temperature-0
//! SUT pins. Caching skips the generate RTT for warm repeats (product UX), not
//! Acc fairness. Defaults follow master `EDGEQUAKE_LLM_CACHE` (LAW-C6).
//!
//! Key = LR-shaped `{mode}:query:{hash}-cache` where hash is SHA-256 of the
//! fully built RAG prompt (context-inclusive — LAW-C3). Prefer
//! [`crate::cache::llm_response_cache`] for durable L1/L2.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

<<<<<<< HEAD
use sha2::{Digest, Sha256};

/// Opt-in product answer cache (Acc Acc / Acc Fact leave unset/off).
pub fn answer_cache_enabled_from_env() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_QUERY_ANSWER_CACHE")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

/// Stable cache key for a fully assembled RAG prompt.
pub fn answer_cache_key(prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    format!("{:x}", hasher.finalize())
}
=======
pub use crate::cache::llm_response_cache::{answer_cache_enabled_from_env, answer_cache_key};
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

#[derive(Debug)]
struct Entry {
    answer: String,
    expires_at: Option<Instant>,
    accessed_at: Instant,
}

/// In-process LRU+TTL answer cache (product warm-repeat).
#[derive(Debug)]
pub struct InMemoryAnswerCache {
    max_size: usize,
    ttl: Duration,
    cache: RwLock<HashMap<String, Entry>>,
    hits: RwLock<u64>,
    misses: RwLock<u64>,
}

impl InMemoryAnswerCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            max_size,
            ttl,
            cache: RwLock::new(HashMap::new()),
            hits: RwLock::new(0),
            misses: RwLock::new(0),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(1_000, Duration::from_secs(3600))
    }

    pub fn hits(&self) -> u64 {
        *self.hits.read().unwrap()
    }

    pub fn misses(&self) -> u64 {
        *self.misses.read().unwrap()
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let now = Instant::now();
        let mut cache = self.cache.write().unwrap();
        if let Some(entry) = cache.get_mut(key) {
            let live = entry.expires_at.map(|exp| exp > now).unwrap_or(true);
            if live {
                entry.accessed_at = now;
                *self.hits.write().unwrap() += 1;
                return Some(entry.answer.clone());
            }
            cache.remove(key);
        }
        *self.misses.write().unwrap() += 1;
        None
    }

    pub fn set(&self, key: &str, answer: &str) {
        let now = Instant::now();
        self.evict_if_needed();
        let mut cache = self.cache.write().unwrap();
        cache.insert(
            key.to_string(),
            Entry {
                answer: answer.to_string(),
                expires_at: Some(now + self.ttl),
                accessed_at: now,
            },
        );
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }

    fn evict_if_needed(&self) {
        let mut cache = self.cache.write().unwrap();
        let now = Instant::now();
        cache.retain(|_, e| e.expires_at.map(|exp| exp > now).unwrap_or(true));
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

/// Shared handle stored on [`crate::QueryEngine`].
pub type SharedAnswerCache = Arc<InMemoryAnswerCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_hit() {
        let c = InMemoryAnswerCache::with_defaults();
        let k = answer_cache_key("prompt-a");
        assert!(c.get(&k).is_none());
        assert_eq!(c.misses(), 1);
        c.set(&k, "answer-a");
        assert_eq!(c.get(&k).as_deref(), Some("answer-a"));
        assert_eq!(c.hits(), 1);
    }

    #[test]
    fn different_prompts_different_keys() {
        assert_ne!(answer_cache_key("a"), answer_cache_key("b"));
    }
}
