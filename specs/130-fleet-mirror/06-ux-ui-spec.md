# 06 — UX / UI specification

## Planes

```ascii
  Plane U — Operator UI (Documents list / detail)
  Plane L — API / task logs (miss samples, GraphMerge)
  Plane S — SQL spine (entities, relationships, embeddings)
```

SPEC-130 makes Plane S and Plane U converge for the #380 failure class. Plane L stays diagnostic.

## Rules

| ID | Rule |
|----|------|
| UX-130-1 | Failed chip only when persist truly failed |
| UX-130-2 | Detail error may show truncated miss sample; no raw UUID flood |
| UX-130-3 | Reprocess action remains one-shot; no client retry storm |
| UX-130-4 | No new status slug for “fleet_mirror_pending” |

## Copy

**Pre-fix (misleading):** “…ensure PostgresEntitySink wrote the spine before fleet mirror…”

**Post-fix (target):** “…in-session RelVectors must use sink-returned `relationships.id`; misses: …”

## Cross-refs

- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md), [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
