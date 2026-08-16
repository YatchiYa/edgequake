# 07 — Implementation plan

> Architecture: [04-target-architecture.md](04-target-architecture.md) · Tests: [08-test-protocol.md](08-test-protocol.md)

## Work packages

```ascii
  WP-0  Spec pack (this folder)                         docs
  WP-1  resolve_effective_temperature + env omit        product P0
  WP-2  Wire all call sites (extract/title/VLM/…)       product P0
  WP-3  OMIT_REASONING_EFFORT after role resolve        product P0
  WP-4  Wire-level strip in openai + openai_compatible  llm P0
  WP-5  IngestionFailureClass::LlmUnsupportedParam      product P0
  WP-6  ApiFormat + Responses mapper + provider branch  llm P1
  WP-7  Streaming Responses → StreamChunk               llm P1
  WP-8  wiremock E2E-131-* + source contract            both
  WP-9  .env.example / AGENTS / setup guide             docs P2
  WP-10 LIVE-131 Mantle gated (optional)                ops
```

## WP-1 — Temperature resolver SSOT

**File:** `edgequake/crates/edgequake-pipeline/src/extractor/temperature.rs`

- Add `resolve_effective_temperature(model, preferred)` implementing LAW-131-2/11.
- Keep `effective_temperature_for_model` as the heuristic gate (or inline behind resolver).
- Unit tests: gemma/grok with omit-env → None; gpt-5-nano without env → None; gpt-4o without env → Some(0.0).

Re-export from pipeline lib if other crates need it (api title, pdf VLM).

## WP-2 — Call sites

Replace direct preferred/hardcoded temps:

| Site | Preferred |
|------|-----------|
| `completion_options.rs` | 0.0 |
| `title_generator.rs` | 0.3 |
| `figure_filter.rs` | 0.0 via resolver (**remove bare Some(0.0)**) |
| Any chat server path that forces temp | respect omit |

## WP-3 — Omit reasoning effort

Helper `apply_omit_reasoning_effort(opt: Option<String>) -> Option<String>` reading `EDGEQUAKE_LLM_OMIT_REASONING_EFFORT`.

Apply after SPEC-109 resolve in extract/query/vlm/summary/keyword builders (DRY: one helper in core or llm).

## WP-4 — Wire defense

In `edgequake-llm` OpenAI + OpenAI-compatible (+ Azure if same pattern):

- Before serialize, if omit-temp env → force no temperature field.
- Optionally same for reasoning_effort omit env.

## WP-5 — Classifier

`edgequake-tasks/src/ingestion_reliability.rs`:

- Add `LlmUnsupportedParam`
- `as_str`, `from_token`, `recommended_action`, `is_permanent: true`
- Match issue error substrings
- Unit tests with verbatim #379 error text
- Update OpenAPI / UI maps if generated from enum

## WP-6 / WP-7 — Responses transport

New modules in `edgequake-llm`:

- `api_format.rs` — parse env; default chat_completions; invalid → error
- `responses_map.rs` — messages↔input, options↔body, output↔LLMResponse, SSE↔StreamChunk
- Always `store: false`
- Branch in `OpenAIProvider` and `OpenAICompatibleProvider` for chat + stream (+ tools if already on chat path; tools via Responses client-side only if trivial — else document chat-only tools until P1.1)

**Dependency note:** Prefer raw `reqwest` JSON for Responses in openai_compatible; for native OpenAI, use async-openai Responses API if available in pinned crate version, else BYOT/reqwest same mapper.

## WP-8 — Tests

See [08-test-protocol.md](08-test-protocol.md). Gate names `e2e_spec131_*`.

## WP-9 — Docs

`.env.example`, AGENTS.md env table, setup guide Mantle section, optional GitHub #379 comment linking SPEC-131.

## DRY / SOLID checklist

- [ ] One temperature resolver used everywhere
- [ ] One Responses mapper shared by two providers
- [ ] No second model substring list for Gemma/Grok
- [ ] Classifier permanent + actionable
- [ ] Default transport remains chat_completions
- [ ] Product chat server facade untouched
- [ ] `store:false` asserted in wiremock

## Suggested PR split

1. P0: WP-1..WP-5 + E2E-131-01/02/03/06/07  
2. P1: WP-6..WP-7 + E2E-131-04/05  
3. P2: WP-9 docs (+ LIVE-131 manual)

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- Fullstack lens: [05-lenses/002-fullstack.md](05-lenses/002-fullstack.md)
