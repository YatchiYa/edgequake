# Ablation — A2_p2b_rr_cer_fact_bias_v1

**Step:** a2  
**Pins:** 028 A2: P2b+rr_cer + INTENT_FACTUAL_BIAS=1 (no L2 heuristic-OR)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | 0.714 | 0.772 | −0.058 (CI includes 0) |
| Complex Acc | 0.698 | 0.811 | −0.113 |
| Fact Acc | 0.693 | 0.727 | −0.034 |
| ctx_rel | 0.456 | 0.538 | −0.081 |
| evidence_recall | 0.928 | 0.963 | −0.034 |

## Verdict

**No promote.** Acc tax vs A1 (0.772→0.714); Fact Acc lost lead. Intent bias alone is not the Acc path forward on this pack.
