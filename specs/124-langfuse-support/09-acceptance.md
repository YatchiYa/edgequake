# 09 — Acceptance

## Definition of Done

| # | Criterion | Evidence |
|---|-----------|----------|
| A1 | Doc pack complete with cross-refs + ASCII | `specs/124-langfuse-support/` |
| A2 | Skill vendored | `.github/skills/langfuse/` |
| A3 | OTLP/HTTP Langfuse exporter behind `otel` + env | code + unit tests |
| A4 | Dual export: gRPC path preserved | subscriber still builds Jaeger path |
| A5 | Generation spans wired on query path | call sites + tests |
| A6 | Health + GET settings/langfuse (no secrets) | API tests |
| A7 | Settings card + Open in Langfuse | Playwright / manual |
| A8 | Edge-case matrix mitigated | [10-edge-cases.md](10-edge-cases.md) checked |
| A9 | OBSERVABILITY.md + .env.example updated | docs |
| A10 | Sessions: chat conversation_id → Langfuse Sessions | [12-sessions-and-genai.md](12-sessions-and-genai.md) + e2e |
| A11 | Token usage on generation spans (never cost attrs) | [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md) + in-memory OTEL |
| A12 | Stream / bypass / keywords / **gleaning** / ingest / embed coverage | call sites + stream + gleaning contract tests |
| A13 | Observation Input/Output non-null on GenAI + key workflow spans | [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md) + InMemory |
| A14 | DRY SSOT helpers (`with_llm_generation`, `stamp_query_langfuse`, …) | I7 + `make spec124-proof` |
| A15 | Honest session Open link (not fake TraceId) | WebUI `LangfuseOpenSessionLink` + vitest href |
| A16 | Tenant/workspace slugs additive to GUIDs | [15-pipeline-observe-and-slugs.md](15-pipeline-observe-and-slugs.md) + InMemory |
| A17 | Query pipeline meta + ingest parse/vision/chunk stages | I8 + `make spec124-proof` |
| A18 | Local Langfuse v4 Docker (optional) + smoke/E2E | `make langfuse-up` / `make spec124-langfuse-e2e` |

## Partner acceptance script

1. Build (otel is default) / restart
2. Export `LANGFUSE_PUBLIC_KEY`, `LANGFUSE_SECRET_KEY`, `LANGFUSE_BASE_URL`
3. `make backend-bg` (or equivalent)
4. Open `/settings` → Langfuse card Enabled → Open in Langfuse
5. Run a Mix query → retrieval + generation with **Usage** and **Input/Output** populated
6. Send two chat turns (same conversation) → Observability → Sessions shows the conversation UUID; assistant metadata shows **Open session in Langfuse**
7. Ingest a document → `ingest.document` / `pipeline_chunk_extraction` / glean generations Input/Output non-null
8. Confirm EdgeQuake did not send cost attrs (CI denylist); Langfuse may still show computed cost from model pricing
9. `make spec124-proof` passes without Langfuse credentials
10. Optional local Docker: `make langfuse-up` then point `.env` at `http://localhost:3310` + init keys, restart backend, `make spec124-langfuse-e2e`

## Honest gaps allowed in v1

- Prompt management not migrated
- Token usage may be missing if provider omits usage (true `stream()` without final counts)
- Per-trace `/trace/{id}` deep-link deferred until API `trace_id` ≡ OTEL TraceId — use Sessions instead
- Product USD cost tracker is intentionally **not** exported
- Middleware SPANs without I/O (LAW-124-18)

## Cross-refs

- Impl: [07-implementation-plan.md](07-implementation-plan.md)
- Tokens: [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md)
- Observation I/O: [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md)
- PO: [05-lenses/001-product-owner.md](05-lenses/001-product-owner.md)
