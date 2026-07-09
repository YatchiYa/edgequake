//! Semantic vector chunking (SPEC-046 EQ-046-09 / LightRAG strategy `V`).
//!
//! First principle: split where **meaning** changes, not where a token budget
//! happens to land. Algorithm (LangChain SemanticChunker / LightRAG V):
//!
//! 1. Split text into sentences
//! 2. Embed each sentence (optional buffer window)
//! 3. Compute consecutive cosine distances
//! 4. Break where distance ≥ percentile threshold (default 95th)
//! 5. Hard-split oversized groups with recursive character chunking
//!
//! SOLID: pure distance/breakpoint logic is provider-free; the strategy only
//! depends on `EmbeddingProvider` at the boundary (DIP).

use std::sync::Arc;

use async_trait::async_trait;
use edgequake_llm::traits::EmbeddingProvider;

use super::recursive::RecursiveCharacterChunking;
use super::text_utils::{estimate_tokens, split_into_sentences};
use super::types::{ChunkResult, ChunkerConfig, ChunkingStrategy};
use crate::error::{PipelineError, Result};

/// Breakpoint threshold strategy (LightRAG / LangChain parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakpointThreshold {
    #[default]
    Percentile,
    StandardDeviation,
    Interquartile,
}

impl BreakpointThreshold {
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_SEMANTIC_BREAKPOINT")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "std" | "standard_deviation" | "stddev" => Self::StandardDeviation,
            "iqr" | "interquartile" => Self::Interquartile,
            _ => Self::Percentile,
        }
    }
}

/// Tunables for semantic chunking.
#[derive(Debug, Clone)]
pub struct SemanticChunkConfig {
    pub threshold: BreakpointThreshold,
    /// Percentile (0–100), std-dev multiplier, or IQR multiplier depending on type.
    pub threshold_amount: f32,
    /// Adjacent sentences included in each embedding window (LightRAG buffer_size).
    pub buffer_size: usize,
}

impl Default for SemanticChunkConfig {
    fn default() -> Self {
        Self {
            threshold: BreakpointThreshold::from_env(),
            threshold_amount: std::env::var("EDGEQUAKE_SEMANTIC_BREAKPOINT_AMOUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(95.0),
            buffer_size: std::env::var("EDGEQUAKE_SEMANTIC_BUFFER_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1)
                .clamp(0, 5),
        }
    }
}

/// Cosine distance = 1 − cosine similarity. Returns 0 for zero vectors.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 1.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 1.0;
    }
    let sim = (dot / (na.sqrt() * nb.sqrt())).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Compute breakpoint threshold from consecutive distances.
pub fn breakpoint_threshold(distances: &[f32], kind: BreakpointThreshold, amount: f32) -> f32 {
    if distances.is_empty() {
        return f32::MAX;
    }
    match kind {
        BreakpointThreshold::Percentile => {
            let mut sorted = distances.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p = (amount.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f32;
            let idx = p.floor() as usize;
            let frac = p - idx as f32;
            if idx + 1 < sorted.len() {
                sorted[idx] * (1.0 - frac) + sorted[idx + 1] * frac
            } else {
                sorted[idx]
            }
        }
        BreakpointThreshold::StandardDeviation => {
            let mean = distances.iter().sum::<f32>() / distances.len() as f32;
            let var = distances.iter().map(|d| (d - mean).powi(2)).sum::<f32>()
                / distances.len() as f32;
            mean + amount.max(0.0) * var.sqrt()
        }
        BreakpointThreshold::Interquartile => {
            let mut sorted = distances.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let q1 = percentile_of_sorted(&sorted, 25.0);
            let q3 = percentile_of_sorted(&sorted, 75.0);
            let iqr = q3 - q1;
            q3 + amount.max(0.0) * iqr
        }
    }
}

fn percentile_of_sorted(sorted: &[f32], pct: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let p = (pct.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f32;
    let idx = p.floor() as usize;
    let frac = p - idx as f32;
    if idx + 1 < sorted.len() {
        sorted[idx] * (1.0 - frac) + sorted[idx + 1] * frac
    } else {
        sorted[idx]
    }
}

/// Group sentence indices into chunks given distances and a threshold.
///
/// Break **after** sentence `i` when `distances[i] >= threshold`.
pub fn group_by_breakpoints(n_sentences: usize, distances: &[f32], threshold: f32) -> Vec<Vec<usize>> {
    if n_sentences == 0 {
        return Vec::new();
    }
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for i in 0..n_sentences {
        current.push(i);
        let is_break = i < distances.len() && distances[i] >= threshold;
        if is_break || i + 1 == n_sentences {
            groups.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

/// Build buffered sentence windows for embedding (buffer_size neighbors).
pub fn buffered_windows(sentences: &[String], buffer_size: usize) -> Vec<String> {
    let n = sentences.len();
    (0..n)
        .map(|i| {
            let start = i.saturating_sub(buffer_size);
            let end = (i + buffer_size + 1).min(n);
            sentences[start..end].join(" ")
        })
        .collect()
}

/// Semantic chunking strategy. Falls back to recursive when embedder is absent.
pub struct SemanticChunking {
    embedder: Option<Arc<dyn EmbeddingProvider>>,
    semantic: SemanticChunkConfig,
    fallback: RecursiveCharacterChunking,
}

impl SemanticChunking {
    pub fn new(embedder: Option<Arc<dyn EmbeddingProvider>>) -> Self {
        Self {
            embedder,
            semantic: SemanticChunkConfig::default(),
            fallback: RecursiveCharacterChunking,
        }
    }

    pub fn with_config(
        embedder: Option<Arc<dyn EmbeddingProvider>>,
        semantic: SemanticChunkConfig,
    ) -> Self {
        Self {
            embedder,
            semantic,
            fallback: RecursiveCharacterChunking,
        }
    }

    async fn chunk_semantic(
        &self,
        content: &str,
        config: &ChunkerConfig,
        embedder: &dyn EmbeddingProvider,
    ) -> Result<Vec<ChunkResult>> {
        let sentences = split_into_sentences(content);
        if sentences.is_empty() {
            return Ok(Vec::new());
        }
        if sentences.len() == 1 {
            return self
                .fallback
                .chunk(content, config)
                .await
                .map(|mut v| {
                    for (i, c) in v.iter_mut().enumerate() {
                        c.chunk_order_index = i;
                    }
                    v
                });
        }

        let windows = buffered_windows(&sentences, self.semantic.buffer_size);
        let embeddings = embedder
            .embed(&windows)
            .await
            .map_err(|e| PipelineError::EmbeddingError(e.to_string()))?;

        if embeddings.len() != sentences.len() {
            tracing::warn!(
                expected = sentences.len(),
                got = embeddings.len(),
                "semantic chunk: embedding count mismatch; falling back to recursive"
            );
            return self.fallback.chunk(content, config).await;
        }

        let distances: Vec<f32> = embeddings
            .windows(2)
            .map(|w| cosine_distance(&w[0], &w[1]))
            .collect();
        let threshold = breakpoint_threshold(
            &distances,
            self.semantic.threshold,
            self.semantic.threshold_amount,
        );
        let groups = group_by_breakpoints(sentences.len(), &distances, threshold);

        let mut results = Vec::new();
        let mut order = 0usize;
        let mut cursor = 0usize;

        for group in groups {
            if group.is_empty() {
                continue;
            }
            let text = group
                .iter()
                .map(|&i| sentences[i].as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let start = content[cursor..]
                .find(sentences[*group.first().unwrap()].as_str())
                .map(|o| cursor + o)
                .unwrap_or(cursor);
            let end = start + text.len().min(content.len().saturating_sub(start));
            cursor = end;

            // Hard-split oversized semantic groups (LightRAG advisory max)
            if estimate_tokens(&text) > config.chunk_size {
                let sub = self.fallback.chunk(&text, config).await?;
                for mut piece in sub {
                    piece.chunk_order_index = order;
                    piece.start_offset = Some(start);
                    piece.end_offset = Some(end);
                    results.push(piece);
                    order += 1;
                }
            } else {
                results.push(ChunkResult {
                    content: text.clone(),
                    tokens: estimate_tokens(&text),
                    chunk_order_index: order,
                    section: None,
                    start_offset: Some(start),
                    end_offset: Some(end),
                    page_start: None,
                    page_end: None,
                });
                order += 1;
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl ChunkingStrategy for SemanticChunking {
    async fn chunk(&self, content: &str, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
        match &self.embedder {
            Some(embedder) => self.chunk_semantic(content, config, embedder.as_ref()).await,
            None => {
                tracing::warn!(
                    "ChunkStrategy::Semantic requested without embedding provider; falling back to recursive"
                );
                self.fallback.chunk(content, config).await
            }
        }
    }

    fn name(&self) -> &str {
        "semantic_vector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_distance_identical_is_zero() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_distance(&v, &v) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_distance_orthogonal_is_one() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn percentile_threshold_splits_outlier() {
        let distances = vec![0.1, 0.1, 0.9, 0.1];
        let t = breakpoint_threshold(&distances, BreakpointThreshold::Percentile, 75.0);
        assert!(t < 0.9);
        let groups = group_by_breakpoints(5, &distances, 0.5);
        // Break after index 2 (distance 0.9)
        assert!(groups.len() >= 2);
    }

    #[test]
    fn group_single_sentence() {
        let groups = group_by_breakpoints(1, &[], 0.5);
        assert_eq!(groups, vec![vec![0]]);
    }

    #[test]
    fn buffered_windows_respect_edges() {
        let s = vec!["a".into(), "b".into(), "c".into()];
        let w = buffered_windows(&s, 1);
        assert_eq!(w[0], "a b");
        assert_eq!(w[1], "a b c");
        assert_eq!(w[2], "b c");
    }
}
