//! RAG / GenAI tracing spans (SPEC-046 / SPEC-124 / OTel GenAI conventions).
//!
//! Always emits `tracing` spans (works without OTLP). Attribute names follow
//! OpenTelemetry GenAI + Langfuse observation types.
//!
//! LAW-124-12: record token usage; never emit cost attributes.
//! LAW-124-16..18: observation I/O via SSOT helpers Langfuse actually maps.
//! SOLID: observability owns attribute mapping; query/API only call helpers.

use std::future::Future;

use tracing::Instrument;

use crate::langfuse_attrs::{
    GEN_AI_COMPLETION, GEN_AI_PROMPT, GEN_AI_USAGE_INPUT_TOKENS, GEN_AI_USAGE_OUTPUT_TOKENS,
    LANGFUSE_OBSERVATION_INPUT, LANGFUSE_OBSERVATION_OUTPUT, LANGFUSE_TRACE_TAGS,
    OBSERVATION_TYPE_CHAIN, OBSERVATION_TYPE_EMBEDDING, OBSERVATION_TYPE_GENERATION,
    OBSERVATION_TYPE_RETRIEVER, OBSERVATION_TYPE_SPAN,
};

/// Default byte budget for observation I/O previews (PII-safe).
pub const OBSERVATION_IO_PREVIEW_CHARS: usize = 512;

/// Attributes for a retrieval-phase span.
#[derive(Debug, Clone, Default)]
pub struct RagRetrievalAttrs {
    pub data_source_id: Option<&'static str>,
    pub top_k: Option<usize>,
    pub arm: Option<&'static str>,
    pub mode: Option<&'static str>,
    pub query_preview: Option<String>,
}

/// Run `fut` inside a GenAI retrieval span (Langfuse observation type `retriever`).
///
/// Sets `langfuse.observation.input` at span start (Langfuse UI Input) — not only
/// `gen_ai.retrieval.query.text`, which Langfuse does **not** map to Input.
pub async fn with_rag_retrieval_span<Fut, T>(attrs: RagRetrievalAttrs, fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    let data_source = attrs.data_source_id.unwrap_or("edgequake");
    let span_name = format!("retrieval {data_source}");
    let query_in = attrs.query_preview.as_deref().unwrap_or("");
    let span = tracing::info_span!(
        "rag.retrieval",
        otel.name = %span_name,
        otel.kind = "client",
        gen_ai.operation.name = "retrieval",
        gen_ai.data_source.id = %data_source,
        gen_ai.retrieval.top_k = attrs.top_k.map(|k| k as i64).unwrap_or(0),
        rag.retrieval.arm = attrs.arm.unwrap_or(""),
        rag.query.mode = attrs.mode.unwrap_or(""),
        rag.retrieval.empty_result = tracing::field::Empty,
        rag.context.truncated = tracing::field::Empty,
        rag.retrieval.fallback = tracing::field::Empty,
        gen_ai.retrieval.query.text = %query_in,
        langfuse.observation.type = OBSERVATION_TYPE_RETRIEVER,
        langfuse.observation.input = %query_in,
        langfuse.observation.output = tracing::field::Empty,
        gen_ai.prompt = %query_in,
        gen_ai.completion = tracing::field::Empty,
    );
    fut.instrument(span).await
}

/// Record post-retrieval flags on the current span.
pub fn record_rag_retrieval_outcome(empty: bool, truncated: bool, fallback: Option<&str>) {
    let span = tracing::Span::current();
    span.record("rag.retrieval.empty_result", empty);
    span.record("rag.context.truncated", truncated);
    if let Some(fb) = fallback {
        span.record("rag.retrieval.fallback", fb);
    }
}

/// Record retriever observation output as compact JSON (LAW-124-16).
///
/// Call after retrieval alongside [`record_rag_retrieval_outcome`].
pub fn record_rag_retrieval_io(
    empty: bool,
    chunk_count: usize,
    entity_count: usize,
    preview: Option<&str>,
) {
    let mut out =
        format!("{{\"empty\":{empty},\"chunks\":{chunk_count},\"entities\":{entity_count}}}");
    if let Some(p) = preview.filter(|s| !s.is_empty()) {
        let clipped = query_preview(p, 128);
        let escaped = escape_json_string(&clipped);
        out = format!(
            "{{\"empty\":{empty},\"chunks\":{chunk_count},\"entities\":{entity_count},\"preview\":\"{escaped}\"}}"
        );
    }
    record_observation_io(None, Some(&out));
}

/// Outcome flags + retriever I/O in one call (DRY for arm_timed / query_pipeline).
pub fn record_rag_retrieval_complete(
    empty: bool,
    truncated: bool,
    fallback: Option<&str>,
    chunk_count: usize,
    entity_count: usize,
    preview: Option<&str>,
) {
    record_rag_retrieval_outcome(empty, truncated, fallback);
    record_rag_retrieval_io(empty, chunk_count, entity_count, preview);
}

/// Token + I/O snapshot recorded after a successful LLM generation (LAW-124-12/16).
#[derive(Debug, Clone, Default)]
pub struct LlmGenerationRecord {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl LlmGenerationRecord {
    /// Build from a typical LLM response (`prompt_tokens` / `completion_tokens` / content).
    pub fn from_response(
        input: Option<&str>,
        output: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Self {
        Self {
            input_tokens: Some(prompt_tokens),
            output_tokens: Some(completion_tokens),
            input: input.map(str::to_string),
            output: Some(output.to_string()),
        }
    }

    /// Apply usage + I/O onto the current generation span.
    pub fn record_on_current_span(&self) {
        record_gen_ai_usage(self.input_tokens, self.output_tokens);
        record_observation_io(self.input.as_deref(), self.output.as_deref());
    }
}

/// Run `fut` inside a GenAI chat/generation span.
pub async fn with_rag_generation_span<Fut, T>(
    operation: &str,
    model: &str,
    provider: &str,
    fut: Fut,
) -> T
where
    Fut: Future<Output = T>,
{
    let span = tracing::info_span!(
        "rag.generation",
        otel.name = %operation,
        otel.kind = "client",
        gen_ai.operation.name = "chat",
        gen_ai.request.model = %model,
        gen_ai.provider.name = %provider,
        langfuse.observation.type = OBSERVATION_TYPE_GENERATION,
        gen_ai.usage.input_tokens = tracing::field::Empty,
        gen_ai.usage.output_tokens = tracing::field::Empty,
        langfuse.observation.input = tracing::field::Empty,
        langfuse.observation.output = tracing::field::Empty,
        gen_ai.prompt = tracing::field::Empty,
        gen_ai.completion = tracing::field::Empty,
    );
    fut.instrument(span).await
}

/// Generation span that records usage + I/O on `Ok` (SSOT for extract/keywords/glean/…).
///
/// Call site returns `(value, LlmGenerationRecord)`; attrs are applied only on success.
pub async fn with_llm_generation<Fut, T, E>(
    operation: &str,
    model: &str,
    provider: &str,
    fut: Fut,
) -> Result<T, E>
where
    Fut: Future<Output = Result<(T, LlmGenerationRecord), E>>,
{
    with_rag_generation_span(operation, model, provider, async {
        match fut.await {
            Ok((value, rec)) => {
                rec.record_on_current_span();
                Ok(value)
            }
            Err(e) => Err(e),
        }
    })
    .await
}

/// Run `fut` inside an embedding observation span (Langfuse type `embedding`).
pub async fn with_rag_embedding_span<Fut, T>(
    operation: &str,
    model: &str,
    provider: &str,
    fut: Fut,
) -> T
where
    Fut: Future<Output = T>,
{
    let span = tracing::info_span!(
        "rag.embedding",
        otel.name = %operation,
        otel.kind = "client",
        gen_ai.operation.name = "embeddings",
        gen_ai.request.model = %model,
        gen_ai.provider.name = %provider,
        langfuse.observation.type = OBSERVATION_TYPE_EMBEDDING,
        gen_ai.usage.input_tokens = tracing::field::Empty,
        gen_ai.usage.output_tokens = tracing::field::Empty,
        langfuse.observation.input = tracing::field::Empty,
        langfuse.observation.output = tracing::field::Empty,
        gen_ai.prompt = tracing::field::Empty,
        gen_ai.completion = tracing::field::Empty,
    );
    fut.instrument(span).await
}

/// Compact embedding I/O on the current embedding span.
pub fn record_embedding_io(kind: &str, text_count: usize, vector_count: usize, dim: Option<usize>) {
    let input = format!("{{\"kind\":\"{kind}\",\"texts\":{text_count}}}");
    let output = match dim {
        Some(d) => format!("{{\"vectors\":{vector_count},\"dim\":{d}}}"),
        None => format!("{{\"vectors\":{vector_count}}}"),
    };
    record_observation_io(Some(&input), Some(&output));
}

/// Run `fut` inside a feature root span (e.g. ingest.document as Langfuse `chain`).
pub async fn with_feature_root_span<Fut, T>(
    name: &str,
    feature_tag: &str,
    observation_type: &str,
    fut: Fut,
) -> T
where
    Fut: Future<Output = T>,
{
    let span = tracing::info_span!(
        "feature.root",
        otel.name = %name,
        otel.kind = "internal",
        langfuse.observation.type = %observation_type,
        langfuse.trace.tags = %feature_tag,
        langfuse.observation.input = tracing::field::Empty,
        langfuse.observation.output = tracing::field::Empty,
        gen_ai.prompt = tracing::field::Empty,
        gen_ai.completion = tracing::field::Empty,
    );
    fut.instrument(span).await
}

/// Convenience: ingest document root (`chain` + tag `ingest`).
pub async fn with_ingest_document_span<Fut, T>(fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    with_feature_root_span("ingest.document", "ingest", OBSERVATION_TYPE_CHAIN, fut).await
}

/// Record ingest.document input (doc id + content preview).
pub fn record_ingest_document_input(document_id: &str, content: &str) {
    let preview = query_preview(content, 256);
    let escaped = escape_json_string(&preview);
    let input = format!("{{\"document_id\":\"{document_id}\",\"preview\":\"{escaped}\"}}");
    record_observation_io(Some(&input), None);
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Record ingest.document output stats JSON (no cost fields).
pub fn record_ingest_document_output(
    chunk_count: usize,
    entity_count: usize,
    relationship_count: usize,
    successful_chunks: usize,
    failed_chunks: usize,
) {
    let output = format!(
        "{{\"chunks\":{chunk_count},\"entities\":{entity_count},\"relationships\":{relationship_count},\"successful_chunks\":{successful_chunks},\"failed_chunks\":{failed_chunks}}}"
    );
    record_observation_io(None, Some(&output));
}

/// Record pipeline_chunk_extraction I/O on the current span.
pub fn record_pipeline_chunk_extraction_io(
    chunk_count: usize,
    successful: Option<usize>,
    failed: Option<usize>,
    entity_count: Option<usize>,
) {
    let input = format!("{{\"chunk_count\":{chunk_count}}}");
    match (successful, failed, entity_count) {
        (Some(ok), Some(fail), Some(ents)) => {
            let output = format!("{{\"successful\":{ok},\"failed\":{fail},\"entities\":{ents}}}");
            record_observation_io(Some(&input), Some(&output));
        }
        _ => {
            record_observation_io(Some(&input), None);
        }
    }
}

/// Ensure current span is tagged as a Langfuse `span` observation (workflow step).
pub fn record_observation_type_span() {
    let span = tracing::Span::current();
    span.record("langfuse.observation.type", OBSERVATION_TYPE_SPAN);
    record_otel_str_attr(
        crate::langfuse_attrs::LANGFUSE_OBSERVATION_TYPE,
        OBSERVATION_TYPE_SPAN,
    );
}

/// Record GenAI token usage on the current span (LAW-124-12: tokens only, never cost).
pub fn record_gen_ai_usage(input_tokens: Option<u64>, output_tokens: Option<u64>) {
    let span = tracing::Span::current();
    if let Some(n) = input_tokens {
        span.record(GEN_AI_USAGE_INPUT_TOKENS, n as i64);
        record_otel_int_attr(GEN_AI_USAGE_INPUT_TOKENS, n);
    }
    if let Some(n) = output_tokens {
        span.record(GEN_AI_USAGE_OUTPUT_TOKENS, n as i64);
        record_otel_int_attr(GEN_AI_USAGE_OUTPUT_TOKENS, n);
    }
}

/// Record truncated observation I/O (LAW-124-8 / 16 / 17).
///
/// Dual-writes Langfuse keys + `gen_ai.prompt` / `gen_ai.completion` aliases.
pub fn record_observation_io(input: Option<&str>, output: Option<&str>) {
    let span = tracing::Span::current();
    if let Some(inp) = input {
        let preview = query_preview(inp, OBSERVATION_IO_PREVIEW_CHARS);
        span.record(LANGFUSE_OBSERVATION_INPUT, preview.as_str());
        span.record(GEN_AI_PROMPT, preview.as_str());
        record_otel_str_attr(LANGFUSE_OBSERVATION_INPUT, &preview);
        record_otel_str_attr(GEN_AI_PROMPT, &preview);
    }
    if let Some(out) = output {
        let preview = query_preview(out, OBSERVATION_IO_PREVIEW_CHARS);
        span.record(LANGFUSE_OBSERVATION_OUTPUT, preview.as_str());
        span.record(GEN_AI_COMPLETION, preview.as_str());
        record_otel_str_attr(LANGFUSE_OBSERVATION_OUTPUT, &preview);
        record_otel_str_attr(GEN_AI_COMPLETION, &preview);
    }
}

/// Set Langfuse feature tag on the current span (`query` | `ingest`).
pub fn record_feature_tag(tag: &str) {
    let span = tracing::Span::current();
    span.record(LANGFUSE_TRACE_TAGS, tag);
    record_otel_str_attr(LANGFUSE_TRACE_TAGS, tag);
}

/// Bind Langfuse identity + tag current span as feature `query` (API handler SSOT).
pub fn stamp_query_langfuse(
    session_id: Option<&str>,
    user_id: Option<&str>,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
) -> crate::langfuse_context::LangfuseIdentityGuard {
    stamp_query_langfuse_identity(crate::langfuse_attrs::LangfuseTraceIdentity::from_parts(
        session_id,
        user_id,
        tenant_id,
        workspace_id,
    ))
}

/// Bind full query identity (GUIDs + slugs) and tag `query`.
pub fn stamp_query_langfuse_identity(
    identity: crate::langfuse_attrs::LangfuseTraceIdentity,
) -> crate::langfuse_context::LangfuseIdentityGuard {
    let guard = crate::langfuse_context::bind_langfuse_trace_identity(identity);
    record_feature_tag("query");
    guard
}

/// Bind ingest identity (session = document_id) and tag `ingest`.
pub fn stamp_ingest_langfuse(
    identity: crate::langfuse_attrs::LangfuseTraceIdentity,
) -> crate::langfuse_context::LangfuseIdentityGuard {
    let guard = crate::langfuse_context::bind_langfuse_trace_identity(identity);
    record_feature_tag("ingest");
    guard
}

/// Sync stage span (fusion / cache hits) — enter/drop around a block.
pub fn enter_pipeline_stage(name: &'static str) -> tracing::span::EnteredSpan {
    tracing::info_span!(
        "pipeline.stage",
        otel.name = %name,
        otel.kind = "internal",
        langfuse.observation.type = OBSERVATION_TYPE_SPAN,
    )
    .entered()
}

/// Cheap workflow observation (`span`) for query/ingest stages (LAW-124-21/22).
pub async fn with_pipeline_stage_span<Fut, T>(name: &str, fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    let span = tracing::info_span!(
        "pipeline.stage",
        otel.name = %name,
        otel.kind = "internal",
        langfuse.observation.type = OBSERVATION_TYPE_SPAN,
        langfuse.observation.input = tracing::field::Empty,
        langfuse.observation.output = tracing::field::Empty,
        gen_ai.prompt = tracing::field::Empty,
        gen_ai.completion = tracing::field::Empty,
    );
    fut.instrument(span).await
}

/// Ingest worker root (`chain` + tag `ingest`).
pub async fn with_ingest_task_span<Fut, T>(fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    with_feature_root_span("ingest.task", "ingest", OBSERVATION_TYPE_CHAIN, fut).await
}

/// Root observation I/O for query/chat HTTP spans (LAW-124-16).
pub fn record_query_root_io(query: &str, answer: &str) {
    record_observation_io(Some(query), Some(answer));
}

/// Pure helper: truncate query text for span attributes (PII-safe preview).
pub fn query_preview(query: &str, max_chars: usize) -> String {
    if query.chars().count() <= max_chars {
        return query.to_string();
    }
    format!("{}…", crate::utf8_prefix(query, max_chars))
}

#[cfg(feature = "otel")]
fn record_otel_int_attr(key: &str, value: u64) {
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry::{Context, KeyValue};
    let cx = Context::current();
    if cx.has_active_span() {
        cx.span()
            .set_attribute(KeyValue::new(key.to_string(), value as i64));
    }
}

#[cfg(not(feature = "otel"))]
fn record_otel_int_attr(_key: &str, _value: u64) {}

#[cfg(feature = "otel")]
fn record_otel_str_attr(key: &str, value: &str) {
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry::{Context, KeyValue};
    let cx = Context::current();
    if cx.has_active_span() {
        cx.span()
            .set_attribute(KeyValue::new(key.to_string(), value.to_string()));
    }
}

#[cfg(not(feature = "otel"))]
fn record_otel_str_attr(_key: &str, _value: &str) {}

pub use crate::langfuse_attrs::{
    OBSERVATION_TYPE_CHAIN as OBS_TYPE_CHAIN, OBSERVATION_TYPE_EMBEDDING as OBS_TYPE_EMBEDDING,
    OBSERVATION_TYPE_GENERATION as OBS_TYPE_GENERATION,
    OBSERVATION_TYPE_RETRIEVER as OBS_TYPE_RETRIEVER, OBSERVATION_TYPE_SPAN as OBS_TYPE_SPAN,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langfuse_attrs::{is_forbidden_cost_attr, COST_ATTR_DENYLIST};

    #[test]
    fn query_preview_short_unchanged() {
        assert_eq!(query_preview("hello", 100), "hello");
    }

    #[test]
    fn query_preview_truncates_at_boundary() {
        let s = "a".repeat(50);
        let p = query_preview(&s, 10);
        assert!(p.ends_with('…'));
        assert!(p.chars().count() <= 11);
    }

    #[test]
    fn helpers_source_has_no_cost_attr_literals() {
        let src = include_str!("rag_span.rs");
        for key in COST_ATTR_DENYLIST {
            assert!(
                !src.contains(key),
                "rag_span.rs must not contain forbidden cost attr {key}"
            );
        }
    }

    #[test]
    fn retrieval_source_sets_langfuse_observation_input() {
        let src = include_str!("rag_span.rs");
        assert!(
            src.contains("langfuse.observation.input"),
            "retrieval helper must set langfuse.observation.input for Langfuse UI"
        );
        assert!(src.contains("record_rag_retrieval_io"));
    }

    #[test]
    fn cost_denylist_does_not_block_usage_keys() {
        assert!(!is_forbidden_cost_attr(GEN_AI_USAGE_INPUT_TOKENS));
        assert!(!is_forbidden_cost_attr(GEN_AI_USAGE_OUTPUT_TOKENS));
    }

    #[tokio::test]
    async fn with_rag_retrieval_span_runs_future() {
        let v = with_rag_retrieval_span(
            RagRetrievalAttrs {
                data_source_id: Some("test"),
                top_k: Some(5),
                arm: Some("naive"),
                mode: Some("mix"),
                query_preview: Some("q".into()),
            },
            async {
                record_rag_retrieval_io(false, 2, 1, Some("hit"));
                42
            },
        )
        .await;
        assert_eq!(v, 42);
    }

    #[tokio::test]
    async fn with_rag_generation_span_runs_future() {
        let v = with_rag_generation_span("generate-answer", "mock", "mock", async {
            record_gen_ai_usage(Some(10), Some(20));
            record_observation_io(Some("in"), Some("out"));
            7
        })
        .await;
        assert_eq!(v, 7);
    }

    #[tokio::test]
    async fn with_llm_generation_records_on_ok() {
        let v: Result<i32, &str> = with_llm_generation("extract-keywords", "m", "p", async {
            Ok((9, LlmGenerationRecord::from_response(Some("q"), "kw", 3, 4)))
        })
        .await;
        assert_eq!(v, Ok(9));
    }

    #[tokio::test]
    async fn with_llm_generation_skips_record_on_err() {
        let v: Result<i32, &str> =
            with_llm_generation("extract-keywords", "m", "p", async { Err("boom") }).await;
        assert_eq!(v, Err("boom"));
    }

    #[test]
    fn stamp_query_langfuse_is_callable() {
        let _g = stamp_query_langfuse(Some("s1"), Some("u1"), None, None);
        record_query_root_io("q", "a");
    }

    #[tokio::test]
    async fn with_rag_embedding_and_ingest_root_run() {
        let a = with_rag_embedding_span("embed-chunks", "m", "p", async {
            record_embedding_io("chunks", 3, 3, Some(8));
            1
        })
        .await;
        let b = with_ingest_document_span(async {
            record_ingest_document_input("doc-1", "hello world");
            record_ingest_document_output(1, 2, 0, 1, 0);
            2
        })
        .await;
        assert_eq!((a, b), (1, 2));
    }
}
