# 01 — First Principles (LAW-124)

## Domain

Langfuse is an **observability credential domain**, not a SPEC-123 model-resolution domain.

```ascii
  SPEC-123 (LLM / PDF / embed):  Request > Workspace > Tenant > Env > Default
  SPEC-124 (Langfuse):           Env only → ObservabilityConfig → exporters
                                 Workspace/Tenant MUST NOT override keys
```

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-124-1 | **HTTP for Langfuse** — never gRPC. Primary export is OTLP/HTTP to `{base}/api/public/otel/v1/traces` (Langfuse ≥ 3.22 / Cloud). Self-hosted **3.1.x** 404s that path; `EDGEQUAKE_LANGFUSE_API=auto` then falls back to `POST /api/public/ingestion`. Upgrade to ≥ 3.22 remains recommended (ingestion is deprecated; Cloud sunsets 2026-11-16). | Langfuse platform constraint |
| LAW-124-2 | **Env-only secrets** — `LANGFUSE_PUBLIC_KEY` / `LANGFUSE_SECRET_KEY` never in DB, never in API responses, never in UI fields | Matches OpenAI key pattern |
| LAW-124-3 | **Dual export independence** — Jaeger gRPC and Langfuse HTTP may both be on; neither requires the other | Ops flexibility |
| LAW-124-4 | **Non-blocking export** — batch processor; Langfuse down must not fail user requests | Reliability |
| LAW-124-5 | **One SSOT config** — `LangfuseConfig::from_env()` (or `ObservabilityConfig` fields); API/UI read DTO only | DRY / DIP |
| LAW-124-6 | **UI honesty** — “Open in Langfuse” only when configured; unsatisfied requirements show `— not set`, never secrets | SPEC-101 spirit |
| LAW-124-7 | **Stable observation names** — verb-first, no dynamic IDs in names (`generate-answer`, not `generate-answer-retry-2`) | Langfuse evaluators/dashboards |
| LAW-124-8 | **Explicit span I/O** — set meaningful input/output; never dump all args (API keys, full configs) | PII / best practices |
| LAW-124-9 | **Trace attributes on every span** — user/session/tenant/workspace/feature tags propagated for filter/aggregate | Langfuse OTel attribute propagation |
| LAW-124-10 | **Flush on shutdown** — `ObservabilityGuard` shuts down tracer provider | Scripts / short-lived processes |
| LAW-124-11 | **No synthetic conversation ids** — session / `gen_ai.conversation.id` only from durable chat `conversation_id` or explicit `/query` `session_id` | OTEL GenAI + Langfuse Sessions honesty |
| LAW-124-12 | **Tokens in, cost out** — emit `gen_ai.usage.input_tokens` / `output_tokens` when the provider returns them; **never** emit `gen_ai.usage.cost`, `langfuse.observation.cost_details`, or USD cost attrs | Honest usage; no product-cost leakage |
| LAW-124-13 | **Observation types** — generation → `generation`; retrieval → `retriever`; embed → `embedding`; ingest root → `chain` | Langfuse observation UX |
| LAW-124-14 | **One helper records usage** — `record_gen_ai_usage` in observability SSOT; call sites never invent attribute strings | DRY / DIP |
| LAW-124-15 | **Unfakable proof** — CI asserts span attrs via in-memory OTEL exporter (no skip for “no Langfuse keys”) | Honest acceptance |
| LAW-124-16 | **Observation I/O SSOT** — only `record_observation_io` / span helpers set `langfuse.observation.input`/`output` | DRY; Langfuse UI Input/Output |
| LAW-124-17 | **Map what Langfuse reads** — prefer `langfuse.observation.*`; dual-write `gen_ai.prompt`/`gen_ai.completion` | Official OTEL allowlist |
| LAW-124-18 | **GenAI + key workflow I/O** — generation/retriever/embedding/chain + `pipeline_chunk_extraction` get truncated I/O | Full observe without span noise |
| LAW-124-19 | **Slugs additive to GUIDs** — emit `tenant_slug` / `workspace_slug` metadata; never replace `tenant_id` / `workspace_id` | Human filters without losing identity |
| LAW-124-20 | **Filterable metadata prefix** — only `langfuse.trace.metadata.*` / `langfuse.observation.metadata.*` for Langfuse UI filters | Official OTEL mapping |
| LAW-124-21 | **Query pipeline meta SSOT** — `QueryPipelineMeta` from QueryStats via observability helpers | DRY; mode/fusion/rerank/cache visible |
| LAW-124-22 | **Ingest stages** — parse/vision/chunk/extract/embed/persist observations; ingest session = `document_id` | Full ingest observe |
| LAW-124-23 | **3.1.1 envelope SSOT** — `langfuse_v31_envelope_type` maps LAW-124-13 types onto `generation-create` / `span-create` only. Never `{retriever,embedding,chain}-create`. HTTP 207 must read `errors[]`. Probe fallback only on OTLP 404. | Unfakable 3.1.x bridge |

## Env contract

| Variable | Required | Purpose |
|----------|----------|---------|
| `LANGFUSE_PUBLIC_KEY` | Yes (to enable) | Public key `pk-lf-…` |
| `LANGFUSE_SECRET_KEY` | Yes (to enable) | Secret key `sk-lf-…` |
| `LANGFUSE_BASE_URL` | No (default cloud EU) | UI + OTLP base; alias `LANGFUSE_HOST` |
| `EDGEQUAKE_LANGFUSE_ENABLED` | No | Force on/off (`1`/`0`); default = keys present |
| `EDGEQUAKE_LANGFUSE_API` | No (default `auto`) | `auto` / `otlp` / `ingestion` — auto probes OTLP, ingest only on 404 |
| Build feature `otel` | Yes for export | Same as existing OTLP |

Default base: `https://cloud.langfuse.com`. OTLP path: `/api/public/otel/v1/traces`. Auth: Basic `base64(pk:sk)`. Header: `x-langfuse-ingestion-version: 4`. Ingestion fallback: `POST /api/public/ingestion` (3.1.1 fern types only).

## Trace unit

```ascii
  One HTTP request  →  one root trace (query turn)
  One ingest job    →  one root trace (document pipeline)
  Multi-turn chat   →  many traces + shared session_id (= conversation_id)
```

See [12-sessions-and-genai.md](12-sessions-and-genai.md), [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md), [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md), and [15-pipeline-observe-and-slugs.md](15-pipeline-observe-and-slugs.md).

## Cross-refs

- Why: [00-why.md](00-why.md)
- Sessions: [12-sessions-and-genai.md](12-sessions-and-genai.md)
- Tokens / coverage: [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md)
- Observation I/O: [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md)
- Pipeline / slugs: [15-pipeline-observe-and-slugs.md](15-pipeline-observe-and-slugs.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Skill: [../../.github/skills/langfuse/references/instrumentation.md](../../.github/skills/langfuse/references/instrumentation.md)
