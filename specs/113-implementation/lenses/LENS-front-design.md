# LENS — Front Design (SPEC-113)

## Design job

Make **capability truth** scannable next to the model picker — without new dashboard chrome or purple AI clichés.

## Composition (settings / query model row)

```text
  ┌─────────────────────────────────────────────────────────────┐
  │  Model   ollama / qwen3-vl:8b                               │
  │          completion · vision                                │  ← caps chips
  │          Thinking unavailable                               │  ← quiet status, not alarm red
  └─────────────────────────────────────────────────────────────┘
```

When thinking is available:

```text
  │          completion · tools · thinking                      │
  │          Thinking  [ Auto ▾ ]                               │
```

## Visual rules (fit existing EdgeQuake WebUI)

- Reuse existing chip / muted meta styles from models catalog — **no new card stack** in the hero of settings.
- Capability chips are **metadata**, not CTAs.
- Disabled effort control: reduce contrast; do not use scary error red for “unsupported.”
- Motion: optional 150–200ms fade when caps resolve (avoid spinner spam on every keystroke — resolve once per model select).

## States to design

| State | Treatment |
|-------|-----------|
| Loading caps | Subtle placeholder “…” under model id |
| Caps loaded | Chips + thinking availability line |
| Probe failed | “Capabilities unavailable” muted; effort hidden |
| Legacy mode (ops) | Tiny “legacy think heuristic” footnote in advanced only |

## Out of scope

- Marketing landing redesign  
- New illustration system for “thinking”
