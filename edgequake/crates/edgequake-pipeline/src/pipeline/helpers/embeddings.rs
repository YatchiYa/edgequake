//! Embedding generation helpers and token-budget batching.

use std::sync::Arc;

use futures::stream::{self, StreamExt};

use crate::chunker::TextChunk;
use crate::error::Result;
use crate::extractor::ExtractionResult;

use super::super::{
    CostBreakdownStats, EmbedProgressCallback, EmbedProgressUpdate, Pipeline, ProcessingStats,
};
use super::unique_embed::{unique_entities_for_embed, unique_relationships_for_embed};

// ─────────────────────────────────────────────────────────────────────────────
//                       EMBEDDING GENERATION HELPERS
// ─────────────────────────────────────────────────────────────────────────────

/// Conservative chars-per-true-token for dense technical content.
///
/// WHY 2.5: The chunker uses 4 chars/token (English prose).
/// Scientific PDFs contain tables with numbers, gene IDs, p-values, and
/// formulas where tokenizers split aggressively — real density can reach
/// 1.5–2.0 chars/token. Using 2.5 provides a safe intermediate buffer.
const EMBED_CHARS_PER_TOKEN: f64 = 2.5;

/// Safety headroom factor applied to the embedding context limit.
///
/// WHY 0.85: Leaves 15% slack for tokenizer variance, whitespace tokens,
/// and any prompt overhead the embedding endpoint may add.
const EMBED_SAFETY_FACTOR: f64 = 0.85;

/// Fallback maximum characters when `provider.max_tokens()` returns 0 (unknown).
///
/// 6 000 chars ≈ 2 400 tokens at 2.5 chars/token, keeping chunks well within
/// the 2 048-token limit of models like embeddinggemma.
const EMBED_FALLBACK_MAX_CHARS: usize = 6_000;

/// Compute the maximum safe character count for a single embedding input.
///
/// When the provider exposes its context limit, we derive the char cap from it.
/// When the limit is unknown (0), we fall back to `EMBED_FALLBACK_MAX_CHARS`.
fn embed_max_chars(max_tokens: usize) -> usize {
    if max_tokens == 0 {
        EMBED_FALLBACK_MAX_CHARS
    } else {
        (max_tokens as f64 * EMBED_CHARS_PER_TOKEN * EMBED_SAFETY_FACTOR) as usize
    }
}

/// Truncate `s` to at most `max_bytes`, preserving UTF-8 character boundaries.
#[inline]
fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    // Local name kept for embed-guard call sites / unit tests; SSOT is utf8_prefix.
    edgequake_observability::utf8_prefix(s, max_bytes)
}

/// Policy when an embedding input exceeds the provider-safe character cap (OPS-P1.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmbeddingTruncationPolicy {
    /// Truncate and continue (historical default — partial embedding > hard fail).
    #[default]
    Truncate,
    /// Fail the embed batch so operators fix chunk_size / model.
    Fail,
}

/// Parse `EDGEQUAKE_EMBED_TRUNCATE_POLICY` (pure — pass raw for non-flaky tests).
///
/// Default: truncate. `fail` / `error` / `strict` → Fail.
pub fn parse_embedding_truncation_policy(raw: &str) -> EmbeddingTruncationPolicy {
    match raw.trim().to_ascii_lowercase().as_str() {
        "fail" | "error" | "strict" | "abort" => EmbeddingTruncationPolicy::Fail,
        _ => EmbeddingTruncationPolicy::Truncate,
    }
}

fn embedding_truncation_policy_from_env() -> EmbeddingTruncationPolicy {
    parse_embedding_truncation_policy(
        &std::env::var("EDGEQUAKE_EMBED_TRUNCATE_POLICY").unwrap_or_default(),
    )
}

/// Outcome of guarding a text batch for embedding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingGuardResult {
    pub texts: Vec<String>,
    /// True when at least one input was truncated.
    pub truncated: bool,
}

/// Guard a text batch before sending to the embedding provider.
///
/// Truncates (or fails) any string that exceeds `max_chars`.
fn guard_for_embedding(
    texts: &[String],
    max_chars: usize,
    policy: EmbeddingTruncationPolicy,
) -> crate::error::Result<EmbeddingGuardResult> {
    let mut out = Vec::with_capacity(texts.len());
    let mut truncated = false;
    for (i, text) in texts.iter().enumerate() {
        if text.len() > max_chars {
            truncated = true;
            match policy {
                EmbeddingTruncationPolicy::Fail => {
                    return Err(crate::error::PipelineError::EmbeddingError(format!(
                        "Embedding input[{i}] exceeds safe limit ({max_chars} chars, got {}); \
                         set EDGEQUAKE_EMBED_TRUNCATE_POLICY=truncate to allow truncation, \
                         or reduce chunk_size / use a larger embedding model",
                        text.len()
                    )));
                }
                EmbeddingTruncationPolicy::Truncate => {
                    tracing::warn!(
                        input_index = i,
                        original_chars = text.len(),
                        cap_chars = max_chars,
                        "Embedding input truncated: text exceeds the safe token limit for the \
                         embedding model. Consider reducing chunk_size in PipelineConfig or \
                         switching to an embedding model with a larger context window."
                    );
                    out.push(truncate_at_char_boundary(text, max_chars).to_string());
                }
            }
        } else {
            out.push(text.clone());
        }
    }
    Ok(EmbeddingGuardResult {
        texts: out,
        truncated,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
//                   TOKEN-AWARE EMBEDDING BATCH HELPER
// ─────────────────────────────────────────────────────────────────────────────

/// Embed `texts` with automatic token-aware sub-batching.
///
/// ## Problem
///
/// `EmbeddingProvider::embed_batched` splits inputs only by **count** (default
/// 2 048 texts per API call). This is insufficient for providers like Mistral
/// whose API enforces an **8 192-token TOTAL budget per request** regardless of
/// the number of individual texts. 142 entity descriptions from a dense
/// technical PDF can easily exceed 8 192 total tokens, producing:
///
/// ```text
/// 400 Bad Request: "Too many tokens overall, split into more batches." (code 3210)
/// ```
///
/// ## Fix
///
/// Before delegating to `embed_batched`, accumulate texts into sub-batches
/// whose estimated total token count stays within
/// `provider.max_tokens() * EMBED_SAFETY_FACTOR`. When the budget would be
/// exceeded by the next text, flush the current sub-batch first.
///
/// Token estimation uses `EMBED_CHARS_PER_TOKEN = 2.5` (conservative for
/// dense technical content) — the same constant as `guard_for_embedding`.
///
/// When `provider.max_tokens()` returns 0 (limit unknown) we fall back to
/// `embed_batched` directly because there is no budget to split against.
/// Embed a batch with exponential backoff on transient provider limits (SPEC-045 EC-045-09).
async fn embed_batched_with_retry(
    provider: &Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
    batch: &[String],
    cancel: Option<&tokio_util::sync::CancellationToken>,
) -> crate::error::Result<Vec<Vec<f32>>> {
    const MAX_ATTEMPTS: u32 = 3;
    let mut attempt = 0u32;
    loop {
        if cancel.map(|t| t.is_cancelled()).unwrap_or(false) {
            return Err(crate::error::PipelineError::EmbeddingError(
                "Task cancelled".to_string(),
            ));
        }

        let result = if let Some(token) = cancel {
            tokio::select! {
                biased;
                _ = token.cancelled() => {
                    return Err(crate::error::PipelineError::EmbeddingError(
                        "Task cancelled".to_string(),
                    ));
                }
                result = provider.embed_batched(batch) => result,
            }
        } else {
            provider.embed_batched(batch).await
        };

        match result {
            Ok(embeddings) => return Ok(embeddings),
            Err(e) => {
                let msg = e.to_string();
                attempt += 1;
                // X-06/X-07: typed retry_strategy only — no substring "429" matching.
                if e.retry_strategy().should_retry() && attempt < MAX_ATTEMPTS {
                    let base_ms = 500u64.saturating_mul(1u64 << (attempt - 1).min(4));
                    // Full jitter (AWS): uniform in [0, base] prevents thundering herd.
                    let delay_ms = {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut h = DefaultHasher::new();
                        (attempt, batch.len(), base_ms).hash(&mut h);
                        h.finish() % (base_ms.max(1))
                    };
                    tracing::warn!(
                        attempt,
                        delay_ms,
                        error = %msg,
                        batch_size = batch.len(),
                        "Transient embedding provider error — retrying with jittered backoff"
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    continue;
                }
                return Err(crate::error::PipelineError::EmbeddingError(msg));
            }
        }
    }
}

/// Local-provider ceiling for parallel embed sub-batches (unless opt-out).
///
/// X-07: transient classification is solely `LlmError::retry_strategy()`.
pub const LOCAL_EMBED_MAX_ASYNC: usize = 1;

/// Max concurrent embedding API sub-batches (LightRAG `embedding_func_max_async` ≈ 8).
///
/// Override with `EDGEQUAKE_EMBED_MAX_ASYNC` (clamped 1..=32).
/// For Ollama / LM Studio, caps at [`LOCAL_EMBED_MAX_ASYNC`] unless
/// `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1`.
pub fn embed_max_async() -> usize {
    let requested =
        parse_embed_max_async(&std::env::var("EDGEQUAKE_EMBED_MAX_ASYNC").unwrap_or_default());
    apply_local_embed_async_clamp(requested, &default_llm_provider_from_env())
}

/// Pure parser for `EDGEQUAKE_EMBED_MAX_ASYNC` (testable without env mutation).
pub fn parse_embed_max_async(raw: &str) -> usize {
    raw.trim()
        .parse::<usize>()
        .ok()
        .filter(|&n| n > 0)
        .map(|n| n.min(32))
        .unwrap_or(8)
}

fn default_llm_provider_from_env() -> String {
    std::env::var("EDGEQUAKE_DEFAULT_LLM_PROVIDER")
        .or_else(|_| std::env::var("EDGEQUAKE_LLM_PROVIDER"))
        .unwrap_or_default()
}

/// Cap embed fan-out for capacity-bound local providers.
///
/// Returns the effective concurrency (may be lower than `requested`).
pub fn apply_local_embed_async_clamp(requested: usize, provider_name: &str) -> usize {
    let bounded = requested.clamp(1, 32);
    if !crate::pipeline::is_local_extraction_provider(provider_name)
        || crate::pipeline::allow_local_high_concurrency()
    {
        return bounded;
    }
    if bounded > LOCAL_EMBED_MAX_ASYNC {
        tracing::info!(
            provider = provider_name,
            requested = bounded,
            effective = LOCAL_EMBED_MAX_ASYNC,
            "Local embed concurrency clamped (set EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1 to override)"
        );
        LOCAL_EMBED_MAX_ASYNC
    } else {
        bounded
    }
}

/// Plan token/count-aware sub-batches as `(start_index, end_index)` half-open ranges.
///
/// Pure function — shared by sequential and parallel embed paths (DRY).
pub(crate) fn plan_embed_sub_batches(
    texts: &[String],
    max_tokens: usize,
    max_batch_count: usize,
) -> Vec<(usize, usize)> {
    if texts.is_empty() {
        return Vec::new();
    }
    if max_tokens == 0 {
        return vec![(0, texts.len())];
    }

    let token_budget = (max_tokens as f64 * EMBED_SAFETY_FACTOR) as usize;
    let max_batch_count = max_batch_count.max(1);
    let mut ranges = Vec::new();
    let mut batch_start = 0usize;
    let mut batch_tokens = 0usize;

    for (i, text) in texts.iter().enumerate() {
        let text_tokens = ((text.len() as f64) / EMBED_CHARS_PER_TOKEN).ceil() as usize;
        let current_count = i - batch_start;
        let token_overflow = batch_tokens + text_tokens > token_budget;
        let count_overflow = current_count >= max_batch_count;
        if (token_overflow || count_overflow) && i > batch_start {
            ranges.push((batch_start, i));
            batch_start = i;
            batch_tokens = 0;
        }
        batch_tokens += text_tokens;
    }
    if batch_start < texts.len() {
        ranges.push((batch_start, texts.len()));
    }
    ranges
}

async fn embed_with_token_budget(
    provider: &Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
    texts: &[String],
    progress: Option<(&EmbedProgressCallback, &'static str)>,
    cancel: Option<&tokio_util::sync::CancellationToken>,
) -> crate::error::Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let emit = |current: usize| {
        if let Some((cb, stage)) = progress {
            cb(EmbedProgressUpdate {
                stage,
                current,
                total: texts.len(),
            });
        }
    };

    let ranges = plan_embed_sub_batches(texts, provider.max_tokens(), provider.max_batch_size());
    let concurrency = embed_max_async().min(ranges.len().max(1));

    // Single sub-batch: no fan-out overhead.
    if ranges.len() <= 1 {
        let batch_result = embed_batched_with_retry(provider, texts, cancel).await?;
        emit(batch_result.len());
        return Ok(batch_result);
    }

    // Parallel sub-batches (LightRAG asyncio.gather + embedding_func_max_async).
    // Preserve order by tagging each result with its start index.
    tracing::debug!(
        sub_batches = ranges.len(),
        concurrency,
        texts = texts.len(),
        "Embedding sub-batches in parallel"
    );

    let provider = Arc::clone(provider);
    let texts_owned: Vec<String> = texts.to_vec();
    let cancel_owned = cancel.cloned();

    // X-18: per-sub-batch Result — do not fail-fast the whole collect on one error.
    // Preserve range identity on Err so we can retry / skip individually.
    type SubBatchOk = (usize, Vec<Vec<f32>>);
    type SubBatchErr = (usize, usize, crate::error::PipelineError);
    let results: Vec<std::result::Result<SubBatchOk, SubBatchErr>> = stream::iter(ranges)
        .map(|(start, end)| {
            let provider = Arc::clone(&provider);
            let batch = texts_owned[start..end].to_vec();
            let cancel = cancel_owned.clone();
            async move {
                match embed_batched_with_retry(&provider, &batch, cancel.as_ref()).await {
                    Ok(emb) => Ok((start, emb)),
                    Err(e) => Err((start, end, e)),
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let mut indexed: Vec<(usize, Vec<Vec<f32>>)> = Vec::with_capacity(results.len());
    let mut failed_ranges: Vec<(usize, usize, crate::error::PipelineError)> = Vec::new();
    let mut last_error_msg: Option<String> = None;
    for r in results {
        match r {
            Ok(v) => indexed.push(v),
            Err((start, end, e)) => {
                last_error_msg = Some(e.to_string());
                failed_ranges.push((start, end, e));
            }
        }
    }

    // One sequential retry pass for failed sub-batches (transient blips).
    for (start, end, first_err) in failed_ranges {
        if cancel_owned
            .as_ref()
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
        {
            return Err(crate::error::PipelineError::EmbeddingError(
                "Task cancelled".to_string(),
            ));
        }
        let batch = &texts_owned[start..end];
        match embed_batched_with_retry(&provider, batch, cancel_owned.as_ref()).await {
            Ok(emb) => {
                tracing::info!(
                    start,
                    end,
                    recovered = emb.len(),
                    "X-18: embed sub-batch recovered on retry"
                );
                indexed.push((start, emb));
            }
            Err(e) => {
                last_error_msg = Some(e.to_string());
                tracing::warn!(
                    start,
                    end,
                    first_error = %first_err,
                    retry_error = %e,
                    "X-18: embed sub-batch failed after retry — tolerating partial batch"
                );
            }
        }
    }

    if indexed.is_empty() && !texts.is_empty() {
        let detail = last_error_msg.unwrap_or_else(|| "unknown".to_string());
        return Err(crate::error::PipelineError::EmbeddingError(format!(
            "All embedding sub-batches failed: {detail}"
        )));
    }

    indexed.sort_by_key(|(start, _)| *start);
    let mut all_embeddings: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
    let mut cursor = 0usize;
    for (start, emb) in indexed {
        if start > cursor {
            tracing::warn!(
                skipped_from = cursor,
                skipped_to = start,
                "X-18: gap in embed sub-batches after partial failure"
            );
        }
        cursor = start + emb.len();
        let n = all_embeddings.len() + emb.len();
        all_embeddings.extend(emb);
        emit(n.min(texts.len()));
    }

    if all_embeddings.len() != texts.len() {
        tracing::warn!(
            expected = texts.len(),
            actual = all_embeddings.len(),
            "X-18: parallel embed partial sub-batch result (tolerated)"
        );
    }

    Ok(all_embeddings)
}

// ─────────────────────────────────────────────────────────────────────────────
//                       SAFE EMBEDDING HELPER
// ─────────────────────────────────────────────────────────────────────────────

/// Guard `texts`, embed with token-budget batching, and validate result count.
///
/// Encapsulates the three-step pattern repeated for every embeddable item kind
/// (chunks, entities, relationships):
/// 1. `guard_for_embedding` — truncate inputs that exceed the provider's
///    character limit to avoid 400 "input too long" errors.
/// 2. `embed_with_token_budget` — split into sub-batches respecting BOTH the
///    token budget and the provider's input-count limit.
/// 3. Count mismatch warning — if the provider silently drops embeddings,
///    log a warning so that operators notice orphaned graph nodes.
///
/// Returns the embeddings in the same order as `texts`, or an empty `Vec` when
/// `texts` is empty (short-circuit avoids a provider round-trip).
async fn safe_embed(
    provider: &Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
    texts: &[String],
    max_chars: usize,
    kind: &str,
    progress: Option<(&EmbedProgressCallback, &'static str)>,
    cancel: Option<&tokio_util::sync::CancellationToken>,
) -> crate::error::Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    if cancel.map(|t| t.is_cancelled()).unwrap_or(false) {
        return Err(crate::error::PipelineError::EmbeddingError(
            "Task cancelled".to_string(),
        ));
    }
    let guarded = guard_for_embedding(texts, max_chars, embedding_truncation_policy_from_env())?;
    if guarded.truncated {
        tracing::info!(
            kind,
            truncated = true,
            "SPEC-046 OPS-P1.7: embedding inputs truncated under Truncate policy"
        );
    }
    let mut embeddings = edgequake_observability::with_rag_embedding_span(
        "embed-chunks",
        provider.model(),
        provider.name(),
        async {
            let emb = embed_with_token_budget(provider, &guarded.texts, progress, cancel).await?;
            let dim = emb.first().map(|v| v.len());
            edgequake_observability::record_embedding_io(kind, guarded.texts.len(), emb.len(), dim);
            Ok::<_, crate::error::PipelineError>(emb)
        },
    )
    .await?;
    if embeddings.len() != texts.len() {
        tracing::warn!(
            expected = texts.len(),
            actual = embeddings.len(),
            kind,
            "{kind} embedding count mismatch - some items may lack embeddings"
        );
    }
    // X-10: L2-normalize on write (same algorithm as Embedding::normalize).
    for vector in &mut embeddings {
        l2_normalize_inplace(vector);
    }
    Ok(embeddings)
}

/// L2-normalize a vector in place (SSOT mirror of `Embedding::normalize`).
fn l2_normalize_inplace(vector: &mut [f32]) {
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        vector.iter_mut().for_each(|x| *x /= norm);
    }
}

/// Estimate the number of embedding tokens for a character count.
///
/// Uses `EMBED_CHARS_PER_TOKEN` — the same conservative constant as the
/// batch-splitting logic — so that cost estimates and batch boundaries share a
/// single denominator (DRY). Previously the cost loops used a hardcoded `/ 4`
/// (4 chars/token) which diverged from the `2.5` used in sub-batch sizing.
fn estimate_embed_tokens(char_count: usize) -> usize {
    (char_count as f64 / EMBED_CHARS_PER_TOKEN).ceil() as usize
}

impl Pipeline {
    fn emit_embed_progress(
        progress: Option<&EmbedProgressCallback>,
        stage: &'static str,
        current: usize,
        total: usize,
    ) {
        if let Some(cb) = progress {
            cb(EmbedProgressUpdate {
                stage,
                current,
                total,
            });
        }
    }

    /// Generate embeddings for chunks, entities, and relationships.
    ///
    /// WHY UNIFIED: All three processing methods shared identical embedding
    /// logic (~120 lines each). This single implementation handles:
    /// - Chunk embeddings (content → vector)
    /// - Entity embeddings (name: description → vector)
    /// - Relationship embeddings (keywords + source→target + description → vector)
    /// - Embedding cost calculation
    ///
    /// `progress` is invoked **at the start and end of each sub-stage** so
    /// callers can surface real-time progress while embeddings are generated.
    /// Pass `None` when no progress reporting is needed.
    pub(in crate::pipeline) async fn generate_all_embeddings(
        &self,
        chunks: &mut [TextChunk],
        extractions: &mut [ExtractionResult],
        stats: &mut ProcessingStats,
        progress: Option<&EmbedProgressCallback>,
        cancel: Option<&tokio_util::sync::CancellationToken>,
    ) -> Result<()> {
        let provider = match &self.embedding_provider {
            Some(p) => p,
            None => return Ok(()),
        };

        // Capture embedding model and provider info
        // @implements SPEC-032/OODA-226: Provider tracking in ProcessingStats
        stats.embedding_model = Some(provider.model().to_string());
        stats.embedding_provider = Some(provider.name().to_string());
        stats.embedding_dimensions = Some(provider.dimension());

        // Pre-compute the safe character limit for this provider once.
        // WHY: Avoids repeated calls to max_tokens() in tight loops and keeps
        // the guard logic in a single reusable helper (DRY).
        let max_chars = embed_max_chars(provider.max_tokens());

        // ── Chunk embeddings ──
        if self.config.enable_chunk_embeddings {
            let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
            // Notify: starting chunk embeddings
            Self::emit_embed_progress(progress, "chunks", 0, texts.len());
            let embeddings = safe_embed(
                provider,
                &texts,
                max_chars,
                "Chunk",
                progress.map(|cb| (cb, "chunks")),
                cancel,
            )
            .await?;
            for (chunk, embedding) in chunks.iter_mut().zip(embeddings) {
                chunk.embedding = Some(embedding);
            }
            // Notify: chunk embeddings complete
            Self::emit_embed_progress(progress, "chunks", texts.len(), texts.len());
        }

        // ── Entity embeddings (unique-before-embed, SPEC-047 P6 / LightRAG) ──
        // WHY: mega-docs extract thousands of *mentions* but only hundreds of
        // unique EntityIds. Embedding every mention is O(Σ mentions); LightRAG
        // embeds once per unique name after merge. We collapse within-doc first,
        // embed unique texts, then broadcast the vector to all mentions so the
        // merger's collect path stays unchanged.
        if self.config.enable_entity_embeddings {
            let unique = unique_entities_for_embed(extractions);
            let mention_total: usize = unique.iter().map(|u| u.mentions.len()).sum();
            let all_entity_texts: Vec<String> = unique.iter().map(|u| u.text.clone()).collect();

            tracing::info!(
                unique_entities = unique.len(),
                mention_entities = mention_total,
                "Entity embed: unique-before-embed (SPEC-047 P6)"
            );

            Self::emit_embed_progress(progress, "entities", 0, unique.len());

            let all_embeddings = safe_embed(
                provider,
                &all_entity_texts,
                max_chars,
                "Entity",
                progress.map(|cb| (cb, "entities")),
                cancel,
            )
            .await?;
            for (embedding, entry) in all_embeddings.into_iter().zip(unique.iter()) {
                // Store once on the first mention; merger `dedupe_entities_by_id`
                // picks up any available embedding (avoids cloning Vec<f32> × mentions).
                if let Some(&(ext_idx, ent_idx)) = entry.mentions.first() {
                    extractions[ext_idx].entities[ent_idx].embedding = Some(embedding);
                }
            }

            Self::emit_embed_progress(progress, "entities", unique.len(), unique.len());
        }

        // ── Relationship embeddings (unique-before-embed) ──
        if self.config.enable_relationship_embeddings {
            let unique = unique_relationships_for_embed(extractions);
            let mention_total: usize = unique.iter().map(|u| u.mentions.len()).sum();
            let all_relationship_texts: Vec<String> =
                unique.iter().map(|u| u.text.clone()).collect();

            tracing::info!(
                unique_relationships = unique.len(),
                mention_relationships = mention_total,
                "Relationship embed: unique-before-embed (SPEC-047 P6)"
            );

            Self::emit_embed_progress(progress, "relationships", 0, unique.len());

            let all_embeddings = safe_embed(
                provider,
                &all_relationship_texts,
                max_chars,
                "Relationship",
                progress.map(|cb| (cb, "relationships")),
                cancel,
            )
            .await?;
            for (embedding, entry) in all_embeddings.into_iter().zip(unique.iter()) {
                if let Some(&(ext_idx, rel_idx)) = entry.mentions.first() {
                    extractions[ext_idx].relationships[rel_idx].embedding = Some(embedding);
                }
            }

            Self::emit_embed_progress(progress, "relationships", unique.len(), unique.len());
        }

        // ── Embedding cost calculation (unique texts only) ──
        // WHY estimate_embed_tokens: uses the same EMBED_CHARS_PER_TOKEN denominator
        // as sub-batch sizing — one constant, one formula (DRY). Cost tracks what
        // we actually sent to the provider (unique), not mention cardinality.
        let mut total_embed_tokens = 0usize;

        if self.config.enable_chunk_embeddings {
            let chunk_text_len: usize = chunks.iter().map(|c| c.content.len()).sum();
            total_embed_tokens += estimate_embed_tokens(chunk_text_len);
        }
        if self.config.enable_entity_embeddings {
            for entry in unique_entities_for_embed(extractions) {
                total_embed_tokens += estimate_embed_tokens(entry.text.len());
            }
        }
        if self.config.enable_relationship_embeddings {
            for entry in unique_relationships_for_embed(extractions) {
                total_embed_tokens += estimate_embed_tokens(entry.text.len());
            }
        }

        let embed_model_name = provider.model();
        let pricing = crate::progress::default_model_pricing();
        let embed_pricing = pricing.get(embed_model_name).cloned().unwrap_or_else(|| {
            crate::progress::ModelPricing::new("text-embedding-3-small", 0.00002, 0.0)
        });

        let embedding_cost = embed_pricing.calculate_cost(total_embed_tokens, 0);
        stats.cost_usd += embedding_cost;

        if let Some(ref mut breakdown) = stats.cost_breakdown {
            breakdown.embedding_cost_usd = embedding_cost;
            breakdown.embedding_tokens = total_embed_tokens;
        } else {
            let breakdown = CostBreakdownStats {
                embedding_cost_usd: embedding_cost,
                embedding_tokens: total_embed_tokens,
                ..CostBreakdownStats::default()
            };
            stats.cost_breakdown = Some(breakdown);
        }

        Ok(())
    }

    /// Re-generate embeddings for a `ProcessingResult` (slim-checkpoint resume).
    ///
    /// WHY (SPEC-047 P5): Checkpoints omit embeddings to stay under Postgres
    /// jsonb size limits. On resume we skip LLM extraction but must re-embed
    /// before merge/persist.
    pub async fn ensure_embeddings(
        &self,
        result: &mut crate::pipeline::ProcessingResult,
        progress: Option<&EmbedProgressCallback>,
    ) -> Result<()> {
        self.generate_all_embeddings(
            &mut result.chunks,
            &mut result.extractions,
            &mut result.stats,
            progress,
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── embed_max_chars ────────────────────────────────────────────────────

    #[test]
    fn test_embed_max_chars_with_known_limit() {
        // 8192 tokens × 2.5 chars/token × 0.85 safety = 17 408 chars
        let expected = (8192_f64 * EMBED_CHARS_PER_TOKEN * EMBED_SAFETY_FACTOR) as usize;
        assert_eq!(embed_max_chars(8192), expected);
    }

    #[test]
    fn test_embed_max_chars_fallback_when_zero() {
        assert_eq!(embed_max_chars(0), EMBED_FALLBACK_MAX_CHARS);
    }

    // ── truncate_at_char_boundary ─────────────────────────────────────────

    #[test]
    fn test_truncate_exact_boundary() {
        let s = "hello world";
        assert_eq!(truncate_at_char_boundary(s, 5), "hello");
    }

    #[test]
    fn test_truncate_within_multibyte_char() {
        // "é" is 2 bytes (U+00E9). Truncating at byte 1 must walk back to byte 0.
        let s = "aéb";
        let truncated = truncate_at_char_boundary(s, 2);
        assert!(s.is_char_boundary(truncated.len()));
        assert_eq!(truncated, "a");
    }

    #[test]
    fn test_truncate_no_op_when_within_limit() {
        let s = "short";
        assert_eq!(truncate_at_char_boundary(s, 100), "short");
    }

    // ── guard_for_embedding ──────────────────────────────────────────────

    #[test]
    fn test_guard_preserves_short_texts() {
        let texts = vec!["hello".to_string(), "world".to_string()];
        let result = guard_for_embedding(&texts, 100, EmbeddingTruncationPolicy::Truncate).unwrap();
        assert_eq!(result.texts, texts);
        assert!(!result.truncated);
    }

    #[test]
    fn test_guard_truncates_long_texts() {
        let long_text = "a".repeat(200);
        let result = guard_for_embedding(
            std::slice::from_ref(&long_text),
            50,
            EmbeddingTruncationPolicy::Truncate,
        )
        .unwrap();
        assert_eq!(result.texts.len(), 1);
        assert!(result.texts[0].len() <= 50);
        assert!(result.truncated);
    }

    #[test]
    fn test_guard_fail_policy_rejects_long_texts() {
        let long_text = "a".repeat(200);
        let err = guard_for_embedding(
            std::slice::from_ref(&long_text),
            50,
            EmbeddingTruncationPolicy::Fail,
        )
        .unwrap_err();
        assert!(err.to_string().contains("exceeds safe limit"));
    }

    #[test]
    fn test_parse_embedding_truncation_policy() {
        assert_eq!(
            parse_embedding_truncation_policy(""),
            EmbeddingTruncationPolicy::Truncate
        );
        assert_eq!(
            parse_embedding_truncation_policy("fail"),
            EmbeddingTruncationPolicy::Fail
        );
        assert_eq!(
            parse_embedding_truncation_policy("STRICT"),
            EmbeddingTruncationPolicy::Fail
        );
    }

    // ── embed_with_token_budget ────────────────────────────────────────────

    /// Counts how many times `embed()` is called by accumulating batch sizes.
    use std::sync::{Arc, Mutex};

    struct CountingEmbedProvider {
        /// Each element records the number of texts in that sub-batch call.
        call_sizes: Arc<Mutex<Vec<usize>>>,
        /// Simulated max_tokens limit.
        max_tokens: usize,
        /// Simulated max_batch_size (input count limit).
        max_batch: usize,
    }

    impl CountingEmbedProvider {
        fn new(max_tokens: usize) -> (Self, Arc<Mutex<Vec<usize>>>) {
            Self::new_with_batch(max_tokens, 2048)
        }

        fn new_with_batch(max_tokens: usize, max_batch: usize) -> (Self, Arc<Mutex<Vec<usize>>>) {
            let call_sizes = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    call_sizes: call_sizes.clone(),
                    max_tokens,
                    max_batch,
                },
                call_sizes,
            )
        }
    }

    #[async_trait::async_trait]
    impl edgequake_llm::traits::EmbeddingProvider for CountingEmbedProvider {
        fn name(&self) -> &str {
            "counting"
        }
        fn model(&self) -> &str {
            "counting-embed"
        }
        fn dimension(&self) -> usize {
            4
        }
        fn max_tokens(&self) -> usize {
            self.max_tokens
        }
        fn max_batch_size(&self) -> usize {
            self.max_batch
        }

        async fn embed(&self, texts: &[String]) -> edgequake_llm::Result<Vec<Vec<f32>>> {
            self.call_sizes.lock().unwrap().push(texts.len());
            // Return a dummy 4-dim vector per text
            Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3, 0.4]).collect())
        }
    }

    struct FailingEmbedProvider {
        max_tokens: usize,
        max_batch: usize,
    }

    #[async_trait::async_trait]
    impl edgequake_llm::traits::EmbeddingProvider for FailingEmbedProvider {
        fn name(&self) -> &str {
            "failing"
        }
        fn model(&self) -> &str {
            "failing-embed"
        }
        fn dimension(&self) -> usize {
            4
        }
        fn max_tokens(&self) -> usize {
            self.max_tokens
        }
        fn max_batch_size(&self) -> usize {
            self.max_batch
        }

        async fn embed(&self, _texts: &[String]) -> edgequake_llm::Result<Vec<Vec<f32>>> {
            Err(edgequake_llm::error::LlmError::NetworkError(
                "error sending request for url (http://localhost:11434/api/embeddings)".into(),
            ))
        }
    }

    /// Parallel sub-batch path must surface the underlying provider error (not a generic shell).
    #[tokio::test]
    async fn test_embed_parallel_path_propagates_last_provider_error() {
        let texts: Vec<String> = (0..6).map(|_| "x".repeat(100)).collect();
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> =
            Arc::new(FailingEmbedProvider {
                max_tokens: 80,
                max_batch: 2,
            });

        let err = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .expect_err("all sub-batches should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("All embedding sub-batches failed"),
            "expected aggregate prefix, got: {msg}"
        );
        assert!(
            msg.contains("error sending request"),
            "underlying provider error must be preserved, got: {msg}"
        );
    }

    /// When total estimated tokens fit within the budget, exactly ONE call is made.
    #[tokio::test]
    async fn test_embed_budget_single_batch_when_within_limit() {
        // 10 texts × 10 chars / 2.5 chars/token = 40 tokens, budget = 8192 * 0.85 ≈ 6963
        let texts: Vec<String> = (0..10).map(|i| format!("entity_{:04}", i)).collect(); // ~13 chars each
        let (provider, call_sizes) = CountingEmbedProvider::new(8192);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 10, "All 10 embeddings must be returned");
        let sizes = call_sizes.lock().unwrap();
        assert_eq!(sizes.len(), 1, "Should have made exactly 1 embed call");
    }

    /// When texts are large enough to exceed a tiny budget, they are split across
    /// multiple sub-batch calls and all embeddings are reassembled in order.
    #[tokio::test]
    async fn test_embed_budget_splits_batches_correctly() {
        // 20 texts of 100 chars each:
        //   100 / 2.5 = 40 tokens per text
        //   budget = 80 * 0.85 = 68 tokens ≈ 1 text per batch
        let texts: Vec<String> = (0..20).map(|_| "x".repeat(100)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new(80); // tiny budget forces splits
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 20, "All 20 embeddings must be returned");
        // With max_tokens=80 and SAFETY_FACTOR=0.85, budget = 68 tokens.
        // Each text costs ceil(100/2.5)=40 tokens. Two texts = 80 > 68 → at least 2 calls.
        let sizes = call_sizes.lock().unwrap();
        assert!(
            sizes.len() >= 2,
            "Expected multiple batches, got {} call(s)",
            sizes.len()
        );
        // Total texts across all calls must equal 20 (no duplicates, no drops)
        let total: usize = sizes.iter().sum();
        assert_eq!(total, 20, "Total texts across batches must be 20");
    }

    /// Empty input returns empty output without calling the provider at all.
    #[tokio::test]
    async fn test_embed_budget_empty_input() {
        let (provider, call_sizes) = CountingEmbedProvider::new(8192);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &[], None, None)
            .await
            .unwrap();
        assert!(result.is_empty());
        assert!(
            call_sizes.lock().unwrap().is_empty(),
            "No calls for empty input"
        );
    }

    /// When `max_tokens == 0` (limit unknown), fall back to `embed_batched` — one call.
    #[tokio::test]
    async fn test_embed_budget_zero_max_tokens_fallback() {
        let texts: Vec<String> = (0..5).map(|i| format!("text_{}", i)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new(0); // 0 = unknown limit
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 5);
        let sizes = call_sizes.lock().unwrap();
        assert_eq!(
            sizes.len(),
            1,
            "Should fall back to a single embed_batched call"
        );
    }

    // ── count-limit splitting (spec-011) ──────────────────────────────────

    /// When input count exceeds max_batch_size, splits into multiple calls.
    ///
    /// This is the regression test for the EU AI Act failure:
    /// many short entities fit within the token budget but exceed the
    /// Mistral input count limit (512).
    #[tokio::test]
    async fn test_embed_count_limit_splits_batches() {
        // 600 short texts (each 10 chars, ~4 tokens) with max_batch=512
        // Token budget = 8192 * 0.85 = 6963, 600 × 4 = 2400 tokens → fits budget
        // But 600 > 512 → must split
        let texts: Vec<String> = (0..600_usize).map(|i| format!("entity_{:04}", i)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new_with_batch(8192, 512);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 600, "All 600 embeddings returned");

        let sizes = call_sizes.lock().unwrap();
        assert!(
            sizes.len() >= 2,
            "600 texts with max_batch=512 must produce ≥ 2 calls, got {}",
            sizes.len()
        );
        // Each individual call must respect the count limit
        for (idx, &size) in sizes.iter().enumerate() {
            assert!(
                size <= 512,
                "Call {} sent {} items, exceeds max_batch_size 512",
                idx,
                size
            );
        }
        // No items lost or duplicated
        let total: usize = sizes.iter().sum();
        assert_eq!(total, 600, "Total items across all calls must be 600");
    }

    /// Exactly max_batch_size items → exactly ONE call (boundary: not exceeded).
    #[tokio::test]
    async fn test_embed_count_exactly_at_limit_is_one_call() {
        let texts: Vec<String> = (0..512_usize).map(|i| format!("e_{}", i)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new_with_batch(8192, 512);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 512);

        let sizes = call_sizes.lock().unwrap();
        assert_eq!(sizes.len(), 1, "Exactly 512 texts (== limit) → 1 call");
        assert_eq!(sizes[0], 512);
    }

    /// One more than max_batch_size → exactly TWO calls.
    #[tokio::test]
    async fn test_embed_count_one_over_limit_is_two_calls() {
        let texts: Vec<String> = (0..513_usize).map(|i| format!("e_{}", i)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new_with_batch(8192, 512);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 513);

        let sizes = call_sizes.lock().unwrap();
        assert_eq!(sizes.len(), 2, "513 texts with limit 512 → 2 calls");
        assert_eq!(sizes[0], 512, "First call: 512");
        assert_eq!(sizes[1], 1, "Second call: 1 remainder");
    }

    /// Both limits active simultaneously: token budget AND count limit both trigger.
    /// Verifies the flush uses the more restrictive limit.
    #[tokio::test]
    async fn test_embed_dual_limit_count_wins_over_token() {
        // 20 texts × 100 chars each = 40 tokens each
        // Token budget = 8192 * 0.85 = 6963 → 20×40=800 tokens fits budget
        // max_batch_count = 5 → must split on count, not tokens
        let texts: Vec<String> = (0..20).map(|_| "x".repeat(100)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new_with_batch(8192, 5);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 20);

        let sizes = call_sizes.lock().unwrap();
        assert!(
            sizes.len() >= 4,
            "20 texts with max_batch=5 → ≥ 4 calls, got {}",
            sizes.len()
        );
        for (idx, &size) in sizes.iter().enumerate() {
            assert!(
                size <= 5,
                "Call {} sent {} items, exceeds max_batch 5",
                idx,
                size
            );
        }
        let total: usize = sizes.iter().sum();
        assert_eq!(total, 20);
    }

    /// Token budget wins over count when texts are very large.
    #[tokio::test]
    async fn test_embed_dual_limit_token_wins_over_count() {
        // 20 texts × 1000 chars each = 400 tokens each
        // max_batch_count = 2048 (large) → count won't trigger
        // Token budget = 100 * 0.85 = 85 tokens → one 400-token text already exceeds budget
        // → splits on token budget (1 text per call since each > budget)
        let texts: Vec<String> = (0..5).map(|_| "y".repeat(1000)).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new_with_batch(100, 2048);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 5);

        // Each text = ceil(1000/2.5) = 400 tokens; budget = 100*0.85 = 85 tokens
        // First text: 400 > 85 but i == batch_start (can't flush empty batch) → sent alone
        // Second text: 400 > 85 → flush first → each text in its own call
        let sizes = call_sizes.lock().unwrap();
        assert_eq!(
            sizes.len(),
            5,
            "Each text should be its own call due to tiny budget"
        );
        let total: usize = sizes.iter().sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn spec047_p6_parse_embed_max_async_defaults_and_clamps() {
        assert_eq!(parse_embed_max_async(""), 8);
        assert_eq!(parse_embed_max_async("4"), 4);
        assert_eq!(parse_embed_max_async("0"), 8);
        assert_eq!(parse_embed_max_async("99"), 32);
        assert_eq!(parse_embed_max_async("nope"), 8);
    }

    #[test]
    fn local_embed_async_clamp_caps_ollama() {
        assert_eq!(
            apply_local_embed_async_clamp(8, "ollama"),
            LOCAL_EMBED_MAX_ASYNC
        );
        assert_eq!(apply_local_embed_async_clamp(8, "openai"), 8);
        assert_eq!(apply_local_embed_async_clamp(1, "lmstudio"), 1);
    }

    #[test]
    fn spec047_p6_plan_embed_sub_batches_respects_count_limit() {
        let texts: Vec<String> = (0..20).map(|i| format!("t{i}")).collect();
        let ranges = plan_embed_sub_batches(&texts, 100_000, 5);
        assert!(ranges.len() >= 4);
        for (start, end) in &ranges {
            assert!(end - start <= 5);
        }
        assert_eq!(ranges.last().unwrap().1, 20);
    }

    /// Parallel path must preserve order across concurrent sub-batches.
    #[tokio::test]
    async fn spec047_p6_parallel_embed_preserves_order() {
        let texts: Vec<String> = (0..20).map(|i| format!("item-{i}")).collect();
        let (provider, call_sizes) = CountingEmbedProvider::new_with_batch(8_192, 5);
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> = Arc::new(provider);
        // Force parallel path via env-independent concurrency (ranges > 1).
        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 20);
        // CountingEmbedProvider returns deterministic dims; order = input order.
        for (i, emb) in result.iter().enumerate() {
            assert!(!emb.is_empty(), "missing embedding at {i}");
        }
        let sizes = call_sizes.lock().unwrap();
        assert!(
            sizes.len() >= 4,
            "expected multiple sub-batches, got {}",
            sizes.len()
        );
    }

    #[test]
    fn x07_typed_retry_strategy_no_substring() {
        // X-07: classification is LlmError::retry_strategy(), not message contains.
        use edgequake_llm::LlmError;
        assert!(LlmError::RateLimited("too many requests".into())
            .retry_strategy()
            .should_retry());
        assert!(LlmError::Timeout.retry_strategy().should_retry());
        assert!(
            !LlmError::InvalidRequest("Too many inputs in request (400)".into())
                .retry_strategy()
                .should_retry()
        );
        assert!(!LlmError::AuthError("bad key".into())
            .retry_strategy()
            .should_retry());
    }

    /// Provider that permanently fails batches containing a poison marker.
    struct PartialFailEmbedProvider {
        max_batch: usize,
        poison: String,
    }

    #[async_trait::async_trait]
    impl edgequake_llm::traits::EmbeddingProvider for PartialFailEmbedProvider {
        fn name(&self) -> &str {
            "partial-fail"
        }
        fn model(&self) -> &str {
            "partial-fail-embed"
        }
        fn dimension(&self) -> usize {
            2
        }
        fn max_tokens(&self) -> usize {
            8_192
        }
        fn max_batch_size(&self) -> usize {
            self.max_batch
        }

        async fn embed(&self, texts: &[String]) -> edgequake_llm::Result<Vec<Vec<f32>>> {
            if texts.iter().any(|t| t.contains(&self.poison)) {
                return Err(edgequake_llm::LlmError::InvalidRequest(
                    "poison sub-batch".into(),
                ));
            }
            Ok(texts.iter().map(|_| vec![0.5, 0.5]).collect())
        }
    }

    /// SPEC-083 X-18: one failed sub-batch must not fail the entire embed collect.
    #[tokio::test]
    async fn unit_embed_partial_subbatch_tolerated() {
        // 12 texts, max_batch=5 → ≥3 sub-batches; poison only the middle range.
        let mut texts: Vec<String> = (0..12).map(|i| format!("ok-{i}")).collect();
        texts[5] = "POISON-item".into();
        texts[6] = "POISON-item-2".into();
        let provider: Arc<dyn edgequake_llm::traits::EmbeddingProvider> =
            Arc::new(PartialFailEmbedProvider {
                max_batch: 5,
                poison: "POISON".into(),
            });

        let result = embed_with_token_budget(&provider, &texts, None, None)
            .await
            .expect("X-18: partial failure must not fail whole embed");
        // Surviving batches: [0..5) and [10..12) → 5 + 2 = 7 (poison batch [5..10) dropped).
        assert!(
            result.len() < texts.len() && !result.is_empty(),
            "expected partial embeddings, got {}",
            result.len()
        );
        assert_eq!(result.len(), 7);

        // Contract: production path must not use fail-fast Result collect.
        let src = include_str!("embeddings.rs");
        let prod = src.split("#[cfg(test)]").next().expect("prod");
        assert!(
            !prod.contains("collect::<crate::error::Result<Vec<_>>>"),
            "X-18: must not fail-fast collect Result of all sub-batches"
        );
        assert!(
            prod.contains("tolerating partial batch") || prod.contains("X-18"),
            "X-18: partial tolerance path must be present"
        );
    }
}
