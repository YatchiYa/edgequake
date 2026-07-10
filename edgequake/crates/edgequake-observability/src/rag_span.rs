//! RAG / GenAI tracing spans (SPEC-046 OPS-P2 / OTel GenAI conventions).
//!
//! Always emits `tracing` spans (works without OTLP). Attribute names follow
//! OpenTelemetry GenAI semantic conventions (development status):
//! - `gen_ai.operation.name` = `retrieval` | `chat`
//! - `gen_ai.data_source.id`
//! - `gen_ai.retrieval.top_k`
//! - EdgeQuake extensions: `rag.retrieval.arm`, `rag.retrieval.empty_result`,
//!   `rag.context.truncated`, `rag.retrieval.fallback`
//!
//! SOLID: observability owns attribute mapping; query/API only call helpers.

use std::future::Future;

use tracing::Instrument;

/// Attributes for a retrieval-phase span.
#[derive(Debug, Clone, Default)]
pub struct RagRetrievalAttrs {
    pub data_source_id: Option<&'static str>,
    pub top_k: Option<usize>,
    pub arm: Option<&'static str>,
    pub mode: Option<&'static str>,
    pub query_preview: Option<String>,
}

/// Run `fut` inside a GenAI retrieval span (always; OTLP optional via feature).
pub async fn with_rag_retrieval_span<Fut, T>(attrs: RagRetrievalAttrs, fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    let data_source = attrs.data_source_id.unwrap_or("edgequake");
    let span_name = format!("retrieval {data_source}");
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
        gen_ai.retrieval.query.text = attrs
            .query_preview
            .as_deref()
            .unwrap_or(""),
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

/// Run `fut` inside a GenAI chat/generation span.
pub async fn with_rag_generation_span<Fut, T>(model: &str, provider: &str, fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    let span = tracing::info_span!(
        "rag.generation",
        otel.name = "chat",
        otel.kind = "client",
        gen_ai.operation.name = "chat",
        gen_ai.request.model = %model,
        gen_ai.provider.name = %provider,
    );
    fut.instrument(span).await
}

/// Pure helper: truncate query text for span attributes (PII-safe preview).
pub fn query_preview(query: &str, max_chars: usize) -> String {
    if query.len() <= max_chars {
        return query.to_string();
    }
    let mut end = max_chars;
    while end > 0 && !query.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &query[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_preview_short_unchanged() {
        assert_eq!(query_preview("hello", 100), "hello");
    }

    #[test]
    fn query_preview_truncates_at_boundary() {
        let s = "a".repeat(50);
        let p = query_preview(&s, 10);
        assert!(p.ends_with('…'));
        assert!(p.len() <= 11 + "…".len()); // 10 chars + ellipsis (may be multi-byte)
        assert!(p.chars().count() <= 11);
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
            async { 42 },
        )
        .await;
        assert_eq!(v, 42);
    }
}
