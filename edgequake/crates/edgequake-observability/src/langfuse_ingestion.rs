//! Langfuse **native ingestion** span exporter (`POST /api/public/ingestion`).
//!
//! # Why this exists
//!
//! LAW-124-1: the supported path is OTLP/HTTP to `/api/public/otel/v1/traces`.
//! That endpoint arrived in Langfuse **3.22**. Self-hosted **3.1.1** returns 404.
//!
//! The legacy ingestion API (`/api/public/ingestion`) is present on 3.1.1. It is
//! a **compatibility bridge**, not a replacement: Cloud sunsets trace events on
//! this endpoint on 2026-11-16; self-hosted v4 `events_only` rejects them.
//!
//! # 3.1.1 envelope contract (unfakable)
//!
//! [ingestion.yml @ v3.1.1](https://raw.githubusercontent.com/langfuse/langfuse/v3.1.1/fern/apis/server/definition/ingestion.yml)
//! discriminant values: `trace-create`, `span-create`, `generation-create`,
//! `event-create` (+ score/sdk-log, legacy `observation-*`).
//! `ObservationType` is only `SPAN | GENERATION | EVENT`.
//!
//! LAW-124-13 tags `retriever` / `embedding` / `chain`. Those **must not** be
//! stringified into `{type}-create` (that emits `retriever-create`, which 3.1.1
//! rejects inside a 207). Mapping SSOT: [`langfuse_v31_envelope_type`].

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry::Value;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::trace::{SpanData, SpanExporter};
use serde_json::{json, Map as JsonMap, Value as JsonValue};

use crate::langfuse_attrs::{
    GEN_AI_COMPLETION, GEN_AI_PROMPT, GEN_AI_USAGE_INPUT_TOKENS, GEN_AI_USAGE_OUTPUT_TOKENS,
    LANGFUSE_OBSERVATION_INPUT, LANGFUSE_OBSERVATION_OUTPUT, LANGFUSE_OBSERVATION_TYPE,
    LANGFUSE_SESSION_ID, LANGFUSE_USER_ID, OBSERVATION_TYPE_GENERATION, SESSION_ID, USER_ID,
};

/// 3.1.1 fern discriminant: generation observation.
pub const LANGFUSE_V31_GENERATION_CREATE: &str = "generation-create";
/// 3.1.1 fern discriminant: span observation (retriever/embedding/chain land here).
pub const LANGFUSE_V31_SPAN_CREATE: &str = "span-create";
/// 3.1.1 fern discriminant: trace upsert.
pub const LANGFUSE_V31_TRACE_CREATE: &str = "trace-create";

/// Allowed envelope `type` values on Langfuse 3.1.1 (subset we emit).
pub const LANGFUSE_V31_EMITTED_ENVELOPE_TYPES: &[&str] = &[
    LANGFUSE_V31_TRACE_CREATE,
    LANGFUSE_V31_SPAN_CREATE,
    LANGFUSE_V31_GENERATION_CREATE,
];

/// Map LAW-124-13 observation types onto the Langfuse **3.1.1** envelope type.
///
/// Explicit `generation` → `generation-create`. Any other explicit type
/// (`retriever`, `embedding`, `chain`, `span`) → `span-create` so 3.1.1 accepts
/// the event. Untagged spans with model or token usage are treated as
/// generations. An explicit non-generation type always wins over model/usage.
#[must_use]
pub fn langfuse_v31_envelope_type(
    observation_type: Option<&str>,
    has_model: bool,
    has_usage: bool,
) -> &'static str {
    match observation_type.map(str::trim) {
        Some(t) if t.eq_ignore_ascii_case(OBSERVATION_TYPE_GENERATION) => {
            LANGFUSE_V31_GENERATION_CREATE
        }
        Some(_) => LANGFUSE_V31_SPAN_CREATE,
        None if has_model || has_usage => LANGFUSE_V31_GENERATION_CREATE,
        None => LANGFUSE_V31_SPAN_CREATE,
    }
}

/// Interpret a 3.1.1 ingestion HTTP response.
///
/// The API returns **207** with `errors[]` for per-event schema failures instead
/// of 4xx. Treating 207 as success without reading `errors[]` hides dropped
/// retriever/ingest observations.
pub fn ingestion_http_outcome(status: u16, body: &str) -> Result<(), String> {
    let ok_status = (200..300).contains(&status);
    if !ok_status {
        return Err(format!(
            "langfuse ingestion HTTP {status}: {}",
            truncate_chars(body, 300)
        ));
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let parsed: JsonValue = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return Ok(()), // non-JSON 2xx (should not happen)
    };
    let errors = parsed
        .get("errors")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();
    if errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "langfuse ingestion HTTP {status} with {} event error(s): {}",
        errors.len(),
        truncate_chars(&errors[0].to_string(), 300)
    ))
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Span exporter that speaks Langfuse's native ingestion API.
#[derive(Debug)]
pub struct LangfuseIngestionExporter {
    endpoint: String,
    auth_header: String,
    /// Built lazily on first export. Construction does not need a runtime;
    /// `send().await` runs on the batch processor's Tokio worker (LAW-124-4).
    client: OnceLock<reqwest::Client>,
}

impl LangfuseIngestionExporter {
    /// Build an exporter for `base_url` (no trailing path) with Basic auth.
    #[must_use]
    pub fn new(base_url: &str, public_key: &str, secret_key: &str) -> Self {
        let token = crate::langfuse::basic_auth_token(public_key, secret_key);
        Self {
            endpoint: format!("{}/api/public/ingestion", base_url.trim_end_matches('/')),
            auth_header: format!("Basic {token}"),
            client: OnceLock::new(),
        }
    }

    /// Endpoint this exporter posts to (diagnostics).
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// RFC3339 (UTC, millisecond precision) — the format Langfuse 3.1.1 expects.
fn rfc3339(ts: SystemTime) -> String {
    let d = ts.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs() as i64;
    let millis = d.subsec_millis();
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{mi:02}:{s:02}.{millis:03}Z")
}

fn trace_hex(id: TraceId) -> String {
    format!("{:032x}", u128::from_be_bytes(id.to_bytes()))
}

fn span_hex(id: SpanId) -> String {
    format!("{:016x}", u64::from_be_bytes(id.to_bytes()))
}

/// Deterministic UUID v5 so retries dedupe (3.1 envelope id is a UUID).
fn envelope_uuid(kind: &str, key: &str) -> String {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("https://edgequake.dev/langfuse/{kind}/{key}").as_bytes(),
    )
    .to_string()
}

fn value_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Bool(b) => json!(b),
        Value::I64(i) => json!(i),
        Value::F64(f) => json!(f),
        Value::String(s) => json!(s.as_str()),
        other => json!(other.to_string()),
    }
}

fn value_as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::I64(i) => Some(*i),
        Value::F64(f) if f.is_finite() => Some(*f as i64),
        Value::String(s) => s.as_str().parse().ok(),
        _ => None,
    }
}

struct Extracted {
    attributes: JsonMap<String, JsonValue>,
    observation_type: Option<String>,
    model: Option<String>,
    input: Option<JsonValue>,
    output: Option<JsonValue>,
    usage_in: Option<i64>,
    usage_out: Option<i64>,
    session_id: Option<String>,
    user_id: Option<String>,
    level: Option<String>,
}

fn extract(span: &SpanData) -> Extracted {
    let mut e = Extracted {
        attributes: JsonMap::new(),
        observation_type: None,
        model: None,
        input: None,
        output: None,
        usage_in: None,
        usage_out: None,
        session_id: None,
        user_id: None,
        level: None,
    };
    for kv in span.attributes.iter() {
        let k = kv.key.as_str();
        let v = &kv.value;
        match k {
            LANGFUSE_OBSERVATION_TYPE => e.observation_type = Some(v.to_string()),
            "gen_ai.request.model"
            | "gen_ai.response.model"
            | "langfuse.observation.model.name" => {
                e.model = Some(v.to_string());
            }
            LANGFUSE_OBSERVATION_INPUT | GEN_AI_PROMPT | "input.value" => {
                e.input = Some(value_to_json(v));
            }
            LANGFUSE_OBSERVATION_OUTPUT | GEN_AI_COMPLETION | "output.value" => {
                e.output = Some(value_to_json(v));
            }
            GEN_AI_USAGE_INPUT_TOKENS | "gen_ai.usage.prompt_tokens" => {
                e.usage_in = value_as_i64(v);
            }
            GEN_AI_USAGE_OUTPUT_TOKENS | "gen_ai.usage.completion_tokens" => {
                e.usage_out = value_as_i64(v);
            }
            LANGFUSE_SESSION_ID | SESSION_ID => e.session_id = Some(v.to_string()),
            LANGFUSE_USER_ID | USER_ID => e.user_id = Some(v.to_string()),
            "langfuse.observation.level" => e.level = Some(v.to_string()),
            _ => {
                e.attributes.insert(k.to_string(), value_to_json(v));
            }
        }
    }
    e
}

/// Span names that describe a request better than the Axum `HTTP` transport root.
const PREFERRED_TRACE_NAMES: &[&str] = &[
    "query_pipeline",
    "query_execute",
    "chat_stream",
    "query_stream",
    "ingest.document",
    "task_process",
];

/// Map a batch of OTel spans onto Langfuse 3.1.1 ingestion events.
///
/// One `trace-create` per trace id. Every span becomes `generation-create` or
/// `span-create` via [`langfuse_v31_envelope_type`] — never `{retriever,embedding,chain}-create`.
#[must_use]
pub fn spans_to_batch(batch: &[SpanData]) -> Vec<JsonValue> {
    let mut events = Vec::with_capacity(batch.len() * 2);

    struct TraceInfo {
        name: String,
        name_is_preferred: bool,
        session_id: Option<String>,
        user_id: Option<String>,
        timestamp: String,
        input: Option<JsonValue>,
        output: Option<JsonValue>,
    }
    let mut traces: HashMap<String, TraceInfo> = HashMap::new();

    for span in batch {
        let trace_id = trace_hex(span.span_context.trace_id());
        let is_root = span.parent_span_id == SpanId::INVALID;
        let e = extract(span);
        let name = span.name.to_string();
        let preferred = PREFERRED_TRACE_NAMES.contains(&name.as_str());
        let start = rfc3339(span.start_time);

        let info = traces.entry(trace_id.clone()).or_insert_with(|| TraceInfo {
            name: name.clone(),
            name_is_preferred: preferred,
            session_id: None,
            user_id: None,
            timestamp: start.clone(),
            input: None,
            output: None,
        });
        if preferred && !info.name_is_preferred {
            info.name = name.clone();
            info.name_is_preferred = true;
        } else if is_root && !info.name_is_preferred {
            info.name = name.clone();
            info.timestamp = start.clone();
        }
        if info.session_id.is_none() {
            info.session_id = e.session_id.clone();
        }
        if info.user_id.is_none() {
            info.user_id = e.user_id.clone();
        }
        if is_root {
            if info.input.is_none() {
                info.input = e.input.clone();
            }
            if info.output.is_none() {
                info.output = e.output.clone();
            }
        }
    }

    for (trace_id, info) in &traces {
        let mut body = json!({
            "id": trace_id,
            "name": info.name,
            "timestamp": info.timestamp,
        });
        if let Some(s) = &info.session_id {
            body["sessionId"] = json!(s);
        }
        if let Some(u) = &info.user_id {
            body["userId"] = json!(u);
        }
        if let Some(i) = &info.input {
            body["input"] = i.clone();
        }
        if let Some(o) = &info.output {
            body["output"] = o.clone();
        }
        events.push(json!({
            "id": envelope_uuid("trace", trace_id),
            "timestamp": info.timestamp,
            "type": LANGFUSE_V31_TRACE_CREATE,
            "body": body,
        }));
    }

    for span in batch {
        let trace_id = trace_hex(span.span_context.trace_id());
        let span_id = span_hex(span.span_context.span_id());
        let is_root = span.parent_span_id == SpanId::INVALID;
        let e = extract(span);
        let start = rfc3339(span.start_time);
        let end = rfc3339(span.end_time);
        let has_usage = e.usage_in.is_some() || e.usage_out.is_some();
        let envelope =
            langfuse_v31_envelope_type(e.observation_type.as_deref(), e.model.is_some(), has_usage);

        let mut body = json!({
            "id": span_id,
            "traceId": trace_id,
            "name": span.name.to_string(),
            "startTime": start,
            "endTime": end,
        });
        if !is_root {
            body["parentObservationId"] = json!(span_hex(span.parent_span_id));
        }
        if let Some(m) = &e.model {
            body["model"] = json!(m);
        }
        if let Some(i) = &e.input {
            body["input"] = i.clone();
        }
        if let Some(o) = &e.output {
            body["output"] = o.clone();
        }
        if has_usage {
            let i = e.usage_in.unwrap_or(0);
            let o = e.usage_out.unwrap_or(0);
            body["usage"] = json!({ "input": i, "output": o, "total": i + o });
        }
        let level = match span.status {
            opentelemetry::trace::Status::Error { .. } => "ERROR",
            _ => e.level.as_deref().unwrap_or("DEFAULT"),
        };
        body["level"] = json!(level);
        if !e.attributes.is_empty() {
            body["metadata"] = JsonValue::Object(e.attributes);
        }

        events.push(json!({
            "id": envelope_uuid("obs", &span_id),
            "timestamp": end,
            "type": envelope,
            "body": body,
        }));
    }

    events
}

impl SpanExporter for LangfuseIngestionExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        if batch.is_empty() {
            return Ok(());
        }
        let events = spans_to_batch(&batch);
        if events.is_empty() {
            return Ok(());
        }

        let client = self.client.get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default()
        });

        let resp = client
            .post(&self.endpoint)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&json!({ "batch": events }))
            .send()
            .await
            .map_err(|e| OTelSdkError::InternalFailure(format!("langfuse ingestion send: {e}")))?;

        let code = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        ingestion_http_outcome(code, &body).map_err(OTelSdkError::InternalFailure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langfuse_attrs::{
        OBSERVATION_TYPE_CHAIN, OBSERVATION_TYPE_EMBEDDING, OBSERVATION_TYPE_RETRIEVER,
        OBSERVATION_TYPE_SPAN,
    };
    use opentelemetry::trace::{SpanContext, SpanKind, TraceFlags, TraceState};
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::trace::{SpanEvents, SpanLinks};

    /// Distinct span ids per name so a live Langfuse 3.1.1 batch is not collapsed.
    fn span_id_from_name(name: &str) -> SpanId {
        let mut h = 0xcbf29ce484222325u64;
        for b in name.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x100000001b3);
        }
        if h == 0 {
            h = 1;
        }
        SpanId::from_bytes(h.to_be_bytes())
    }

    fn fake_span(name: &str, attrs: Vec<KeyValue>, root: bool) -> SpanData {
        fake_span_on(name, attrs, root, TraceId::from_bytes([1u8; 16]))
    }

    fn fake_span_on(name: &str, attrs: Vec<KeyValue>, root: bool, trace: TraceId) -> SpanData {
        fake_span_on_at(
            name,
            attrs,
            root,
            trace,
            UNIX_EPOCH,
            UNIX_EPOCH + Duration::from_secs(1),
        )
    }

    fn fake_span_on_at(
        name: &str,
        attrs: Vec<KeyValue>,
        root: bool,
        trace: TraceId,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> SpanData {
        let span = span_id_from_name(name);
        let parent = if root {
            SpanId::INVALID
        } else {
            span_id_from_name("ingest.document")
        };
        SpanData {
            span_context: SpanContext::new(
                trace,
                span,
                TraceFlags::SAMPLED,
                false,
                TraceState::NONE,
            ),
            parent_span_id: parent,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: name.to_string().into(),
            start_time,
            end_time,
            attributes: attrs,
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: opentelemetry::trace::Status::Unset,
            instrumentation_scope: Default::default(),
        }
    }

    fn types_in(batch: &[JsonValue]) -> Vec<String> {
        batch
            .iter()
            .filter_map(|e| e.get("type").and_then(|t| t.as_str()).map(str::to_string))
            .collect()
    }

    #[test]
    fn v31_envelope_types_are_only_generation_or_span() {
        let cases = [
            (
                Some(OBSERVATION_TYPE_GENERATION),
                false,
                false,
                LANGFUSE_V31_GENERATION_CREATE,
            ),
            (
                Some(OBSERVATION_TYPE_RETRIEVER),
                false,
                false,
                LANGFUSE_V31_SPAN_CREATE,
            ),
            (
                Some(OBSERVATION_TYPE_EMBEDDING),
                true,
                false,
                LANGFUSE_V31_SPAN_CREATE,
            ),
            (
                Some(OBSERVATION_TYPE_CHAIN),
                false,
                false,
                LANGFUSE_V31_SPAN_CREATE,
            ),
            (
                Some(OBSERVATION_TYPE_SPAN),
                false,
                false,
                LANGFUSE_V31_SPAN_CREATE,
            ),
            (None, true, false, LANGFUSE_V31_GENERATION_CREATE),
            (None, false, true, LANGFUSE_V31_GENERATION_CREATE),
            (None, false, false, LANGFUSE_V31_SPAN_CREATE),
            (Some("HTTP"), false, false, LANGFUSE_V31_SPAN_CREATE),
        ];
        for (ty, model, usage, want) in cases {
            assert_eq!(
                langfuse_v31_envelope_type(ty, model, usage),
                want,
                "type={ty:?} model={model} usage={usage}"
            );
            assert!(
                LANGFUSE_V31_EMITTED_ENVELOPE_TYPES.contains(&want),
                "emitted type {want} not in 3.1.1 allowlist"
            );
        }
    }

    #[test]
    fn embedding_with_model_is_not_forced_to_generation() {
        assert_eq!(
            langfuse_v31_envelope_type(Some(OBSERVATION_TYPE_EMBEDDING), true, true),
            LANGFUSE_V31_SPAN_CREATE
        );
    }

    #[test]
    fn ingestion_207_without_errors_is_ok() {
        assert!(ingestion_http_outcome(207, r#"{"successes":[],"errors":[]}"#).is_ok());
        assert!(ingestion_http_outcome(200, "").is_ok());
    }

    #[test]
    fn ingestion_207_with_errors_is_err() {
        let err = ingestion_http_outcome(
            207,
            r#"{"successes":[],"errors":[{"id":"x","status":400,"message":"Invalid type"}]}"#,
        )
        .expect_err("must fail");
        assert!(err.contains("event error"), "{err}");
        assert!(err.contains("Invalid type") || err.contains("400"), "{err}");
    }

    #[test]
    fn ingestion_401_is_err() {
        assert!(ingestion_http_outcome(401, "nope").is_err());
    }

    #[test]
    fn rfc3339_formats_epoch() {
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn endpoint_joins_path_once() {
        let e = LangfuseIngestionExporter::new("http://lf:3000/", "pk", "sk");
        assert_eq!(e.endpoint(), "http://lf:3000/api/public/ingestion");
    }

    #[test]
    fn spans_to_batch_maps_law124_types_to_v31_envelopes() {
        let gen = fake_span(
            "generate-answer",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_GENERATION),
                KeyValue::new("gen_ai.request.model", "gpt-5-nano"),
                KeyValue::new(GEN_AI_USAGE_INPUT_TOKENS, 10i64),
                KeyValue::new(GEN_AI_USAGE_OUTPUT_TOKENS, 4i64),
            ],
            false,
        );
        let retriever = fake_span(
            "retrieval edgequake",
            vec![KeyValue::new(
                LANGFUSE_OBSERVATION_TYPE,
                OBSERVATION_TYPE_RETRIEVER,
            )],
            false,
        );
        let embedding = fake_span(
            "query.embed",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_EMBEDDING),
                KeyValue::new("gen_ai.request.model", "text-embedding-3-small"),
            ],
            false,
        );
        let chain = fake_span(
            "ingest.document",
            vec![KeyValue::new(
                LANGFUSE_OBSERVATION_TYPE,
                OBSERVATION_TYPE_CHAIN,
            )],
            true,
        );
        let http = fake_span("HTTP", vec![], true);

        let batch = spans_to_batch(&[gen, retriever, embedding, chain, http]);
        let types = types_in(&batch);
        assert!(
            types
                .iter()
                .all(|t| LANGFUSE_V31_EMITTED_ENVELOPE_TYPES.contains(&t.as_str())),
            "illegal envelope type in {types:?}"
        );
        assert!(types.contains(&LANGFUSE_V31_TRACE_CREATE.to_string()));
        assert!(types.contains(&LANGFUSE_V31_GENERATION_CREATE.to_string()));
        assert!(types.contains(&LANGFUSE_V31_SPAN_CREATE.to_string()));
        assert!(
            !types
                .iter()
                .any(|t| t.contains("retriever") || t.contains("embedding") || t.contains("chain")),
            "must not stringify LAW-124-13 types into envelope: {types:?}"
        );

        let gen_ev = batch
            .iter()
            .find(|e| e["type"] == LANGFUSE_V31_GENERATION_CREATE)
            .expect("generation-create");
        assert_eq!(gen_ev["body"]["name"], "generate-answer");
        assert_eq!(gen_ev["body"]["usage"]["input"], 10);
        assert_eq!(gen_ev["body"]["usage"]["output"], 4);

        let names_as_span: Vec<_> = batch
            .iter()
            .filter(|e| e["type"] == LANGFUSE_V31_SPAN_CREATE)
            .map(|e| e["body"]["name"].as_str().unwrap_or("").to_string())
            .collect();
        for want in [
            "retrieval edgequake",
            "query.embed",
            "ingest.document",
            "HTTP",
        ] {
            assert!(
                names_as_span.iter().any(|n| n == want),
                "missing span-create for {want}: {names_as_span:?}"
            );
        }
    }

    #[test]
    fn usage_parses_string_tokens() {
        let span = fake_span(
            "generate-answer",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_GENERATION),
                KeyValue::new(GEN_AI_USAGE_INPUT_TOKENS, "11"),
                KeyValue::new(GEN_AI_USAGE_OUTPUT_TOKENS, "5"),
            ],
            true,
        );
        let batch = spans_to_batch(&[span]);
        let gen = batch
            .iter()
            .find(|e| e["type"] == LANGFUSE_V31_GENERATION_CREATE)
            .expect("generation");
        assert_eq!(gen["body"]["usage"]["input"], 11);
        assert_eq!(gen["body"]["usage"]["total"], 16);
    }

    /// Fail-closed live round-trip against Langfuse **3.1.1**.
    ///
    /// Default `cargo test` no-ops unless `LANGFUSE_311_E2E=1` (so CI without
    /// Docker stays green). `make spec124-langfuse-3.1-e2e` sets the env and
    /// **must** talk to a real 3.1.1 — missing base/keys panics.
    #[test]
    fn live_langfuse_3_1_1_ingestion_roundtrip() {
        if std::env::var("LANGFUSE_311_E2E").ok().as_deref() != Some("1") {
            return;
        }
        let base = std::env::var("LANGFUSE_311_E2E_BASE")
            .or_else(|_| std::env::var("LANGFUSE_BASE_URL"))
            .expect("LANGFUSE_311_E2E=1 requires LANGFUSE_311_E2E_BASE");
        let pk = std::env::var("LANGFUSE_PUBLIC_KEY").expect("LANGFUSE_PUBLIC_KEY");
        let sk = std::env::var("LANGFUSE_SECRET_KEY").expect("LANGFUSE_SECRET_KEY");
        let base = base.trim_end_matches('/').to_string();

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("client");
        let token = crate::langfuse::basic_auth_token(&pk, &sk);

        let health: JsonValue = client
            .get(format!("{base}/api/public/health"))
            .send()
            .expect("health")
            .json()
            .expect("health json");
        let ver = health
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        assert!(
            ver.starts_with("3.1."),
            "expected Langfuse 3.1.x, got version={ver:?} (unfakable pin)"
        );

        let otlp = client
            .post(format!("{base}/api/public/otel/v1/traces"))
            .header("Authorization", format!("Basic {token}"))
            .header("Content-Type", "application/x-protobuf")
            .body(Vec::<u8>::new())
            .send()
            .expect("otlp probe");
        assert_eq!(
            otlp.status().as_u16(),
            404,
            "3.1.1 must 404 OTLP (got {})",
            otlp.status()
        );

        assert_eq!(
            crate::langfuse::probe_langfuse_api(&base, &token),
            crate::langfuse::LangfuseApi::Ingestion,
            "3.1.1 auto-probe must resolve to ingestion"
        );

        let illegal = json!({
            "batch": [{
                "id": envelope_uuid("illegal", "retriever"),
                "timestamp": rfc3339(SystemTime::now()),
                "type": "retriever-create",
                "body": {
                    "id": "abc123deadbeef",
                    "traceId": "11".repeat(16),
                    "name": "must-be-rejected",
                    "startTime": rfc3339(UNIX_EPOCH),
                    "endTime": rfc3339(UNIX_EPOCH + Duration::from_secs(1)),
                }
            }]
        });
        let illegal_resp = client
            .post(format!("{base}/api/public/ingestion"))
            .header("Authorization", format!("Basic {token}"))
            .json(&illegal)
            .send()
            .expect("illegal post");
        let illegal_code = illegal_resp.status().as_u16();
        let illegal_body = illegal_resp.text().unwrap_or_default();
        assert!(
            ingestion_http_outcome(illegal_code, &illegal_body).is_err(),
            "retriever-create must produce 207 errors on 3.1.1, body={illegal_body}"
        );

        let marker = format!("eq-311-{}", uuid::Uuid::new_v4());
        let mut tid = [0u8; 16];
        tid.copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        let trace = TraceId::from_bytes(tid);
        let start = SystemTime::now();
        let end = start + Duration::from_secs(1);
        let gen = fake_span_on_at(
            "generate-answer",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_GENERATION),
                KeyValue::new("gen_ai.request.model", "gpt-5-nano"),
                KeyValue::new(GEN_AI_USAGE_INPUT_TOKENS, 3i64),
                KeyValue::new(GEN_AI_USAGE_OUTPUT_TOKENS, 1i64),
                KeyValue::new(LANGFUSE_OBSERVATION_INPUT, marker.clone()),
            ],
            false,
            trace,
            start,
            end,
        );
        let retriever = fake_span_on_at(
            "retrieval edgequake",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_RETRIEVER),
                KeyValue::new(LANGFUSE_OBSERVATION_INPUT, marker.clone()),
            ],
            false,
            trace,
            start,
            end,
        );
        let embedding = fake_span_on_at(
            "query.embed",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_EMBEDDING),
                KeyValue::new("gen_ai.request.model", "text-embedding-3-small"),
                KeyValue::new(LANGFUSE_OBSERVATION_INPUT, marker.clone()),
            ],
            false,
            trace,
            start,
            end,
        );
        let chain = fake_span_on_at(
            "ingest.document",
            vec![
                KeyValue::new(LANGFUSE_OBSERVATION_TYPE, OBSERVATION_TYPE_CHAIN),
                KeyValue::new(LANGFUSE_OBSERVATION_INPUT, marker.clone()),
            ],
            true,
            trace,
            start,
            end,
        );
        let spans = vec![gen, retriever, embedding, chain];
        let events = spans_to_batch(&spans);
        for ev in &events {
            let ty = ev["type"].as_str().unwrap_or("");
            assert!(LANGFUSE_V31_EMITTED_ENVELOPE_TYPES.contains(&ty));
        }
        let exporter = LangfuseIngestionExporter::new(&base, &pk, &sk);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio rt for exporter.export");
        rt.block_on(exporter.export(spans))
            .unwrap_or_else(|e| panic!("LangfuseIngestionExporter::export failed: {e}"));

        let trace_hex = super::trace_hex(trace);
        let mut found_gen = false;
        let mut found_span = false;
        for _ in 0..30 {
            std::thread::sleep(Duration::from_secs(2));
            for name in [
                "generate-answer",
                "retrieval edgequake",
                "query.embed",
                "ingest.document",
            ] {
                let list = client
                    .get(format!(
                        "{base}/api/public/observations?limit=50&name={}",
                        urlencoding_name(name)
                    ))
                    .header("Authorization", format!("Basic {token}"))
                    .send()
                    .ok()
                    .and_then(|r| r.json::<JsonValue>().ok());
                let Some(list) = list else { continue };
                let rows = list
                    .get("data")
                    .and_then(|d| d.as_array())
                    .cloned()
                    .unwrap_or_default();
                for row in rows {
                    let row_name = row.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let ty = row
                        .get("type")
                        .or_else(|| row.get("observationType"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    let input = row.get("input").map(|v| v.to_string()).unwrap_or_default();
                    let row_trace = row
                        .get("traceId")
                        .or_else(|| row.get("trace_id"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    if !input.contains(&marker) && row_trace != trace_hex {
                        continue;
                    }
                    if row_name == "generate-answer" && ty.eq_ignore_ascii_case("GENERATION") {
                        found_gen = true;
                    }
                    if (row_name == "retrieval edgequake"
                        || row_name == "query.embed"
                        || row_name == "ingest.document")
                        && ty.eq_ignore_ascii_case("SPAN")
                    {
                        found_span = true;
                    }
                }
            }
            if found_gen && found_span {
                break;
            }
        }
        assert!(
            found_gen && found_span,
            "3.1.1 did not persist mapped observations via LangfuseIngestionExporter (gen={found_gen} span={found_span} marker={marker} trace={trace_hex})"
        );
    }

    fn urlencoding_name(name: &str) -> String {
        name.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    (b as char).to_string()
                }
                b' ' => "%20".into(),
                _ => format!("%{b:02X}"),
            })
            .collect()
    }
}
