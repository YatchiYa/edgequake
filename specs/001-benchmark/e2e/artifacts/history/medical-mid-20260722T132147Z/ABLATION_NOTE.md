# Ablation — LR_DENSE_FACT_L2_v1

**Step:** lr-dense-fact-l2  
**Stage:** medical-mid  
**Pins:** 077 E1 — L1.5 + `EDGEQUAKE_BM25_RETRIEVAL=0`  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `medical-mid-20260722T132147Z`

## Gates vs L1.5 keep

| Gate | Target | Result |
|------|--------|--------|
| Acc CI | includes 0 or ci_low ≥ −0.061 | **FAIL** [−0.083, −0.010] LR |
| ctx_rel | ≥0.50 or ≥0.494 | **PASS** 0.504 |
| overall ER | ≥ LR − 0.03 | **PASS** 0.954 vs 0.956 |
| Fact ER | prefer ≥ LR − 0.03 | **PASS** 0.943 vs 0.960 |
| Acc `publish/latest` | untouched | **PASS** P0 |

## Verdict

- [x] Gate missed (do not promote) — **REJECT E1** Acc CI worse than L1.5; proceed E2 on L1.5 base
