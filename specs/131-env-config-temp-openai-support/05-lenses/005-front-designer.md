# Lens 005 — Front Designer

## Stake

Visual system stays inside existing Documents / Settings patterns. SPEC-131 is mostly backend + docs; front work is **failure presentation** and optional **read-only config chip**.

## Composition rules

- Do not invent a new “LLM Advanced” marketing page for P0.
- Failure banner uses existing danger/status tokens (SPEC-045 / Documents failed states).
- If P2 adds Effective LLM wire card: one section, one headline, short supporting sentence, key/value rows — **no** dashboard of gauges.

## Failure chip (Documents detail)

```ascii
  ┌──────────────────────────────────────────────┐
  │  Failed · LLM parameters unsupported         │
  │  Set OMIT_TEMPERATURE or API_FORMAT=responses│
  │  [Setup guide]            [Technical details]│
  └──────────────────────────────────────────────┘
```

Map `failure_class`:

| Class | Chip label |
|-------|------------|
| `llm_unsupported_param` | LLM parameters unsupported |
| (existing classes) | unchanged |

## Settings (P2 optional)

Read-only rows when server exposes effective config:

| Label | Value source |
|-------|--------------|
| API format | `EDGEQUAKE_LLM_API_FORMAT` |
| Omit temperature | bool |
| Omit reasoning effort | bool |

No edit controls until product decides workspace-scoped overrides (out of v1).

## Non-goals

- Purple gradient “AI transport” hero
- Floating badges on query composer
- New icons for Responses vs Chat

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
