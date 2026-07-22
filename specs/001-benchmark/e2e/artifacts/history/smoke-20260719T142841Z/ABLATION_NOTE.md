# Ablation: cosine + soft PathRAG + DashScope qwen3-rerank CE

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T142841Z`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79` (query-only)

**Server pins:**
- Cosine prune keep=12 / floor=0.25 (same as `T140420Z`)
- `EDGEQUAKE_RERANKER=cross_encoder` · `qwen3-rerank` (DashScope intl)
- `EDGEQUAKE_PATH_PRUNE_FRACTION=0.4` · `ORPHAN_ENTITIES=0`
- `BENCH001_EQ_RERANK_TOP_K=16`

## Results

| Metric | Baseline `T124903Z` | Aggressive CE+path0.6 `T142532Z` | **Soft CE (this)** |
|--------|---------------------|----------------------------------|--------------------|
| EQ Acc | **0.765** | 0.704 | 0.696 |
| EQ ctx_rel | 0.375 | 0.531 | **0.544** |
| EQ recall | 0.928 | 0.898 | **0.911** |
| EQ p50 | ~9.6 s | ~11.6 s | ~9.8 s |

**Verdict:** Soft path keeps **S1 ctx_rel** (0.544) and brings recall drop vs baseline to **−0.017** (within ≤0.03). Acc remains ~−0.07 vs baseline — CE still taxes generation Acc. Best L2 package so far; Acc headline unchanged until Acc recovers (Phase 2 / keep-m tune).
