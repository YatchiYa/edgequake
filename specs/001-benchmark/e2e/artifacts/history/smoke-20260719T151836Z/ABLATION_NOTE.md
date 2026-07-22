# Phase 2 confirmatory Acc+CI (S1 package pins)

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T151836Z`  
**Pins:** CE `qwen3-rerank` · `PROTECT_FIRST=12` · path off · `top_k=30` · prune off  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Results

| Metric | S1 discovery `T151125Z` | **This confirmatory** | Baseline `T124903Z` |
|--------|-------------------------|----------------------|---------------------|
| EQ Acc | 0.760 | **0.751** | 0.765 |
| LR Acc | 0.780 | **0.771** | 0.754 |
| Δ Acc (EQ−LR) | −0.020 | **−0.020** | +0.011 |
| Δ Acc 95% CI | [−0.106, +0.061] | **[−0.112, +0.069]** | (includes 0) |
| EQ ctx_rel | **0.519** | 0.481 | 0.375 |
| EQ recall | 0.928 | 0.911 | 0.928 |

**CI excludes 0?** No (both S1-pin Acc runs).

## Phase 2 verdict

1. **Acc:** Persistent **statistical tie** under S1 pins (Δ CI includes 0). Point estimate favors LR (~−2pp); same honesty class as baseline Acc tie.
2. **L2:** Discovery run cleared ctx_rel ≥ 0.50; confirmatory dipped to **0.481** (still +10pp vs baseline 0.375, still below LR). **Not stable enough for silent headline promotion.**
3. **Promotion:** Keep Acc defaults BM25 / `PRUNE=0`. Treat CE+protect as **labeled profile** until L2 ≥0.50 replicates or Acc CI win.

See [020 §2b](../../../../001-edgquake-improvements/020-roadmap.md).
