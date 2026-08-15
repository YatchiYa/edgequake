# 11 — Honest Assessment

## What this spec delivers

OTLP/HTTP Langfuse export, Sessions, GenAI observation types, token usage (never cost), Langfuse-mapped observation Input/Output, DRY SSOT helpers, gleaning coverage, session Open link, **tenant/workspace slugs alongside GUIDs**, and query/ingest **pipeline stage** metadata.

## Risks

| Risk | Honesty |
|------|---------|
| Token usage completeness | Depends on provider returning usage; true `stream()` may omit final counts |
| Langfuse UI Cost column | Platform may **compute** USD from model + tokens; we never emit cost attrs |
| I/O truncation | Previews capped (`OBSERVATION_IO_PREVIEW_CHARS`); full prompts not dumped (LAW-124-8) |
| Live e2e without keys | Playwright may skip; **InMemorySpanExporter tests do not** (LAW-124-15); `make spec124-proof` is the CI floor |

## Gaps closed

| Iteration | Closed |
|-----------|--------|
| I4 | Sessions from `conversation_id` / optional `session_id` |
| I5 | Stream/bypass/keywords; ingest root; tokens; cost denylist |
| I6 | `langfuse.observation.input`/`output` on retriever, embed, ingest, `pipeline_chunk_extraction`, query roots |
| I7 | DRY/SOLID SSOT helpers; gleaning `extract-entities-glean`; session Open link; `make spec124-proof` |
| I8 | Tenant/workspace slugs additive to GUIDs; query pipeline meta; ingest parse/vision/chunk/persist stages |

## Honest gaps remaining

- Per-trace `/trace/{api_trace_id}` still deferred (API `trace_id` ≠ OTEL TraceId) — Sessions link is the operator path.
- Product USD cost tracker stays out of OTEL.
- Not every middleware SPAN carries I/O (intentional noise control — LAW-124-18).
- Live Langfuse UI smoke after deploys still optional (partner script).

## Cross-refs

- I/O: [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md)
- Tokens: [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
