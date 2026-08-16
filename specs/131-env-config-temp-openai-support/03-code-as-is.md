# 03 — Code as-is (grounded)

> Snapshot for SPEC-131. Paths relative to monorepo root unless noted.
> Sibling crate: `/Users/raphaelmansuy/Github/03-working/edgequake-llm` (patched via `edgequake/Cargo.toml`).

## Axis A — Temperature today

### Model gate (pipeline)

[`edgequake/crates/edgequake-pipeline/src/extractor/temperature.rs`](../../edgequake/crates/edgequake-pipeline/src/extractor/temperature.rs)

```ascii
  effective_temperature_for_model(model, preferred) -> Option<f32>
       │
       ├─ model_requires_default_temperature?
       │     normalized = last path segment, lowercased
       │     omit if contains: gpt-5 | gpt-4.1-nano | gpt-4.1-mini
       │                or starts_with: o1 | o3 | o4
       │
       ├─ yes → None
       └─ no  → Some(preferred)
```

**Gap:** `google.gemma-4-31b`, `xai.grok-4.3`, Bedrock Mantle ids → **not** omitted → extract sends `0.0`.

### Extract options builder

[`.../extractor/completion_options.rs`](../../edgequake/crates/edgequake-pipeline/src/extractor/completion_options.rs)

```text
extraction_completion_options_with_effort(...)
  temperature: effective_temperature_for_model(model, 0.0)
  reasoning_effort: resolve_extraction_reasoning_effort(...)
```

Call sites: `extractor/llm.rs`, `extractor/gleaning.rs`, `extractor/sota.rs`.

### Other product temperature sites

| Role | Path | Behavior today |
|------|------|----------------|
| Title gen | `edgequake-api/.../title_generator.rs` | `effective_temperature(model, 0.3)` |
| Query answer | `edgequake-query/.../prompt.rs` | temperature **unset** (Default) |
| Keywords | `keywords/llm_extractor.rs` | unset |
| VLM figure filter | `edgequake-pdf/.../figure_filter.rs` | **hardcoded `Some(0.0)`** — bypasses gate |
| Chat API request | `handlers/chat_types.rs` | client may pass temperature |
| Core Config | `edgequake-core/src/config.rs` | struct default `0.0` (not extract gate) |

### Wire serialization (edgequake-llm)

```ascii
  OpenAIProvider (async-openai)
    if Some(temp) && |temp-1.0| > eps → request.temperature(temp)
    // Does NOT model-match omit; relies on caller None

  OpenAICompatibleProvider (reqwest)
    ChatRequest.temperature: Option<f32>  // skip_serializing_if None
    assigns options.temperature as-is

  AzureOpenAIProvider
    same ≈1.0 skip as OpenAI

  Other providers
    pass Option through; no gpt-5 special case
```

**No env read for temperature omit anywhere.**

### Reasoning effort omit (existing pattern to mirror)

[`edgequake-llm/src/reasoning_capabilities.rs`](../../../edgequake-llm/src/reasoning_capabilities.rs) — Mistral Large / magistra / codestral → `None` (API 3051).

Product env hierarchy: SPEC-109 / `edgequake-core/src/llm_roles.rs` (`EDGEQUAKE_*_REASONING_EFFORT`). **No** `OMIT_REASONING_EFFORT` env today.

## Axis B — Transport today

```ascii
  LLMProvider::chat / stream / chat_with_tools
       │
       ├─ OpenAIProvider ──────────► POST {base}/chat/completions
       ├─ OpenAICompatibleProvider ► POST {base}/chat/completions
       ├─ AzureOpenAIProvider ─────► deployment .../chat/completions
       ├─ Ollama ──────────────────► /api/chat (native)
       ├─ Anthropic / Gemini / Bedrock Converse — native
       └─ VsCodeCopilot ───────────► chat/completions;
                                     SKIP models that only advertise /responses
```

**Grep result:** no client builder for `/v1/responses`. Copilot Auto **avoids** Responses-only models instead of speaking Responses.

Product server route `/api/v1/chat/completions` is a **query facade**, not the upstream transport (LAW-131-9).

## Failure classification today

[`edgequake/crates/edgequake-tasks/src/ingestion_reliability.rs`](../../edgequake/crates/edgequake-tasks/src/ingestion_reliability.rs)

```ascii
  classify_ingestion_failure(msg)
       │
       ├─ timeout / cancel / misconfig / circuit / size / embed / merge / unavailable
       └─ else → Unknown
```

Temperature `unsupported_value` strings → **`Unknown`** → recommended_action `"retry"` (wrong; retry will fail again). Not marked permanent.

## Env surface today (relevant)

Documented in `.env.example` / AGENTS.md:

| Present | Missing (SPEC-131) |
|---------|---------------------|
| `EDGEQUAKE_LLM_PROVIDER` / `_MODEL` | `EDGEQUAKE_LLM_OMIT_TEMPERATURE` |
| `OPENAI_BASE_URL` / `OPENAI_API_KEY` | `EDGEQUAKE_LLM_OMIT_REASONING_EFFORT` |
| `EDGEQUAKE_*_REASONING_EFFORT` | `EDGEQUAKE_LLM_API_FORMAT` |
| `EDGEQUAKE_LLM_CACHE` / prompt cache | |

## Call graph (extract failure path)

```ascii
  Document ingest
       │
       ▼
  Entity extraction (pipeline)
       │  extraction_completion_options_with_effort
       │  temperature = Some(0.0) for gemma-4
       ▼
  LLMProvider::chat(messages, options)
       │
       ▼
  OpenAICompatible / OpenAI  serialize temperature:0
       │
       ▼
  Mantle 400 unsupported_value
       │
       ▼
  PipelineError → classify → failure_class=unknown
       │
       ▼
  Doc terminal Failed (batch × N)
```

## Analytical repro checklist (no AWS)

1. Unit: `effective_temperature_for_model("google.gemma-4-31b", 0.0) == Some(0.0)` (today).
2. Unit: same for `xai.grok-4.3`.
3. Wiremock openai_compatible: body contains `"temperature":0` when options `Some(0.0)`.
4. Classifier unit: paste issue error string → `Unknown` (today).

Target after fix: (1) with omit-env → `None`; (3) body lacks key; (4) → `llm_unsupported_param`.

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Laws: [01-first-principles.md](01-first-principles.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
- Kin: [../109-configurable-reasoning-effort/](../109-configurable-reasoning-effort/)
