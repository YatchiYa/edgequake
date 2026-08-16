# 06 — UX / UI specification

> Lenses: [004-ux-ui](05-lenses/004-ux-ui.md) · [005-front-designer](05-lenses/005-front-designer.md)

## Scope

| Priority | Surface | Work |
|----------|---------|------|
| P0 | Docs / setup guide / `.env.example` | Three env knobs + Mantle recipes |
| P0 | Failed document metadata | Surface new `failure_class` + recommended_action |
| P1 | Query streaming | No UX change if `StreamChunk` mapping preserves tokens |
| P2 | Settings read-only effective LLM wire | Optional |

## Failed document contract

When classify returns `llm_unsupported_param`:

| Field | Value |
|-------|-------|
| `failure_class` | `llm_unsupported_param` |
| `recommended_action` | `omit_llm_temperature_or_switch_api_format` |
| UI primary message | LLM rejected request parameters |
| UI secondary | Point to omit-temperature / API format docs |
| Retry affordance | Manual only after config change (permanent class) |

Wire existing Documents detail failure renderer to the new class (same pattern as `provider_misconfigured`).

## Setup documentation blocks (must ship)

### Block A — Omit temperature (Chat Completions)

```bash
EDGEQUAKE_LLM_OMIT_TEMPERATURE=true
# optional
EDGEQUAKE_LLM_OMIT_REASONING_EFFORT=true
```

### Block B — Responses format

```bash
EDGEQUAKE_LLM_API_FORMAT=responses
EDGEQUAKE_LLM_OMIT_TEMPERATURE=true
```

### Block C — What we do not ask users to do

- Patch `temperature.rs` for each new model
- Change EdgeQuake’s public `/api/v1/chat/completions` path

## Empty / success states

No new empty states. Success = ingest Completes without temperature 400; query answers stream as today.

## Cross-refs

- Acceptance: [09-acceptance.md](09-acceptance.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
