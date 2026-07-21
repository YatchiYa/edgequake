# Ablation: CE Acc recovery — protect inclusion (CE order)

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T151125Z`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79` (query-only)

## Winning pins

```text
EDGEQUAKE_MIX_RELEVANCY_PRUNE=0
EDGEQUAKE_RERANKER=cross_encoder
EDGEQUAKE_RERANKER_PROVIDER=aliyun
EDGEQUAKE_RERANKER_MODEL=qwen3-rerank          # DashScope intl
EDGEQUAKE_PATH_PRUNE=0                         # or FRACTION=0
EDGEQUAKE_RERANK_PROTECT_FIRST=12              # guarantee Mix top-12 in set; CE order kept
BENCH001_EQ_RERANK_TOP_K=30
```

## Results vs S1 budgets (baseline `T124903Z`)

| Metric | Baseline | Soft CE+cosine `T142841Z` | Pure CE path-off `T145634Z` | **Protect CE (this)** | Budget |
|--------|----------|---------------------------|-----------------------------|------------------------|--------|
| Acc | 0.765 | 0.696 | 0.709 | **0.760** | drop ≤0.02 ✅ (−0.004) |
| ctx_rel | 0.375 | 0.544 | 0.525 | **0.519** | ≥0.50 ✅ |
| recall | 0.928 | 0.911 | 0.936 | **0.928** | drop ≤0.03 ✅ |

**By-type F1:** Fact 0.633 · Complex 0.681 · Summarize 0.772 · Creative 0.691 (Complex/Summarize recovered vs pure CE).

**Verdict:** **Phase 1 S1 gate green** for this labeled package. Mechanism: CE reorders for relevancy; `PROTECT_FIRST=12` force-includes Mix RRF top-12 without putting them ahead of CE order in the prompt. Acc headline defaults remain BM25/`PRUNE=0` until an explicit promotion (Phase 2 CI under these pins).
