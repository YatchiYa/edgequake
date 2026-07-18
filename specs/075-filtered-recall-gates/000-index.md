# SPEC-075 — Filtered recall gates + iterative_scan bounds

**Status:** Complete (2026-07-18)  
**Depends on:** SPEC-064/068 (Wave-2), SPEC-073 §006 (A2/B5), SPEC-074 (P0 done)  
**Goal:** Make **filtered** recall@20 a claim discipline for Wave-2, and productize `hnsw.iterative_scan` / `max_scan_tuples` bounds — no floor raise, no silent flips.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default path | Wave-2 shared+partial @100k unchanged |
| Promote metric | **Filtered** recall@20 (workspace filter) — never unfiltered-only |
| iterative_scan | On for filtered queries (`relaxed_order` default); **off** for unfiltered |
| Floors | No raise |

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | Precision law + iterative_scan vs partial |
| [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md) | Smoke gate archive |

## Commands

```bash
make filtered-recall-gate          # smoke N (default 5k)
EQ_FILTERED_RECALL_ROWS=10000 make filtered-recall-gate

# Contract (no DB): iterative_scan GUCs
cargo test -p edgequake-storage --features postgres --test contract_spec075_iterative_scan_bounds

make product-limits-check
```

## Checklist

- [x] Pack + first principles
- [x] `make filtered-recall-gate` + artifacts
- [x] iterative_scan bounds contract + SSOT docs
- [x] product-limits-check green; SPEC-073 §006 link

## Out of scope

Binary quantize, Filtered-DiskANN labels, RRF/exact-reorder (SPEC-076), Louvain/Mix floors.
