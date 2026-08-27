# 04 — Target Architecture

## Export topology

```ascii
  tracing Subscriber
       │
       ├─ fmt layer (plain|json)
       ├─ metrics (Prometheus)
       └─ OpenTelemetryLayer
              │
              SdkTracerProvider
                   │
                   ├─ BatchSpanProcessor → OTLP gRPC (Jaeger)     [if OTEL endpoint]
                   └─ BatchSpanProcessor → Langfuse              [if LANGFUSE keys]
                                              │
                                              probe OTLP (auto)
                                              ├─ ≥ 3.22 / Cloud → POST {base}/api/public/otel/v1/traces
                                              └─ 3.1.x 404 → POST {base}/api/public/ingestion
                                                 (trace-create / span-create / generation-create)
                                              Authorization: Basic pk:sk
                                              x-langfuse-ingestion-version: 4  (OTLP path)
```

## Config SSOT

```ascii
  LangfuseConfig::from_env()
       ├─ public_key_present: bool
       ├─ secret_key_present: bool
       ├─ base_url: String          // LANGFUSE_BASE_URL || LANGFUSE_HOST || EU cloud
       ├─ enabled: bool             // keys + EDGEQUAKE_LANGFUSE_ENABLED
       └─ ui_url: String            // same as base_url (Open link)

  ObservabilityConfig
       ├─ (existing) otel_enabled, log_format, service_name
       └─ langfuse: LangfuseConfig
```

Build: feature `otel` enables both exporters’ dependencies (`grpc-tonic` + `http-proto`).

## Span tree (query)

```ascii
  http_request / query_stream / chat_*
    └─ tags: langfuse.trace.tags=query (+ session when bound; ids+slugs)
         ├─ extract-keywords | keyword-cache | keyword-heuristic
         ├─ query.embed                 type=embedding
         ├─ rag.retrieval               type=retriever
         ├─ query.fuse                  mix/hybrid
         ├─ query.rerank
         └─ generate-answer | generate-bypass-answer | answer-cache
              rag.generation type=generation
              attrs: gen_ai.request.model, gen_ai.provider.name,
                     gen_ai.usage.input_tokens / output_tokens (never cost)
```

## Span tree (ingest)

```ascii
  ingest.task                           type=chain, tags=ingest, session=document_id
         ├─ ingest.converting           parser / fallback
         │    ├─ ingest.pass_a          vision OCR
         │    └─ ingest.pass_b          figure VLM
         ├─ ingest.document             KG slice
         │    ├─ ingest.chunking
         │    ├─ pipeline_chunk_extraction
         │    │    extract-entities / extract-entities-glean
         │    └─ embed-chunks
         ├─ ingest.persist
         └─ summarize-*                 generation + usage
```

Cost attrs (`gen_ai.usage.cost`, `langfuse.observation.cost_details`) are **forbidden** (LAW-124-12). Details: [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md).

## Attribute map (Langfuse)

| Concern            | Attribute                                                                                             |          |
| --------------------| -------------------------------------------------------------------------------------------------------| ----------|
| User               | `user.id` / `langfuse.user.id`                                                                        |          |
| Session            | `session.id` / `langfuse.session.id` / `gen_ai.conversation.id` (same value = chat `conversation_id`) |          |
| Tenant / workspace | `langfuse.trace.metadata.tenant_id`, `…workspace_id` **and** `…tenant_slug`, `…workspace_slug` (LAW-124-19) |          |
| Feature tag        | `langfuse.trace.tags` = `query` \                                                                     | `ingest` |
| Environment        | `deployment.environment` / `langfuse.environment`                                                     |          |
| Request id         | metadata `request_id`                                                                                 |          |

Propagate to **all** spans (LAW-124-9) via allowlisted baggage + `LangfuseBaggageSpanProcessor`. Bind after chat resolves `conversation_id` (`bind_langfuse_identity` / `with_langfuse_identity_async`). Details: [12-sessions-and-genai.md](12-sessions-and-genai.md).

## Deep links

| Link | When | URL |
|------|------|-----|
| Project / home | Configured | `{LANGFUSE_BASE_URL}` |
| Per-trace | Response has `trace_id` | `{base}/trace/{trace_id}` |

## API surface

```ascii
  GET /health
    operational.observability.langfuse_enabled
    operational.observability.langfuse_base_url
    capabilities.langfuse_ui_url?   (optional)

  GET /api/v1/settings/langfuse
    { enabled, base_url, public_key_configured, secret_key_configured,
      ui_url, otel_feature_built, config_requirements[] }
```

No PATCH for secrets in v1.

## Cross-refs

- As-is: [03-code-as-is.md](03-code-as-is.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Laws: [01-first-principles.md](01-first-principles.md)
