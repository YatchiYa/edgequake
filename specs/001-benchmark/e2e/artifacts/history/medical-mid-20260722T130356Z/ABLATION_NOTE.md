# Ablation — LR_NF_FACT_L2_v1 (medical-mid n=200)

**Step:** lr-nf-fact-l2  
**Stage:** medical-mid  
**Pins:** L1.5 + `EDGEQUAKE_RR_ORDER=naive_first`  
**Archive:** `medical-mid-20260722T130356Z`  
**Peer:** `publish/peers/LR_NF_FACT_L2_v1/`  
**Memo:** [076](../../../../001-edgquake-improvements/076-mix-law-remaining-after-l15.md)

## Results

| Metric | Acc headline | L1.5 mid | **NF mid** | LR (NF run) |
|--------|--------------|----------|------------|-------------|
| Acc | 0.706 | **0.746** (CI tie) | 0.742 | 0.785 |
| Δ Acc CI | [−0.107, −0.033] | **[−0.061, +0.013]** | [−0.080, −0.007] LR | — |
| ctx_rel | 0.396 | 0.474 | 0.474 | 0.505 |
| ER | 0.887 | 0.946 | **0.956** | 0.961 |
| Fact ER | 0.790 | 0.919 | 0.923 | 0.990 |

## Gates

| Gate | Result |
|------|--------|
| Honesty | **PASS** — no Beat claim |
| ctx_rel ≥0.50 | **MISS** (0.474; smoke had 0.513 — n=40 not predictive) |
| ER ≥LR−0.03 | **PASS** |
| Fact ER ≥LR−0.03 | **MISS** |
| Acc CI | **worse than L1.5** — CI excludes 0 for LR |

## Verdict

- [x] Naive-first RR shipped (`EDGEQUAKE_RR_ORDER`) + unit tests
- [x] Smoke looked green; medical-mid Acc CI **regressed** vs L1.5
- [x] **Keep labeled peer = L1.5 mid** (`gap_close_l15`); do **not** promote NF or Acc headline
- Next Mix-law confounds (076): in-arm BM25 off, occurrence sort — one at a time on L1.5 base
