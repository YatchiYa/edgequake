# 03 — Code As-Is (post SPEC-124 I6)

## Observability crate

| Path | Role |
|------|------|
| `edgequake-observability/src/subscriber.rs` | Dual OTLP: gRPC (Jaeger) + HTTP (Langfuse); baggage processor |
| `…/langfuse.rs` | `LangfuseConfig::from_env`, OTLP URL/auth, unquote |
| `…/langfuse_attrs.rs` | Session/user/usage keys, I/O keys, `COST_ATTR_DENYLIST`, observation types |
| `…/langfuse_context.rs` | `bind_langfuse_identity`, `with_langfuse_identity_async` |
| `…/baggage_span_processor.rs` | Allowlisted baggage → span attributes |
| `…/rag_span.rs` | GenAI spans + `record_gen_ai_usage` + `record_observation_io` (+ retrieval/embed/ingest helpers) |
| `…/http_span.rs` | HTTP request spans |

## Export path

```ascii
  init_observability
       │
       └─ SdkTracerProvider
            ├─ LangfuseBaggageSpanProcessor
            ├─ BatchSpanProcessor → OTLP gRPC (if endpoint / EDGEQUAKE_OTEL_ENABLED)
            └─ BatchSpanProcessor → OTLP HTTP Langfuse
                 POST {base}/api/public/otel/v1/traces
```

## GenAI + I/O call sites

| Path | Span / helper |
|------|----------------|
| `query/engine_impl/prompt.rs` | `generate-answer`, bypass + usage + I/O |
| `query/.../query_stream.rs` | stream generation + usage/I/O |
| `query/.../query_pipeline.rs` / `arm_timed.rs` | retriever + retrieval I/O summary |
| `query/keywords/llm_extractor.rs` | `extract-keywords` + usage/I/O |
| `handlers/query/*` / `chat/*` | root `record_observation_io` after answer |
| `pipeline/extractor/sota.rs` | `extract-entities` + usage/I/O |
| `pipeline/summarizer.rs` | `summarize-*` + usage/I/O |
| `pipeline/helpers/embeddings.rs` | `embed-chunks` + I/O |
| `pipeline/processing.rs` | `ingest.document` + I/O |
| `pipeline/extraction.rs` | `pipeline_chunk_extraction` + I/O |

**Forbidden:** `COST_ATTR_DENYLIST`. See [13](13-metadata-tokens-and-coverage.md) / [14](14-observation-io-and-full-observe.md).

## Session bind call sites

| Path | When |
|------|------|
| `handlers/chat/completion.rs` | After `conversation_id` + feature tag |
| `handlers/chat/streaming.rs` | Outer bind + spawn re-bind |
| `handlers/query/query_execute.rs` | Optional `session_id` |
| `handlers/query/query_stream.rs` | Optional `session_id` |

## Health / Settings

- `GET /api/v1/settings/langfuse` — no secrets
- Settings card + Open in Langfuse
- `/health.operational.observability.langfuse_*`
