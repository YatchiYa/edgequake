# SPEC-069 — Mid-scale concurrent unlock (dedicated hot-set)

**Status:** Complete (2026-07-18) — **no 150k promote**; **SPEC-070 opened**  
**Depends on:** SPEC-068 (honest mid-scale wall), SPEC-067 (planner bias), SPEC-065 (dedicated vs partial)  
**Goal:** Prove **150k full gate** on **dedicated per-workspace vector tables** (table = hot-set). Open DiskANN only if that still fails at clients=16.

**Outcome:** Dedicated HNSW recall-green through 200k but concurrent absolute fails from 100k@clients=16 (~3 s). Shared+partial remains the 100k supported shape.

## Deliverables

| Item | Path |
|------|------|
| Index | this file |
| First principles | [`001-first-principles.md`](001-first-principles.md) |
| Artifacts | [`e2e/artifacts/`](e2e/artifacts/) |
| SSOT | [`docs/product-limits.md`](../../docs/product-limits.md) |

## Commands

```bash
make dedicated-midscale
# Optional: EQ_DEDICATED_ROWS_LIST=100000,150000 make dedicated-midscale
make product-limits-check
```

## Locked decisions

- Promote only from full gate at **clients=16**
- Contention matrix (clients 4/8/16) is diagnostic only
- No silent halfvec / ef / m defaults
- DiskANN deferred unless exit criteria in RUN_NOTES fire → SPEC-070

## Checklist

- [x] Pack + first principles
- [x] Dedicated ladder harness + artifacts
- [x] Contention matrix on first fail rung (clients 4/8/16 × scan_mem)
- [x] Honest wall — no 150k promote; SPEC-070 opened
- [x] Mix≪ANN honesty; no Louvain raise; DiskANN study stub only
