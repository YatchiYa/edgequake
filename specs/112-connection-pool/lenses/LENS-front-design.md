# LENS — Front Design (SPEC-112)

## Scope

Operator-facing clarity on **existing** health/ready/settings documentation surfaces — not a landing-page redesign.

## Design constraints (repo UI system)

- Prefer extending current health/ready JSON consumers and ops docs.
- No new card grids or hero chrome for connection stats.
- If WebUI already shows health components, add a **compact** pool row:

```text
  DB pools
  query   3 idle / 5 size / 8 max
  ingest  0 idle / 6 size / 6 max   ← saturated cue
  queue   2 idle / 2 size / 2 max
  admin   1 idle / 1 size / 1 max
```

## Visual hierarchy

1. Budget status (OK / WARN / FAIL) — single signal  
2. Saturated role (if any)  
3. Link/text to runbook formula  

Avoid pill clusters of unrelated metrics on the same row.

## Motion

If any motion is added: one subtle pulse on saturated role only — not ambient glow on the whole page.

## Accessibility

- Do not encode saturation by color alone; include text (`saturated`, `warn`).
- Keep monospace for env var names in docs UI if present.

## Deliverable for Wave D

Design note only until API fields exist: consume `{role, max, size, idle}` from ready/health — no mock metrics invented in the client.
