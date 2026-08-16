# 04 — Target architecture

> Normative shape for implementation. Laws: [01-first-principles.md](01-first-principles.md).

## Overview

```ascii
  ┌─────────────────────────────────────────────────────────────┐
  │ EdgeQuake product (pipeline / query / api / pdf)            │
  │                                                             │
  │  resolve_effective_temperature(model, preferred)            │
  │  resolve_*_reasoning_effort → then apply OMIT_REASONING     │
  │           │                                                 │
  │           ▼                                                 │
  │  CompletionOptions  (temperature / effort / schema / …)     │
  └───────────────────────────┬─────────────────────────────────┘
                              │
                              ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ edgequake-llm                                               │
  │                                                             │
  │  ApiFormat::from_env()                                      │
  │      chat_completions (default) | responses                 │
  │                                                             │
  │  OpenAIProvider / OpenAICompatibleProvider                   │
  │      ├─ wire_omit_temperature_if_env(options)               │
  │      ├─ ChatCompletionsTransport                            │
  │      └─ ResponsesTransport  ◄── shared mapper (SRP)         │
  │              store: false                                   │
  │              messages→input / format→text.format            │
  │              output→LLMResponse / SSE→StreamChunk           │
  └─────────────────────────────────────────────────────────────┘
```

## Module responsibilities (SOLID)

| Module | Owns | Must not own |
|--------|------|--------------|
| `temperature.rs` (pipeline) | `resolve_effective_temperature` = env ∨ model gate | HTTP / Responses JSON |
| `completion_options.rs` | Extract defaults using resolver | Provider URLs |
| Call sites (title, VLM, …) | Preferred temperature only via resolver | Local omit lists |
| `llm_roles` / effort resolve | Desired effort hierarchy (SPEC-109) | Wire format |
| Omit-effort helper | Env force-None after resolve | Capability registry edits |
| `edgequake-llm::api_format` | Parse `EDGEQUAKE_LLM_API_FORMAT` | Product roles |
| `edgequake-llm::responses_map` | Request/response/SSE mapping | Business extract prompts |
| OpenAI / openai_compatible | Choose transport; wire strip | Model substring catalogs |
| `ingestion_reliability` | `llm_unsupported_param` class | Provider HTTP |

## Temperature resolution (normative)

```rust
// Pseudocode — SSOT
fn resolve_effective_temperature(model: &str, preferred: f32) -> Option<f32> {
    if env_truthy("EDGEQUAKE_LLM_OMIT_TEMPERATURE") {
        return None; // LAW-131-2
    }
    effective_temperature_for_model(model, preferred) // existing gate LAW-131-11
}
```

Wire defense (LAW-131-4):

```rust
fn temperature_for_wire(options: &CompletionOptions) -> Option<f32> {
    if env_truthy("EDGEQUAKE_LLM_OMIT_TEMPERATURE") {
        None
    } else {
        options.temperature.filter(|t| (t - 1.0).abs() > f32::EPSILON) // keep OpenAI ≈1.0 quirk for chat
    }
}
```

For Responses, omit when `None`; do not send `temperature: 1.0` as a “default stand-in.”

## Reasoning omit (normative)

```ascii
  resolve_role_reasoning_effort(...) → Option<String>
       │
       ▼
  if OMIT_REASONING_EFFORT → None
  else → clamped value (SPEC-109)
```

Apply in product builders **and** optionally again at wire (defense) for OpenAI + openai_compatible.

## Responses mapping (v1)

| Chat Completions | Responses |
|------------------|-----------|
| `POST …/chat/completions` | `POST …/responses` |
| `messages[]` | `input` (item array; system → `instructions` or system message item) |
| `max_completion_tokens` / `max_tokens` | `max_output_tokens` |
| `temperature` | `temperature` (omit if None) |
| `reasoning_effort` | `reasoning: { effort }` |
| `response_format` json_object / json_schema | `text.format` |
| `prompt_cache_key` | P1: forward if provider accepts; else document defer (SPEC-126) |
| `choices[0].message.content` | `output_text` / message `output_text` parts |
| SSE token deltas | `response.output_text.delta` → `StreamChunk::Content` |
| — | **`store: false` always** |

Non-goals v1: hosted tools, MCP, `previous_response_id`, Conversations, `background=true`, `n` parallel gens.

## Provider coverage matrix (v1)

| Provider | Chat Completions | Responses (`API_FORMAT=responses`) |
|----------|------------------|-------------------------------------|
| OpenAI | keep | implement |
| OpenAI-compatible (Mantle, gateways) | keep | implement (same mapper) |
| Azure OpenAI | keep | P1.1 if endpoint supports; else error loud |
| Ollama native | N/A | ignore format (native `/api/chat`) |
| Anthropic / Gemini / Bedrock Converse | N/A | ignore format |
| Mock | ignore format; deterministic | ignore |
| VS Code Copilot | chat only today | follow-up: use Responses instead of skip |

## Failure class addition

```ascii
  IngestionFailureClass::LlmUnsupportedParam
       as_str: "llm_unsupported_param"
       recommended_action: "omit_llm_temperature_or_switch_api_format"
       is_permanent: true   // retry without config change will fail
```

Detect markers (case-insensitive): `unsupported_value`, `param: temperature`, `'temperature' does not support`, `does not support temperature`, optionally `unsupported_parameter`.

## Config documentation surfaces

1. `.env.example` — commented examples for Mantle Gemma omit + Responses GPT-5.6
2. AGENTS.md / setup guide — table rows
3. Effective-config / health (if already exposes LLM env): show format + omit flags read-only (see [06-ux-ui-spec.md](06-ux-ui-spec.md))

## Phasing

```ascii
  P0  omit temp + omit effort + classifier + call-site resolver
  P1  Responses transport (OpenAI + openai_compatible) + wiremock
  P2  docs / Acc pins / optional LIVE-131 Mantle
```

## Cross-refs

- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- AI lens: [05-lenses/006-ai-engineer.md](05-lenses/006-ai-engineer.md)
