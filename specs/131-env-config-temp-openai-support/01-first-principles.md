# 01 — First Principles (LAW-131)

> **Cross-refs:** [WHY](00-why.md) · [Code as-is](03-code-as-is.md) · [Target](04-target-architecture.md) · [SPEC-109](../109-configurable-reasoning-effort/) · [SPEC-045](../045-fix-ingestion-errors/)

## Domain

EdgeQuake’s product LLM surface is **transport-agnostic intent** (`CompletionOptions` / `LLMResponse`). Providers translate intent to a **wire format**. Two wire concerns collide in #379:

```ascii
  Plane A — Parameter legality
            temperature / reasoning_effort / top_p …
            Model ∩ Provider ∩ Format decide which fields are legal

  Plane B — Transport endpoint
            Chat Completions  vs  Responses
            Same intent, different JSON + URL
```

Hardcoding Plane A exceptions as model substrings is an **open catalog**. Binding Plane B to one endpoint is a **capability cliff**.

## Axioms

1. **Unsupported wire values are defects.** Sending an illegal `temperature` yields HTTP 400 — worse than omitting the field (same law as SPEC-109 LAW-R3 for effort).
2. **Operator policy outranks heuristics.** Env omit must supersede the model substring gate so day-one models work without a release.
3. **Transport is configuration, not model identity.** `EDGEQUAKE_LLM_MODEL` names a model; `EDGEQUAKE_LLM_API_FORMAT` names the HTTP shape.
4. **Product types stay format-agnostic.** Pipeline/query never import Responses request structs.
5. **Omit is not “temperature=1”.** Omitting the field lets the provider apply its default; sending `1.0` is still an explicit override on many gateways.
6. **Stateful Responses defaults are a privacy hazard.** Bedrock Mantle defaults `store=true` (30-day retention). EdgeQuake ingest/query must send `store:false`.

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| **LAW-131-1** | **Omit > illegal send** — Never emit a sampling/effort field the operator or capability policy says to omit | Avoid 400 / mass ingest failure |
| **LAW-131-2** | **Env omit supersedes model gate** — `EDGEQUAKE_LLM_OMIT_TEMPERATURE` / `OMIT_REASONING_EFFORT` force `None` even when the substring gate would allow `Some` | Day-one operator escape hatch |
| **LAW-131-3** | **One temperature resolver SSOT** — All product call sites use `resolve_effective_temperature(model, preferred)`; no raw `Some(0.0)` for LLM calls | DRY; closes VLM bypass |
| **LAW-131-4** | **Wire defense in depth** — OpenAI + OpenAI-compatible providers strip `temperature` when omit-env is set even if options still carry `Some` | Survivable against missed call sites |
| **LAW-131-5** | **Transport is env/config** — `EDGEQUAKE_LLM_API_FORMAT=chat_completions\|responses`; default `chat_completions` | Backward compatible |
| **LAW-131-6** | **Same product types both formats** — Mapper in `edgequake-llm` only; returns `LLMResponse` / `StreamChunk` | Liskov; pipeline unchanged |
| **LAW-131-7** | **`store:false` always (v1)** — EdgeQuake upstream Responses never rely on server-stored conversation state | Privacy + Bedrock retention |
| **LAW-131-8** | **Classify unsupported params** — `unsupported_value` / temperature param errors → `llm_unsupported_param` (not `unknown`) | SPEC-045 triage honesty |
| **LAW-131-9** | **Server facade unchanged** — EdgeQuake `/api/v1/chat/completions` remains the product chat API; this spec is **upstream** only | Issue non-goal |
| **LAW-131-10** | **Unfakable proof** — wiremock body asserts + classifier unit + source contract; live Mantle gated | Honest acceptance |
| **LAW-131-11** | **Heuristic gate remains fallback** — Keep `model_requires_default_temperature` for known OpenAI families when env unset | No regression for gpt-5 / o* |
| **LAW-131-12** | **Responses v1 scope** — OpenAI + OpenAI-compatible; chat/stream/json_schema; no hosted tools / Conversations / `previous_response_id` | Ship P1 without agentic sprawl |

## Causal diagram

```ascii
  preferred_temperature (0.0 extract / 0.3 title / …)
           │
           ▼
  resolve_effective_temperature(model, preferred)
           │
           ├─ OMIT_TEMPERATURE env? ──yes──► None
           ├─ model_requires_default? ─yes──► None
           └─ else ─────────────────────────► Some(preferred)
           │
           ▼
  CompletionOptions.temperature: Option<f32>
           │
           ▼
  Provider serialize
           │
           ├─ ApiFormat::ChatCompletions ──► /chat/completions
           │     (+ wire strip if omit-env)
           └─ ApiFormat::Responses ────────► /responses
                 store:false
                 temperature omitted if None
```

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | Resolver owns omit policy; mapper owns Responses JSON; classifier owns triage tokens |
| **O** | New reject-temperature models → env, not new `contains("…")` rows |
| **L** | Both formats honor `LLMProvider` contract |
| **I** | Small knobs: temperature resolve, effort omit, `ApiFormat` — no mega LLM-policy facade |
| **D** | Pipeline depends on `CompletionOptions`, not OpenAI Responses crates |
| **DRY** | One resolver; one Responses mapper shared by OpenAI + openai_compatible |

## Normative env semantics

| Variable | Values | Default | Effect |
|----------|--------|---------|--------|
| `EDGEQUAKE_LLM_OMIT_TEMPERATURE` | `1`/`true`/`yes` (case-insensitive) | unset/false | Force temperature omit (LAW-131-2/4) |
| `EDGEQUAKE_LLM_OMIT_REASONING_EFFORT` | same | unset/false | Force `reasoning_effort=None` after role resolve |
| `EDGEQUAKE_LLM_API_FORMAT` | `chat_completions` \| `responses` | `chat_completions` | Upstream transport (LAW-131-5) |

Invalid `API_FORMAT` → fail loud at provider factory / server boot (do not silently fall back).

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Reasoning kinship: [../109-configurable-reasoning-effort/01-first-principles.md](../109-configurable-reasoning-effort/01-first-principles.md)
- Failure taxonomy: [../045-fix-ingestion-errors/](../045-fix-ingestion-errors/)
