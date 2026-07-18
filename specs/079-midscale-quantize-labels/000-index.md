# SPEC-079 — Mid-scale B2 + A6 promote-gate archive

**Status:** Complete (archive; Not promoted)  
**Depends on:** SPEC-077 (B2 smoke), SPEC-078 (A6 smoke), SPEC-075 (filtered recall)  
**Goal:** Re-run binary quantize and Filtered-DiskANN labels at **50k / 100k**; archive honesty decision — **no silent flip, no floor raise from smoke-shaped tips alone**.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default path | Wave-2 @100k unchanged |
| Tips | Remain **Not promoted** unless full-gate green at 100k |
| Promote metric | **Filtered** recall@20 vs Wave-2 |
| Silent flip | Forbidden |
| Row ladder | 50 000, 100 000 @ DIM=64 |

## Commands

```bash
make midscale-quantize-labels
# EQ_MIDSCALE_ROWS=50000,100000 EQ_BQ_HANG_MS=30000 EQ_FDL_HANG_MS=30000 make midscale-quantize-labels

make product-limits-check
```

## Checklist

- [x] Pack + first principles
- [x] Runner merges B2 (pg18) + A6 (pg18-vectorscale) @ ladder
- [x] RUN_NOTES decision archived (**Not promoted** — B2 soft-fail @50k/100k; A6 soft-green)
- [x] SSOT tip unchanged; SPEC-073 Phase-3 mid-scale noted

## Out of scope

250k wall, product query wiring, concurrent floor raise, B3/C5 (SPEC-080/081).
