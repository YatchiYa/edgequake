# SPEC-076 — Precision layers (exact reorder + sparse RRF)

**Status:** Complete (2026-07-18)  
**Depends on:** SPEC-073 §006 (A3/A4), SPEC-074, SPEC-075  
**Goal:** Opt-in ANN→exact reorder (A3) + measured sparse FTS+ANN RRF tip (A4) — no floor raise, no silent flips.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default path | Wave-2 shared+partial @100k unchanged |
| A3 exact reorder | **Opt-in OFF** (`EDGEQUAKE_ANN_EXACT_REORDER=0`) |
| A4 sparse RRF | Tip via `EDGEQUAKE_SPARSE_FUSION=rrf`; default stays sparse-first weighted |
| Floors | No raise; Mix/RRF ≠ ANN floor |
| Promote metric | Filtered recall@20 (SPEC-075) |

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | Two-stage ANN precision; FTS fills token miss |
| [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md) | Bake-off / smoke archive |

## Commands

```bash
# A3 contract (no DB)
cargo test -p edgequake-storage --features postgres --test contract_spec076_ann_exact_reorder

# A4 sparse RRF tip contract
cargo test -p edgequake-query --test contract_spec076_sparse_rrf_tip

# Gate (contracts + optional smoke)
make precision-layers-gate

make product-limits-check
```

## Checklist

- [x] Pack + first principles
- [x] A3 opt-in exact reorder + contract/e2e
- [x] A4 sparse RRF tip bake-off + content_tsv honesty
- [x] product-limits-check green; SPEC-073 §006 A3/A4 done

## Out of scope

Binary quantize (B2), Matryoshka (A5), Filtered-DiskANN labels (A6), schema unify, floor raises.
