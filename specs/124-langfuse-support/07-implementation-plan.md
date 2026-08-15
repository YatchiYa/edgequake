# 07 — Implementation Plan

## Principles

- **DRY:** one `LangfuseConfig::from_env()`; FE mirrors DTO; reuse ExternalLink patterns
- **SOLID:** exporter init ≠ span helpers ≠ status handler ≠ UI card
- **First principles:** HTTP for Langfuse; env-only secrets; non-blocking export
- Docs first, then code phases below

## Phase 0 — Skill (done when vendored)

- [x] Vendor [langfuse/skills](https://github.com/langfuse/skills) → `.github/skills/langfuse/`
- Use `references/instrumentation.md` + live best-practices when wiring spans

## Phase 1 — Observability SSOT

| Step | File(s) | Change |
|------|---------|--------|
| 1.1 | `edgequake-observability/Cargo.toml` | `otel` feature: add `http-proto` (+ reqwest client features as needed) alongside `grpc-tonic` |
| 1.2 | `src/langfuse.rs` (new) | `LangfuseConfig`, auth header builder, endpoint URL helper, unit tests |
| 1.3 | `src/subscriber.rs` | Read Langfuse config; register HTTP exporter processor when enabled; keep gRPC path |
| 1.4 | `src/lib.rs` | Export `LangfuseConfig` |
| 1.5 | Docs | Update `docs/OBSERVABILITY.md`, `.env.example` |

## Phase 2 — Instrumentation

| Step | File(s) | Change |
|------|---------|--------|
| 2.1 | `rag_span.rs` | Enrich generation helper: stable `otel.name`, optional usage fields, operation name aliases (`generate-answer`) |
| 2.2 | `edgequake-query` `prompt.rs` / stream paths | Wrap LLM complete/chat with `with_rag_generation_span` |
| 2.3 | `edgequake-pipeline` extract | Wrap extract chat with generation span |
| 2.4 | API query response | Optional `trace_id` when OTEL active (`trace_id_from_request_id` / current span) |
| 2.5 | Propagation | Attach tenant/workspace/user/session on HTTP span metadata — **done** via `bind_langfuse_identity` + baggage processor ([12-sessions-and-genai.md](12-sessions-and-genai.md)) |
| 2.6 | Sessions | Chat `conversation_id` → Langfuse session; optional `/query` `session_id`; Playwright sessions e2e |

## Phase 3 — API + WebUI

| Step | File(s) | Change |
|------|---------|--------|
| 3.1 | `health_types.rs` / health builder | `langfuse_enabled`, `langfuse_base_url` |
| 3.2 | `handlers/settings_langfuse.rs` + routes | `GET /api/v1/settings/langfuse` |
| 3.3 | OpenAPI | Refresh if required by contract tests |
| 3.4 | `langfuse-observability-card.tsx` | Settings card |
| 3.5 | `settings/page.tsx` | Mount card |
| 3.6 | Query chrome | Conditional Open-trace link |

## Phase 4 — Tests

See [08-test-protocol.md](08-test-protocol.md) and [10-edge-cases.md](10-edge-cases.md).

## File ownership (SRP)

```ascii
  langfuse.rs          → config + URL/auth pure functions
  subscriber.rs        → wire exporters only
  rag_span.rs          → GenAI span helpers only
  settings_langfuse.rs → HTTP status DTO only
  *-card.tsx           → present DTO only
```

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
