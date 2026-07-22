# Ablation: CE path-off top_k=30 (Acc recovery R2)

**Archive:** `smoke-20260719T145634Z`  
**Pins:** `PRUNE=0` · CE `qwen3-rerank` · `PATH_PRUNE=0` · `rerank_top_k=30`

| Metric | Baseline | CE-only path0.4 `T145324Z` | **This** |
|--------|----------|----------------------------|----------|
| Acc | 0.765 | 0.710 | 0.709 |
| ctx_rel | 0.375 | 0.506 | **0.525** |
| recall | 0.928 | 0.909 | **0.936** |
| Fact F1 | ~0.690 | 0.567 | **0.705** |

**Verdict:** Path-off recovers Fact F1 and recall; Complex/Summarize F1 still drag Acc (~−0.055). Next: first-stage protect slots under CE.
