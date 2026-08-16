# Lens 006 — AI Engineer

## Problem / constraint

Dense LLM extraction (50–350+ entities, many relationships) amplifies RelVectors mirror surface area. Every extracted edge becomes a legacy vector id that must FK to `relationships`. Caps and gleaning change **how many** rows hit the bug, not whether identity discard is wrong.

## Control loop

```ascii
  Extract (LLM) → unique embed by endpoints → RelGraph sink → RelVectors
       │                                      │
       │ more relations                       └── identity must be retained
       ▼
  EDGEQUAKE_MAX_EXTRACTION_* caps reduce volume (SPEC-117)
  but must not be treated as the SPEC-130 fix
```

## Observability

- Log `resolved/eligible` and miss sample (already).
- After fix: log `map_hits` vs `name_resolve_hits` (name path should be ~0 in-session).
- OTEL ingest stage timings already separate `age_edge_upsert` vs `rel_vector_upsert` — keep.

## Anti-patterns

- Raising extraction caps to “avoid” failures.
- Prompt changes to emit fewer relationships as a substitute for identity pass-through.
- Softening fail-closed so partial graphs look green.

## Eval / acceptance for AI path

- Fixture with known N relationships → N fleet relationship embeddings after merge.
- Regression: arrow-containing entity names still parse (SPEC-091) under UUID map.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- Edges: [../10-edge-cases.md](../10-edge-cases.md)
