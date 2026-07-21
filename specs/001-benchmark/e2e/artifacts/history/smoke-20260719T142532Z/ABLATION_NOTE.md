# Ablation: cosine + PathRAG + DashScope qwen3-rerank CE

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T142532Z`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79` (query-only)

**Server pins:**
- `EDGEQUAKE_MIX_RELEVANCY_PRUNE=1` · `SCORE=cosine` · `KEEP=12` · `MIN_KEEP=8` · `FLOOR=0.25`
- `EDGEQUAKE_RERANKER=cross_encoder` · `PROVIDER=aliyun` · `MODEL=qwen3-rerank` (DashScope **intl**)
- `EDGEQUAKE_PATH_PRUNE_FRACTION=0.6` · `ORPHAN_ENTITIES=1` · `ENTITY_MIN_KEEP=4`
- `BENCH001_EQ_RERANK_TOP_K=12`

## Results vs prior archives

| Metric | Baseline off `T124903Z` | Cosine `T140420Z` | **CE+PathRAG (this)** |
|--------|-------------------------|-------------------|------------------------|
| EQ Acc | **0.765** | 0.722 | 0.704 |
| EQ ctx_rel | 0.375 | 0.456 | **0.531** |
| EQ recall | 0.928 | **0.950** | 0.898 |
| EQ p50 | ~9.6 s | ~7.7 s | ~11.6 s |
| LR ctx_rel | 0.544 | — | 0.525 |

**Verdict:** First config to clear **S1 ctx_rel ≥ 0.50** (and edges LR on L2 relevancy this run). Acc drop (−0.061 vs baseline) and recall drop (−0.030) **miss** the companion budgets (≤0.02 / ≤0.03). Acc headline stays prune **OFF** until Acc/recall recover under a softer CE/path keep.

**Next:** Soften path (`FRACTION=0.4`, orphan off) and/or raise `rerank_top_k` while keeping CE + cosine.
