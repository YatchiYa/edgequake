# 06 — UX / UI spec

## Scope

Presentation of typed fleet mirror near-miss failures on Documents detail.
No new routes.

## States

| State | Status chip | Banner | Primary CTA |
|-------|-------------|--------|-------------|
| S0 Processing | Processing | none / progress | — |
| S1 Failed (spine `0/N`) | Failed | SPEC-091 + spine hint | Reprocess |
| S2 Failed (near-miss arrow) | Failed | SPEC-091 + miss samples (+ optional parse hint) | Reprocess |
| S3 Completed after fix | Completed / Ready | none | — |

## Copy rules

1. Do not map this failure to “Retryable Database” in the classifier (already `GraphMerge` permanent).
2. Miss sample list is first-class support evidence — keep it copyable.
3. Avoid blaming the operator for “corrupt DB” when samples show `->` inside names.

## ASCII wireframe

```ascii
  EdgeQuake > Documents > 0001_Note_manuscrite.pdf

  0001_Note_manuscrite.pdf          [ Failed ! ]  ⬇  ⌬  [ Reprocess ]

  ┌──────────────────────────────────────────────────────────────┐
  │ Knowledge graph persist failed: … resolved 995/1000 …        │
  │ SPEC-098 misses:                                             │
  │   FLOW_DIRECTION->ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET):…   │
  │   …                                                          │
  └──────────────────────────────────────────────────────────────┘
```

## Cross-refs

- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md), [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
