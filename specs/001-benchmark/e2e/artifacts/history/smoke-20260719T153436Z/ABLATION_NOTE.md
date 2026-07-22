# Acc-win E1 — soft path 0.4 + CE protect (S1 base)

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T153436Z`  
**Confound vs S1:** `PATH_PRUNE_FRACTION=0.4` (was off / 0)  
**Pins:** CE `qwen3-rerank` · `PROTECT_FIRST=12` · path **0.4** · prune off · `top_k=30`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Results

| Metric | S1 discovery `T151125Z` | **E1 this run** | Gate |
|--------|-------------------------|-----------------|------|
| EQ Acc | 0.760 | **0.742** | drop −0.018 ≤0.02 ✅ |
| LR Acc | 0.780 | 0.774 | — |
| Δ Acc 95% CI | [−0.106, +0.061] | [−0.129, +0.064] | includes 0 |
| EQ ctx_rel | 0.519 | **0.519** | ≥0.50 ✅ |
| EQ recall | 0.928 | 0.928 | flat ✅ |
| Complex Acc (EQ→LR) | 0.752→0.835 | 0.757→0.836 | still −8pp vs LR |

## Verdict

1. **L2 stabilize:** ctx_rel **0.519** clears ≥0.50 (same band as S1 discovery; better than Phase 2 confirm 0.481).
2. **Acc budget:** drop vs S1 discovery within 0.02; still statistical tie vs LR.
3. **Complex:** Acc gap vs LR unchanged (~−8pp) — path soft-prune alone does not close packing gap → proceed to **E2** query-conditioned entity ranking (path off for one-confound isolation).

See [017](../../../../001-edgquake-improvements/017-beat-lightrag.md).
