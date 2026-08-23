//! Filterable Langfuse metadata SSOT (SPEC-124 I8 / LAW-124-20..22).
//!
//! Only `langfuse.trace.metadata.*` and `langfuse.observation.metadata.*` are
//! the Langfuse UI filter contract. Values are truncated to 200 chars.

use crate::langfuse_attrs::{
    LANGFUSE_METADATA_VALUE_MAX_CHARS, LANGFUSE_OBSERVATION_METADATA_PREFIX,
    LANGFUSE_TRACE_METADATA_PREFIX,
};
use crate::utf8_truncate::utf8_prefix;

fn sanitize_meta_key(key: &str) -> Option<String> {
    let k = key.trim().to_ascii_lowercase();
    if k.is_empty() {
        return None;
    }
    if !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some(k)
}

fn clip_meta_value(value: &str) -> String {
    let t = value.trim();
    if t.chars().count() <= LANGFUSE_METADATA_VALUE_MAX_CHARS {
        return t.to_string();
    }
    format!("{}…", utf8_prefix(t, LANGFUSE_METADATA_VALUE_MAX_CHARS))
}

#[cfg(feature = "otel")]
fn set_otel_str(key: &str, value: &str) {
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry::{Context, KeyValue};
    let cx = Context::current();
    if cx.has_active_span() {
        cx.span()
            .set_attribute(KeyValue::new(key.to_string(), value.to_string()));
    }
}

#[cfg(not(feature = "otel"))]
fn set_otel_str(_key: &str, _value: &str) {}

/// Record one filterable trace-metadata pair on the current span.
pub fn record_trace_meta(key: &str, value: &str) {
    let Some(k) = sanitize_meta_key(key) else {
        return;
    };
    let v = clip_meta_value(value);
    if v.is_empty() {
        return;
    }
    let full = format!("{LANGFUSE_TRACE_METADATA_PREFIX}{k}");
    set_otel_str(&full, &v);
}

/// Record one filterable observation-metadata pair on the current span.
pub fn record_observation_meta(key: &str, value: &str) {
    let Some(k) = sanitize_meta_key(key) else {
        return;
    };
    let v = clip_meta_value(value);
    if v.is_empty() {
        return;
    }
    let full = format!("{LANGFUSE_OBSERVATION_METADATA_PREFIX}{k}");
    set_otel_str(&full, &v);
}

fn opt_bool(v: Option<bool>) -> Option<String> {
    v.map(|b| if b { "true" } else { "false" }.to_string())
}

/// Query pipeline dimensions (LAW-124-21). Call sites fill; observability writes keys.
#[derive(Debug, Clone, Default)]
pub struct QueryPipelineMeta {
    pub mode: Option<String>,
    pub query_intent: Option<String>,
    pub fusion: Option<String>,
    pub arms_run: Option<String>,
    pub keyword_cache_hit: Option<bool>,
    pub answer_cache_hit: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub sparse_outcome: Option<String>,
    pub citation_count: Option<usize>,
    pub context_empty: Option<bool>,
    pub context_truncated: Option<bool>,
}

/// LAW-124-21 SSOT entry (call sites must not invent metadata keys).
pub fn record_query_pipeline_meta(meta: QueryPipelineMeta) {
    meta.record();
}

impl QueryPipelineMeta {
    pub fn record(self) {
        if let Some(v) = self.mode.as_deref() {
            record_trace_meta("mode", v);
        }
        if let Some(v) = self.query_intent.as_deref() {
            record_trace_meta("query_intent", v);
        }
        if let Some(v) = self.fusion.as_deref() {
            record_trace_meta("fusion", v);
        }
        if let Some(v) = self.arms_run.as_deref() {
            record_trace_meta("arms_run", v);
        }
        if let Some(v) = opt_bool(self.keyword_cache_hit) {
            record_trace_meta("keyword_cache_hit", &v);
        }
        if let Some(v) = opt_bool(self.answer_cache_hit) {
            record_trace_meta("answer_cache_hit", &v);
        }
        if let Some(v) = self.reasoning_effort.as_deref() {
            record_trace_meta("reasoning_effort", v);
        }
        if let Some(v) = self.sparse_outcome.as_deref() {
            record_trace_meta("sparse_outcome", v);
        }
        if let Some(n) = self.citation_count {
            record_trace_meta("citation_count", &n.to_string());
        }
        if let Some(v) = opt_bool(self.context_empty) {
            record_trace_meta("context_empty", &v);
        }
        if let Some(v) = opt_bool(self.context_truncated) {
            record_trace_meta("context_truncated", &v);
        }
    }
}

/// PDF/vision parse facts (LAW-124-22).
#[derive(Debug, Clone, Default)]
pub struct IngestParseMeta {
    pub parser: Option<String>,
    pub vision_provider: Option<String>,
    pub vision_model: Option<String>,
    pub page_count: Option<usize>,
    pub pass: Option<String>,
    pub fallback: Option<bool>,
}

/// LAW-124-22 parse/vision facts.
pub fn record_ingest_parse_meta(meta: IngestParseMeta) {
    meta.record();
}

impl IngestParseMeta {
    pub fn record(self) {
        if let Some(v) = self.parser.as_deref() {
            record_observation_meta("parser", v);
            record_trace_meta("parser", v);
        }
        if let Some(v) = self.vision_provider.as_deref() {
            record_observation_meta("vision_provider", v);
            record_trace_meta("vision_provider", v);
        }
        if let Some(v) = self.vision_model.as_deref() {
            record_observation_meta("vision_model", v);
            record_trace_meta("vision_model", v);
        }
        if let Some(n) = self.page_count {
            let s = n.to_string();
            record_observation_meta("page_count", &s);
            record_trace_meta("page_count", &s);
        }
        if let Some(v) = self.pass.as_deref() {
            record_observation_meta("pass", v);
        }
        if let Some(v) = opt_bool(self.fallback) {
            record_observation_meta("fallback", &v);
            record_trace_meta("parse_fallback", &v);
        }
    }
}

/// KG-slice facts after chunk/extract/embed.
#[derive(Debug, Clone, Default)]
pub struct IngestKgMeta {
    pub chunk_strategy: Option<String>,
    pub chunk_size: Option<usize>,
    pub overlap: Option<usize>,
    pub gleaning_max: Option<usize>,
    pub embed_model: Option<String>,
    pub embed_dim: Option<usize>,
    pub extract_entity_cap: Option<usize>,
    /// SPEC-125: actual emitted token min (tiktoken / chunk.token_count).
    pub token_min: Option<usize>,
    /// SPEC-125: median emitted token count.
    pub token_p50: Option<usize>,
    /// SPEC-125: actual emitted token max.
    pub token_max: Option<usize>,
    /// SPEC-125: chunks whose body is ATX headings only.
    pub orphan_heading_chunks: Option<usize>,
    /// SPEC-135: token_p50 / chunk budget.
    pub fill_p50: Option<f64>,
    /// SPEC-135: whether `<!-- multimodal-chunks -->` sidecars were concatenated.
    pub mm_sidecar_appended: Option<bool>,
}

/// LAW-124-22 KG-slice facts.
pub fn record_ingest_kg_meta(meta: IngestKgMeta) {
    meta.record();
}

impl IngestKgMeta {
    pub fn record(self) {
        if let Some(v) = self.chunk_strategy.as_deref() {
            record_trace_meta("chunk_strategy", v);
            record_observation_meta("chunk_strategy", v);
        }
        if let Some(n) = self.chunk_size {
            record_observation_meta("chunk_size", &n.to_string());
        }
        if let Some(n) = self.overlap {
            record_observation_meta("overlap", &n.to_string());
        }
        if let Some(n) = self.gleaning_max {
            record_trace_meta("gleaning_max", &n.to_string());
        }
        if let Some(v) = self.embed_model.as_deref() {
            record_trace_meta("embed_model", v);
        }
        if let Some(n) = self.embed_dim {
            record_trace_meta("embed_dim", &n.to_string());
        }
        if let Some(n) = self.extract_entity_cap {
            record_trace_meta("extract_entity_cap", &n.to_string());
        }
        if let Some(n) = self.token_min {
            record_observation_meta("token_min", &n.to_string());
        }
        if let Some(n) = self.token_p50 {
            record_observation_meta("token_p50", &n.to_string());
        }
        if let Some(n) = self.token_max {
            record_observation_meta("token_max", &n.to_string());
        }
        if let Some(n) = self.orphan_heading_chunks {
            record_observation_meta("orphan_heading_chunks", &n.to_string());
        }
        if let Some(f) = self.fill_p50 {
            record_observation_meta("fill_p50", &format!("{f:.4}"));
        }
        if let Some(b) = self.mm_sidecar_appended {
            record_observation_meta("mm_sidecar_appended", if b { "true" } else { "false" });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_alnum_keys() {
        record_trace_meta("mode!", "mix");
        record_trace_meta("", "x");
        record_trace_meta("mode", "  mix  ");
    }
}
