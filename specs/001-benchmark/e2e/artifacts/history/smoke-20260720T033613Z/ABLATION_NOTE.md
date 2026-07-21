# Ablation — T0b_p2b_l2_bm25_union_v1

**Step:** t0b (CE-first union in binary at run time)  
**Archive:** `smoke-20260720T033613Z`

| Gate | Result |
|------|--------|
| Fact ER | **0.85 flat** (CE-first buried BM25 gold) |
| ctx_rel | **0.50** |
| Acc | 0.724 < 0.736 |
| recall | 0.926 < LR−0.03 |

**Verdict:** no promote → **T0c** `L2_BM25_MODE=replace` (+ BM25-first union default in code).
