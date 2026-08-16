# zz-raw — Intake (not the contract)

> **Status:** Intake snapshot for SPEC-131. Normative contract lives in `00`–`11` and lenses.
> **Source:** [GitHub #379](https://github.com/raphaelmansuy/edgequake/issues/379) (opened 2026-08-14 by @msc2106).

## Issue title

Feature: environment config to omit temperature + OpenAI Responses API support

## Summary (verbatim intent)

EdgeQuake’s LLM layer (`edgequake-llm`) uses the **Chat Completions** API (`POST /v1/chat/completions`) for ingestion extraction, query answering, title generation, and VLM calls.

An increasing number of models **do not support `temperature` on Chat Completions**, or only accept the provider’s default value. Examples on AWS Bedrock Mantle’s OpenAI-compatible endpoint:

- **Gemma 4** (e.g. `google.gemma-4-31b`) — rejects `temperature: 0`; only default (1) supported.
- **Grok 4.3** (e.g. `xai.grok-4.3`) — same class of restriction.

On some platforms, **newer model families are only exposed via the Responses API**, not Chat Completions — e.g. **ChatGPT 5.6 series** (`openai.gpt-5.6-*`) on AWS Bedrock. EdgeQuake cannot use these models today because all upstream calls go through Chat Completions.

EdgeQuake already omits `temperature` for a hardcoded subset (`temperature.rs`), but extending that list per model release is not sustainable.

## Proposed fixes (issue)

1. Env config to omit `temperature` (and optionally `reasoning_effort`) without per-model code changes.
2. Responses API support in `edgequake-llm` for providers that require it.

## Repro (issue)

```text
EDGEQUAKE_LLM_PROVIDER=openai
EDGEQUAKE_LLM_MODEL=google.gemma-4-31b   # or xai.grok-4.3
OPENAI_BASE_URL=https://bedrock-mantle.<region>.api.aws/openai/v1
```

Error during extraction:

```text
Unsupported value: 'temperature' does not support 0 with this model.
Only the default (1) value is supported.
(param: temperature) (code: unsupported_value)
```

Observed: 121 document ingest failures in one batch (Gemma 4); `failure_class=unknown`.

Separate mode: `EDGEQUAKE_LLM_MODEL=openai.gpt-5.6-luna` on Bedrock may fail entirely (Responses-only).

## Proposed env (issue)

| Variable | Intent |
|----------|--------|
| `EDGEQUAKE_LLM_OMIT_TEMPERATURE=true` | Do not send `temperature` on upstream Chat Completions |
| `EDGEQUAKE_LLM_OMIT_REASONING_EFFORT=true` | Same pattern as Mistral Large omit guard |
| `EDGEQUAKE_LLM_API_FORMAT=chat_completions\|responses` | Route to `/v1/responses` when `responses` |

## Acceptance criteria (issue)

- [ ] `OMIT_TEMPERATURE=true` prevents temperature; ingest succeeds on Mantle with Gemma 4 / Grok 4.3
- [ ] `API_FORMAT=responses` routes to `/v1/responses`
- [ ] Extraction and query produce equivalent structured JSON under Responses mode
- [ ] ChatGPT 5.6-series works on AWS Bedrock in Responses mode
- [ ] Configuration documented in setup guide

## Non-goals (issue)

- Replacing EdgeQuake’s own `/api/v1/chat/completions` **server** endpoint
- Dropping Chat Completions upstream before Responses parity on target providers

## Related code pointers (issue)

- `edgequake-pipeline/src/extractor/temperature.rs`
- `edgequake-pipeline/src/extractor/completion_options.rs`
- `edgequake-tasks/src/ingestion_reliability.rs`
- `edgequake-llm` (sibling crate / crates.io) — provider HTTP layer

## External references (issue + research Aug 2026)

- [OpenAI migrate to Responses](https://developers.openai.com/api/docs/guides/migrate-to-responses)
- [Why Responses API](https://developers.openai.com/blog/responses-api)
- [Open Responses specification](https://www.openresponses.org/specification)
- [Bedrock Mantle Responses](https://docs.aws.amazon.com/bedrock/latest/userguide/bedrock-mantle.html)
- [GPT-5.6 on Bedrock](https://aws.amazon.com/blogs/machine-learning/get-started-with-openai-gpt-5-6-sol-terra-and-luna-on-amazon-bedrock/)

## Analytical reproduction (no AWS required)

```ascii
  Call site builds CompletionOptions { temperature: Some(0.0) }
       │  (gemma-4 / grok-4.3 NOT in model_requires_default_temperature)
       ▼
  OpenAI / openai_compatible provider serializes temperature: 0
       │
       ▼
  Upstream returns 400 unsupported_value
       │
       ▼
  classify_ingestion_failure → Unknown  (no temperature / unsupported_value rule)
```

Wiremock / unit proof of body shape is sufficient for SPEC-131 P0; live Mantle is LIVE-131 gated.

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
