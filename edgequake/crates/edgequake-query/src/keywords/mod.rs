//! Keyword extraction module for SOTA query processing.
//!
//! This module provides LightRAG-inspired keyword extraction with:
//! - LLM-based high/low level keyword extraction
//! - Query intent classification for adaptive retrieval
//! - Multi-level caching (in-memory + PostgreSQL)
//!
//! # Architecture
//!
//! ```text
//! Query → CachedKeywordExtractor → Cache Hit? → Return
//!                                      ↓ Miss
//!                               LLMKeywordExtractor → LLM Call → Parse → Cache → Return
//! ```
//!
//! @implements FEAT0107 (Keyword Extraction)

mod cache;
mod extractor;
mod intent;
mod keyword_mode;
mod llm_extractor;
mod mock_extractor;

#[cfg(feature = "postgres")]
pub use cache::PostgresKeywordCache;
pub use cache::{InMemoryKeywordCache, KeywordCache};
pub use extractor::{ExtractedKeywords, KeywordExtractor, Keywords};
pub use intent::QueryIntent;
pub use keyword_mode::{
    heuristic_extracted_keywords, keyword_cache_enabled, keyword_mode_from_env, KeywordMode,
};
pub use llm_extractor::{CachedKeywordExtractor, LLMKeywordExtractor};
pub use mock_extractor::MockKeywordExtractor;
