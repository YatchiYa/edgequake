# Ablation: CE-only Acc recovery (no cosine, top_k=24)

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T145324Z`  
**Hypothesis:** Soft CE Acc drop was double-cut (cosine keep=12 → CE top_k=16) killing Fact evidence.

**Pins:** `PRUNE=0` · `RERANKER=cross_encoder` / `qwen3-rerank` · `PATH_PRUNE_FRACTION=0.4` · `ORPHAN=0` · `rerank_top_k=24`

## Results

| Metric      | Baseline  | Soft CE+cosine `T142841Z` | **CE-only (this)** |
| -------------| -----------| ---------------------------| --------------------|
| EQ Acc      | **0.765** | 0.696                     | **0.710**          |
| EQ ctx_rel  | 0.375     | **0.544**                 | 0.506              |
| EQ recall   | 0.928     | 0.911                     | 0.909              |
| Fact F1     | ~0.690    | 0.407                     | **0.567**          |
| Fact recall | 0.95      | 0.80                      | 0.80               |

**Verdict:** Dropping cosine recovers ~+0.014 Acc and +0.16 Fact F1 vs soft CE; ctx_rel still clears S1 (0.506). Acc budget (−0.054) and Fact recall (0.80) still open → try path-off + top_k=30.
