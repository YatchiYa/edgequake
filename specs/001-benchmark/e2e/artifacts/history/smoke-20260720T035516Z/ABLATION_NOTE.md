# Ablation — T0d_p2b_l2_bm25_fact_replace_v1 (LLM factual only)

**Archive:** `smoke-20260720T035516Z`  
**Pins:** P2b + `L2_BM25_UNION=1` + `L2_BM25_MODE=fact_replace` (no heuristic OR)

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Fact ER | ≥0.90 | **0.95** |
| ctx_rel | ≥0.50 | **0.5125** |
| recall | ≥ LR−0.03 (0.9408) | **0.9386** (−0.002) |
| Acc floor | ≥0.736 | **0.7225** |
| Δ Acc CI | Beat EQ / Parity includes 0 | **[-0.150, -0.006] LR** |

## Verdict

- [x] Gate missed — **do not promote**
- Best labeled **L2** pack so far (Fact+ctx); Acc peer remains Q4 P2b
