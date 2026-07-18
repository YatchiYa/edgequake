# SPEC-072 — DiskANN recall Pareto @150k

**Status:** Measured — **promote opt-in DiskANN @150k** (not silent default)  
**Depends on:** SPEC-070 (concurrent green; recall wall at default q_list)  
**Goal:** Tune `query_search_list_size` / build params to clear full-gate @150k.

## Commands

```bash
make postgres-image-build-pg18-vectorscale   # once
make diskann-recall-pareto                   # primary @150k
EQ_DISKANN_SMOKE=1 make diskann-recall-pareto
```

## Result (2026-07-18)

| Cell | Outcome |
|------|---------|
| 150k q_list=100/200 | recall fail (0.65 / 0.97) |
| **150k q_list≥400** | **full-gate green** |
| 250k spot q_list=800 | full-gate green |
| Rebuild arm | Not required |

**SSOT:** Opt-in DiskANN dedicated recipe → `highest_green_N=150000`. Wave-2 shared+partial stays default 100k. No silent vectorscale flip.

## Checklist

- [x] Pack + harness + make
- [x] Run / archive @150k (query ± rebuild)
- [x] SSOT promote (opt-in) + check tokens

Artifacts: [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md)
