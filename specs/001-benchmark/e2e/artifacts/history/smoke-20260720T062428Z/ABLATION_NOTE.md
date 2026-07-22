# Ablation — A3_p2b_rr_cer_fact_bias_lr_prompt_v1

**Step:** a3  
**Pins:** 028 A3: P2b+rr_cer+fact_bias + ANSWER_PROMPT=lightrag  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | 0.739 | 0.774 | −0.035 (CI includes 0) |
| Complex Acc | 0.739 | 0.829 | −0.090 |
| Fact Acc | 0.723 | 0.772 | −0.049 |
| ctx_rel | **0.519** | 0.544 | −0.025 |
| evidence_recall | 0.921 | 0.988 | −0.066 |

## Gates

- Acc ≥ 0.736: **PASS** (0.739)
- ctx_rel ≥ 0.50: **PASS** (0.519)
- recall ≥ LR−0.03: **FAIL** (0.921 vs need ≥0.958)
- Complex Δ ≤ 0.05: **FAIL** (−0.090)

## Verdict

**No promote.** LR answer prompt helps L2 ctx vs A1/A2 and recovers Acc vs A2, but Complex regresses vs A1. Best Acc pack remains A1 (`rr_cer` only).
