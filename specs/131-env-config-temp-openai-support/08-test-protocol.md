# 08 — Test protocol

> Laws: [01-first-principles.md](01-first-principles.md) · Plan: [07-implementation-plan.md](07-implementation-plan.md)

## Principles

1. **Wiremock > live** for body shape (LAW-131-10).
2. **Source contracts** catch call-site bypasses.
3. **Live Mantle** is optional gated (`LIVE-131`), not CI-blocking.

## Matrix

| ID | Layer | Assert |
|----|-------|--------|
| U-131-01 | unit pipeline | `resolve_effective_temperature("google.gemma-4-31b", 0.0)` with omit env → `None` |
| U-131-02 | unit pipeline | same model, env unset → `Some(0.0)` (today’s bug preserved until omit) |
| U-131-03 | unit pipeline | `gpt-5-nano` env unset → `None` (LAW-131-11) |
| U-131-04 | unit pipeline | `gpt-4o` env unset → `Some(0.0)` |
| U-131-05 | unit tasks | #379 error text → `LlmUnsupportedParam`, permanent, action token set |
| U-131-06 | unit llm | invalid `API_FORMAT=foo` → error |
| E2E-131-01 | wiremock llm | omit-temp=1 → Chat Completions JSON **lacks** `temperature` (OpenAI + openai_compatible) |
| E2E-131-02 | wiremock llm | gpt-5-nano options None → lacks temperature without omit env |
| E2E-131-03 | wiremock llm | omit-effort=1 → lacks `reasoning_effort` |
| E2E-131-04 | wiremock llm | format=responses → `POST /v1/responses` (or `{base}/responses`), body `store:false`, json schema → `text.format` |
| E2E-131-05 | wiremock llm | Responses SSE `response.output_text.delta` → content chunks |
| E2E-131-06 | unit/api | classifier path used by ingest status update stores `failure_class=llm_unsupported_param` |
| E2E-131-07 | source contract | no LLM `CompletionOptions { temperature: Some(` outside resolver tests (grep allowlist) |
| E2E-131-08 | wiremock | Responses output with leading `reasoning` item still yields message text in `LLMResponse.content` |
| LIVE-131-A | manual | Mantle Gemma/Grok extract with omit-temp succeeds |
| LIVE-131-B | manual | Mantle GPT-5.6 with format=responses extract/query JSON succeeds |

## Suggested commands

```bash
# Product
cargo test -p edgequake-pipeline resolve_effective_temperature -- --nocapture
cargo test -p edgequake-tasks llm_unsupported_param -- --nocapture

# LLM crate (sibling)
cd ../edgequake-llm
cargo test e2e_spec131 -- --nocapture

# Source contract (example)
rg -n "temperature:\s*Some\(" edgequake/crates --glob '*.rs' | rg -v 'temperature\.rs|tests/'
```

## Fixtures

- Capture request body like SPEC-126 prompt-cache wiremock.
- Responses success fixture: minimal `output` with one `message` + `output_text`.
- Responses stream fixture: event sequence with `response.output_text.delta`.

## Acc / mock

MockProvider ignores `API_FORMAT`. Acc default remains chat + no omit unless scenario file pins Mantle.

## Exit criteria

All U-131-* and E2E-131-01..08 green on PR. LIVE-131 recorded in measurements/ when credentials available (optional folder).

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
