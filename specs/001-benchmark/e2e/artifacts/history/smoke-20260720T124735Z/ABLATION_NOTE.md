# Ablation — 046 a1fpspec answer specificity

**Archive:** `smoke-20260720T124735Z`  
**Workspace:** `8e990410-…` (B5 peer, query-only)  
**Profile:** `A1FPSPEC_…_answer_specific_v1`  
**Confound:** `ANSWER_PROMPT=specific` on `a1fp` (no TOPIC_*)

## vs peer B5+a1fp [`T120315Z`](../smoke-20260720T120315Z/)

| Metric | Peer | a1fpspec | Gate |
|--------|-----:|---------:|:-----|
| Acc | **0.801** | 0.746 | ✗ ≥0.781 |
| Complex Acc | 0.813 | 0.783 | — |
| Complex Δ vs LR | −0.050 | **−0.014** | ✓ ≤0.03 |
| Fact ER | **0.85** | 0.80 | ✗ ≥0.83 |
| ctx | **0.519** | 0.481 | ✗ ≥0.50 |
| PARP names (`Medical-54a3a465`) | generic class | **olaparib/niraparib/rucaparib** | specificity ✓ |

Δ Acc vs LR **−0.024** · CI **[-0.115, +0.061]** — no Beat.

## Verdict

**REJECT** Acc pin — specificity instruction closes Complex Δ vs LR and names drugs, but Acc/Fact/ctx tax vs peer.

Keep **B5+a1fp**. Do not stack with TOPIC_* or A3 `lightrag` abstain prompt.
