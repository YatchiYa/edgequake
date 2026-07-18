# SPEC-081 — C5 serving view (dual-SSOT assessment)

**Status:** Complete (assessment)  
**Depends on:** SPEC-073 §006 C5 / Phase 4, SPEC-058/059 dual-SSOT  
**Goal:** Read-only admin/debug SQL functions that surface relational chunk presence (+ optional vectors join). **Not** a silent rewrite of stores; **not** the ANN query path.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Surface | `eq_serving_chunk_presence(uuid)` + `eq_serving_vector_presence(uuid, regclass)` |
| Ingest / ANN | Unchanged |
| Promote as SSOT | Forbidden — serving view ≠ RAG corpus |
| Silent schema unify | Forbidden |

## Commands

```bash
make serving-view-check
make product-limits-check
```

## Checklist

- [x] Migration 093
- [x] Contract + runner
- [x] data-layer + SPEC-073 Phase-4 note
