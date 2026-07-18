# SPEC-065 — Product limits SSOT + Wave-2 productization

**Status:** Active  
**Depends on:** SPEC-063 (envelope), SPEC-064 (ANN battle)  
**Goal:** One claim table for operators; opt-in Wave-2 path wired in production (not docs-only).

## Deliverables

| Item | Path |
|------|------|
| Limits SSOT | [`docs/product-limits.md`](../../docs/product-limits.md) |
| Policy object | `HnswRuntimePolicy` in `edgequake-storage` |
| Lifecycle | `PgVectorStorage::ensure_hot_workspace_ann` on filtered query |
| Lifecycle e2e | `e2e_spec065_partial_hnsw_lifecycle.rs` |
| DRY runner | [`scripts/eq_ephemeral_pg.sh`](../../scripts/eq_ephemeral_pg.sh) |
| Honesty gate | `make product-limits-check` |

## Commands

```bash
make product-limits-check
make ann-scale-battle
EQ_PERF_PROFILES=pg18 make data-access-perf-matrix-prod
```

## Checklist

- [x] Flag creates partial HNSW on shared tables (not SQL-only)
- [x] Dedicated `_ws_` tables skip partial
- [x] FAQ cites Wave-2 for 100k Q1-d
- [x] No silent halfvec / GUC default flip
- [x] Lifecycle e2e + battle remasure + pg18 prod matrix (see [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md))
