# Ablation: query-embed cosine relevancy prune

**Date:** 2026-07-19  
**Server pins (authoritative):** `EDGEQUAKE_MIX_RELEVANCY_PRUNE=1` · `SCORE=cosine` · `KEEP=12` · `MIN_KEEP=8` · `FLOOR=0.25` · `GRAPH_SOFT_PRUNE=0`  
**Note:** Client `scorecard.pins.mix_relevancy_*` may show prune=false/rrf — publication env overwrite was fixed after this run; trust `/tmp/edgequake-start.sh` + this note.

## Results vs baseline `smoke-20260719T124903Z` (prune off)

| Metric | Baseline | Cosine (this run) | RRF keep=10 (`T134809Z`) |
|--------|----------|-------------------|--------------------------|
| EQ Acc | 0.765 | **0.722** | 0.706 |
| EQ ctx_rel | 0.375 | **0.456** | 0.438 |
| EQ recall | 0.928 | **0.950** | 0.884 |
| EQ p50 | ~9.6 s | ~7.7 s | ~4.7 s |

**Verdict:** Cosine beats RRF-score prune on L2 (ctx_rel + recall) and Acc vs keep=10, but still **below** S1 gate ctx_rel ≥ 0.50 and Acc below baseline. Acc headline stays prune **OFF**. Next: cross-encoder / IG prune or PathRAG path keep.
