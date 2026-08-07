# SPEC-109 — First Principles

> **Cross-refs**: [WHY](00-why.md) · [Use cases](02-use-cases-and-surfaces.md) · [Capability matrix](03-provider-capability-matrix.md) · [Roadmap](06-implementation-roadmap.md)

## Axioms

1. **Completion budget is finite.** Reasoning tokens and output tokens compete inside the same completion / max-output envelope on reasoning models.
2. **Effort is a soft steering knob, not a second model.** Same model slug, different token allocation policy.
3. **Unsupported values are defects.** Sending an illegal `reasoning_effort` yields HTTP 400 — worse than omitting the field.
4. **Roles have different Pareto points.** Structured ingest ≠ open-ended RAG answer.

## Causal diagram (token budget)

```text
┌─────────────────────────────────────────────────────────────┐
│  max_completion_tokens / max output budget                  │
│  ┌──────────────────────┐  ┌────────────────────────────┐   │
│  │ reasoning / CoT      │  │ visible + structured out   │   │
│  │ (effort steers size) │  │ (JSON, caption, answer)    │   │
│  └──────────────────────┘  └────────────────────────────┘   │
│         ↑ high effort                  ↑ starved if CoT wins │
└─────────────────────────────────────────────────────────────┘

Desired extract/vlm:  effort = lowest supported  → maximize schema tokens
Desired hard query:   effort = medium|high       → maximize answer quality
Desired default query: omit (Auto)               → provider model default
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-R1** | **Budget law** — On reasoning models, internal reasoning and user-visible output share one completion budget; product must be able to steer the split. |
| **LAW-R2** | **Single wire field** — Product + providers use only `CompletionOptions.reasoning_effort: Option<String>` in v1 (no parallel `thinking` / `budget_tokens` product knobs). |
| **LAW-R3** | **Clamp before send** — Never emit an unsupported effort. Registry returns omit / nearest-lower / mapped value. Illegal sends are release blockers. |
| **LAW-R4** | **Capability SSOT** — Supported efforts, defaults, and clamp live in `edgequake-llm::reasoning_capabilities`. EdgeQuake resolves *desired* effort only. |
| **LAW-R5** | **Role defaults** — `extract` / `summary` / `keyword` / `vlm` default to **lowest supported**; `query` / `chat` default to **omit (Auto)** unless configured. |
| **LAW-R6** | **Hierarchy** — Compiled → env → server → tenant seed → workspace role → request → clamp. Later layers win when set; empty means inherit. |
| **LAW-R7** | **Forward or fail loud in tests** — Every provider that claims reasoning support must serialize the mapped field (native OpenAI included). Dropping the field while options set is a defect (E2E-109-01). |
| **LAW-R8** | **Surface parity** — Anything operators can configure must appear at tenant seed, workspace role, and (where runtime-relevant) request override, plus effective-config explainability. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | Resolver owns desired effort; registry owns legality; provider owns wire shape; UI owns presentation. |
| **O** | New model family = extend registry rows; callers unchanged. |
| **L** | All providers accept the same `CompletionOptions`; unsupported → omit after clamp. |
| **I** | No mega “LLM policy” facade — small `resolve_role_reasoning` + `clamp_reasoning_effort`. |
| **D** | Pipeline depends on resolved `CompletionOptions`, not OpenAI/Mistral request structs. |
| **DRY** | One field, one hierarchy helper, one registry, reuse `LlmRole` / `llm_roles` / `ServerLlmDefaults` / `config_resolution`. |

## Normative resolution sketch

```text
fn resolve_reasoning_effort(role, ws, tenant, server, env, request) -> Option<String>:
  desired = first_some([
    request.override_for(role),
    ws.llm_roles[role].reasoning_effort,
    ws.default_reasoning_effort,
    tenant.default_reasoning_effort_for(role),
    server.reasoning_by_role[role] or server.reasoning_effort,
    env.EDGEQUAKE_{ROLE}_REASONING_EFFORT,
    env.EDGEQUAKE_REASONING_EFFORT,
    compiled_default(role),  # lowest for structured roles; None for query
  ])
  return clamp(provider, model, desired)  # may map none→minimal; may omit
```

## Acceptance pins (must appear in e2e)

| Pin | Rule |
|-----|------|
| OpenAI forward | Native provider sets Chat Completions `reasoning_effort` when `Some` after clamp |
| gpt-5-mini floor | Desired `none` → send `minimal` (never 400) |
| gpt-5.4-mini/nano floor | Desired lowest → `none` allowed |
| Mistral Large | Always omit field |
| Extract default | Unconfigured workspace → lowest supported for extract model |
