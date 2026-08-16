# SPEC-131 — Env omit-temperature + OpenAI Responses API (#379)

> **Mission:** Give operators fleet env knobs to omit illegal LLM sampling/effort fields, and an upstream Responses API transport for models that are not available on Chat Completions — without rewriting EdgeQuake’s server chat facade.
>
> **Trigger:** [GitHub #379](https://github.com/raphaelmansuy/edgequake/issues/379).

## Short verdict

| Layer | Finding |
|-------|---------|
| Gap A | Temperature omit is a lagging model substring list; Gemma 4 / Grok 4.3 send `temperature:0` → Mantle `unsupported_value` → `failure_class=unknown` |
| Gap B | Upstream is Chat Completions only; Mantle GPT-5.6 is Responses-oriented / Responses-only |
| Fix A | `EDGEQUAKE_LLM_OMIT_TEMPERATURE` / `OMIT_REASONING_EFFORT` + SSOT resolver + wire strip + new permanent failure class |
| Fix B | `EDGEQUAKE_LLM_API_FORMAT=responses` with shared mapper, `store:false` |
| Non-goals | Product `/api/v1/chat/completions` rewrite; hosted tools / Conversations in v1; dropping Chat Completions default |

```ascii
  preferred temp / effort
         │
         ▼
  env omit OR model gate ──► CompletionOptions (Option fields)
         │
         ▼
  ApiFormat
    chat_completions ──► /chat/completions
    responses ─────────► /responses  (store:false)
```

## Document map

```ascii
 00-why
  → 01-first-principles (LAW-131-*)
  → 02-cross-ref-matrix
  → 03-code-as-is
  → 04-target-architecture
  → 05-lenses/ (PO, fullstack, DB, UX, front, AI Aug 2026)
  → 06-ux-ui-spec
  → 07-implementation-plan
  → 08-test-protocol
  → 09-acceptance
  → 10-edge-cases
  → 11-honest-assessment
  → zz-raw.md (intake, not the contract)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D0 | Intake `zz-raw.md` / #379 | Done |
| D1 | Doc pack (this folder) | Done |
| I0 | P0 omit temp/effort + classifier + call sites | Done |
| I1 | P1 Responses transport (OpenAI + openai_compatible) | Done |
| I2 | P2 docs / AGENTS / setup | Done |
| T1 | E2E-131 wiremock + units | Done |
| L1 | LIVE-131 Mantle | Optional / gated |

## Env knobs (normative)

| Variable | Default | Effect |
|----------|---------|--------|
| `EDGEQUAKE_LLM_OMIT_TEMPERATURE` | false | Never send `temperature` |
| `EDGEQUAKE_LLM_OMIT_REASONING_EFFORT` | false | Never send `reasoning_effort` |
| `EDGEQUAKE_LLM_API_FORMAT` | `chat_completions` | `responses` → `/v1/responses` |

## Related

- [#379](https://github.com/raphaelmansuy/edgequake/issues/379) — this feature
- [SPEC-109](../109-configurable-reasoning-effort/) — reasoning effort hierarchy / clamp
- [SPEC-045](../045-fix-ingestion-errors/) / [SPEC-057](../057-pipeline-reliability/) — failure_class honesty
- [SPEC-123](../123-env-config-priority/) — env/config honesty
- [SPEC-126](../126-provider-kv-cache/) — prompt cache (Responses parity follow-up)
- Sibling crate: `/Users/raphaelmansuy/Github/03-working/edgequake-llm`

## External pins

- [Migrate to Responses](https://developers.openai.com/api/docs/guides/migrate-to-responses)
- [Open Responses](https://www.openresponses.org/specification)
- [Bedrock Mantle](https://docs.aws.amazon.com/bedrock/latest/userguide/bedrock-mantle.html)
- [GPT-5.6 on Bedrock](https://aws.amazon.com/blogs/machine-learning/get-started-with-openai-gpt-5-6-sol-terra-and-luna-on-amazon-bedrock/)
