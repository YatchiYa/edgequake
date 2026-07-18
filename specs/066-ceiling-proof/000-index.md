# SPEC-066 — Ceiling proof + reliability hardening

**Status:** Active  
**Depends on:** SPEC-063 (ladder), SPEC-064 (Wave-2), SPEC-065 (SSOT + productization)  
**Goal:** Push Wave-2 ANN (and graph G1) until we archive a hard Q1-d fail or promote a new floor; harden residency/admission/readiness so floors stay ops-real.

## Deliverables

| Item | Path |
|------|------|
| First principles | [`001-first-principles.md`](001-first-principles.md) |
| Ceiling runner | [`e2e/run_ceiling_ladder.sh`](e2e/run_ceiling_ladder.sh) |
| ANN e2e | `e2e_spec066_ceiling_ladder_ann.rs` |
| Graph G1 e2e | `e2e_spec066_graph_g1.rs` |
| Artifacts | [`e2e/artifacts/`](e2e/artifacts/) |
| Make | `make ceiling-proof` |

## Commands

```bash
# L2 Wave-2 @500k (default)
make ceiling-proof

# Explicit step
EQ_CEILING_STEP=L2 make ceiling-proof
EQ_CEILING_STEP=L3 make ceiling-proof
EQ_CEILING_STEP=seek EDGEQUAKE_CEILING_ROWS=250000 make ceiling-proof

# Graph G1 (included in runner when EQ_CEILING_INCLUDE_GRAPH=1)
EQ_CEILING_INCLUDE_GRAPH=1 make ceiling-proof
```

## Locked decisions

- Wave-2 only for L2+ (halfvec + partial HNSW + column filter)
- No silent halfvec / GUC default flip
- DiskANN only after archived Wave-2 FORBIDDEN cliff
- Promote floors only via [`docs/product-limits.md`](../../docs/product-limits.md)

## Checklist

- [x] L2 Wave-2 soak archived (green or cliff)
- [x] highest_green_N / first_fail_N in RUN_NOTES
- [x] max_documents admission fail-closed
- [x] Wave-2 readiness probe
- [x] Graph G1 measured
- [x] `make product-limits-check` green
