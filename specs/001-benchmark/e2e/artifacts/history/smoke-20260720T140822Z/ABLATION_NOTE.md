# Ablation — B6 + a1fp (049 REL_DEDUP source-chunk union)

**Step:** B6 re-ingest → `a1fp`  
**WS:** `58ffe7da-d181-4a31-8941-9621b051a678`  
**Archive:** `smoke-20260720T140822Z`  
**Ingest audit:** `20260720T140630Z` (ge2_rate **0.1247**)

## Gates

| Gate | Target | Result |
|------|--------|--------|
| eq_edges_ge2_rate | ≥0.05 | **PASS 0.1247** |
| Acc | ≥0.781 | **FAIL 0.725** |
| Fact ER | ≥0.83 | **PASS 0.85** |
| ctx_rel | ≥0.50 | **PASS 0.506** |

## Verdict

- [x] Structural law met — keep merger fix
- [x] Acc peer gate missed — **do not promote**; restore B5 Acc peer `8e990410-…` / Acc **0.801**
