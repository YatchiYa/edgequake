# Ablation — LR_NF_FACT_L2_v1 (smoke)

**Step:** lr-nf-fact-l2  
**Pins:** L1.5 + `EDGEQUAKE_RR_ORDER=naive_first`  
**Archive:** `smoke-20260722T125634Z`  
**Memo:** [076](../../../../001-edgquake-improvements/076-mix-law-remaining-after-l15.md)

## Results (n=40)

| Metric | EQ | LR | vs L1.5 smoke |
|--------|----|----|---------------|
| Acc | **0.776** | 0.740 | Δ+0.036 CI includes 0 |
| ctx_rel | **0.513** | 0.513 | **tied LR** (L1.5 was 0.488) |
| ER | 0.954 | 0.963 | PASS ≥LR−0.03 |
| Fact ER | **0.950** | — | strong |

## Verdict

- [x] Naive-first RR recovers ctx_rel to LR parity on smoke
- [ ] Confirm on medical-mid n=200 (`make bench001-medical-mid-lr-nf-fact-l2`)
