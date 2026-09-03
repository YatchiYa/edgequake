//! Langfuse **native ingestion** span exporter (`POST /api/public/ingestion`).
//!
//! # Why this exists
//!
//! EdgeQuake exports traces over OTLP/HTTP to `/api/public/otel/v1/traces`.
//! That endpoint only exists from Langfuse **3.22x** onward — on **Langfuse 3.1
//! it returns 404**, so no trace can ever arrive, whatever the network setup.
//! Self-hosted fleets that cannot upgrade were therefore left without tracing.
//!
//! The native ingestion API (`/api/public/ingestion`) has shipped since
//! Langfuse v2 and is present on 3.1. This exporter maps OpenTelemetry spans
//! onto that API, so the same instrumentation feeds both transports.
//!
//! Selected via [`crate::langfuse::LangfuseApi`]; OTLP remains the default, so
//! existing deployments are untouched.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry::Value;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::trace::{SpanData, SpanExporter};
use serde_json::{json, Map as JsonMap, Value as JsonValue};

/// Langfuse observation kinds we emit.
const TYPE_GENERATION: &str = "generation";
const TYPE_SPAN: &str = "span";

/// Span exporter that speaks Langfuse's native ingestion API.
#[derive(Debug)]
pub struct LangfuseIngestionExporter {
    endpoint: String,
    auth_header: String,
    /// Built lazily on the batch-processor thread.
    ///
    /// `BatchSpanProcessor` polls `export()` on a **dedicated thread with no
    /// Tokio reactor**, so an async client panics with "there is no reactor
    /// running" — this is why the upstream OTLP exporter ships a blocking
    /// client. A blocking client cannot be *constructed* inside an async
    /// context either (it spawns its own runtime), and `new()` runs during
    /// observability init, which is async. Building it on first export
    /// satisfies both constraints.
    client: OnceLock<reqwest::blocking::Client>,
}

impl LangfuseIngestionExporter {
    /// Build an exporter for `base_url` (no trailing path) with Basic auth.
    pub fn new(base_url: &str, public_key: &str, secret_key: &str) -> Self {
        let token = crate::langfuse::basic_auth_token(public_key, secret_key);
        Self {
            endpoint: format!("{}/api/public/ingestion", base_url.trim_end_matches('/')),
            auth_header: format!("Basic {token}"),
            client: OnceLock::new(),
        }
    }

    /// Endpoint this exporter posts to (diagnostics).
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// RFC3339 (UTC, millisecond precision) — the format Langfuse expects.
fn rfc3339(ts: SystemTime) -> String {
    let d = ts.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs() as i64;
    let millis = d.subsec_millis();
    // Civil-from-days (Howard Hinnant's algorithm) — avoids a chrono dependency
    // in a crate that does not otherwise need one.
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

fn value_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Bool(b) => json!(b),
        Value::I64(i) => json!(i),
        Value::F64(f) => json!(f),
        Value::String(s) => json!(s.as_str()),
        other => json!(other.to_string()),
    }
}

/// Attributes → (json map, langfuse-specific extractions).
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
            "langfuse.observation.type" => e.observation_type = Some(v.to_string()),
            "gen_ai.request.model" | "gen_ai.response.model" | "langfuse.observation.model.name" => {
                e.model = Some(v.to_string())
            }
            "langfuse.observation.input" | "gen_ai.prompt" | "input.value" => {
                e.input = Some(value_to_json(v))
            }
            "langfuse.observation.output" | "gen_ai.completion" | "output.value" => {
                e.output = Some(value_to_json(v))
            }
            "gen_ai.usage.input_tokens" | "gen_ai.usage.prompt_tokens" => {
                if let Value::I64(i) = v {
                    e.usage_in = Some(*i)
                }
            }
            "gen_ai.usage.output_tokens" | "gen_ai.usage.completion_tokens" => {
                if let Value::I64(i) = v {
                    e.usage_out = Some(*i)
                }
            }
            "langfuse.session.id" | "session.id" => e.session_id = Some(v.to_string()),
            "langfuse.user.id" | "user.id" => e.user_id = Some(v.to_string()),
            "langfuse.observation.level" => e.level = Some(v.to_string()),
            _ => {
                e.attributes.insert(k.to_string(), value_to_json(v));
            }
        }
    }
    e
}

/// Map a batch of OTel spans onto Langfuse ingestion events.
///
/// Root spans (no parent) additionally emit a `trace-create` so the trace shows
/// a name, session and user in the UI. Every span becomes an observation —
/// `generation` when it carries model/usage attributes, `span` otherwise.
/// Span names that describe a request better than the transport root.
///
/// The OTel root of an API call is the Axum `HTTP` server span, which makes
/// every Langfuse trace show up as "HTTP". Prefer a RAG/pipeline span name so
/// the trace list is readable.
const PREFERRED_TRACE_NAMES: &[&str] = &[
    "query_pipeline",
    "query_execute",
    "chat_stream",
    "query_stream",
    "ingest.document",
    "task_process",
];

/// Map a batch of OTel spans onto Langfuse ingestion events.
///
/// One `trace-create` per trace id, carrying a readable name plus the
/// session/user ids found on **any** span of that trace — Langfuse propagates
/// those onto all spans, but the transport root (`HTTP`) is created before the
/// baggage exists, so reading only the root loses them. The ingestion API
/// upserts traces, so later batches enrich the same trace.
///
/// Every span becomes an observation — `generation` when it carries model or
/// usage attributes, `span` otherwise.
pub fn spans_to_batch(batch: &[SpanData]) -> Vec<JsonValue> {
    let mut events = Vec::with_capacity(batch.len() * 2);

    // Per-trace aggregation: name candidate, session, user, root timestamp.
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
        // A preferred name always wins; otherwise the root names the trace.
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
            "id": format!("{trace_id}-trace"),
            "timestamp": info.timestamp,
            "type": "trace-create",
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

        let is_generation = e.observation_type.as_deref() == Some(TYPE_GENERATION)
            || e.model.is_some()
            || e.usage_in.is_some()
            || e.usage_out.is_some();
        let obs_type = if is_generation {
            TYPE_GENERATION
        } else {
            e.observation_type.as_deref().unwrap_or(TYPE_SPAN)
        };

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
        if e.usage_in.is_some() || e.usage_out.is_some() {
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
            "id": format!("{span_id}-obs"),
            "timestamp": end,
            "type": format!("{obs_type}-create"),
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
            reqwest::blocking::Client::builder()
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
            .map_err(|e| {
                // WARN, not DEBUG: a failing export is otherwise invisible at the
                // default RUST_LOG=info, and the most common causes here are
                // silent ones — an internal CA the trust store rejects, or an
                // ingress dropping the request.
                tracing::warn!(
                    endpoint = %self.endpoint,
                    error = %e,
                    "Langfuse ingestion export failed (traces lost for this batch)"
                );
                OTelSdkError::InternalFailure(format!("langfuse ingestion send: {e}"))
            })?;

        // 207 Multi-Status is the nominal success code for this API.
        let code = resp.status().as_u16();
        if resp.status().is_success() || code == 207 {
            Ok(())
        } else {
            let body = resp.text().unwrap_or_default();
            let snippet: String = body.chars().take(300).collect();
            tracing::warn!(
                endpoint = %self.endpoint,
                status = code,
                body = %snippet,
                "Langfuse ingestion rejected the batch (traces lost)"
            );
            Err(OTelSdkError::InternalFailure(format!(
                "langfuse ingestion HTTP {code}: {snippet}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_formats_epoch() {
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn rfc3339_formats_known_instant() {
        // 2026-08-26T12:00:00Z
        let ts = UNIX_EPOCH + Duration::from_secs(1_787_745_600);
        assert!(ts_is_2026(&rfc3339(ts)), "{}", rfc3339(ts));
    }

    fn ts_is_2026(s: &str) -> bool {
        s.starts_with("2026-") && s.ends_with('Z') && s.contains('T')
    }

    #[test]
    fn endpoint_joins_path_once() {
        let e = LangfuseIngestionExporter::new("http://lf:3000/", "pk", "sk");
        assert_eq!(e.endpoint(), "http://lf:3000/api/public/ingestion");
    }
}
