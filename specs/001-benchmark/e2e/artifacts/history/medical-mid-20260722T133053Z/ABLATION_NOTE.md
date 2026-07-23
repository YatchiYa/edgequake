# Ablation — LR_OCC_FACT_L2_v1

**Step:** lr-occ-fact-l2  
**Stage:** medical-mid  
**Pins:** 077 E2 — L1.5 + `EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1`  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `medical-mid-20260722T133053Z`

## Gates vs L1.5 keep

| Gate | Target | Result |
|------|--------|--------|
| Acc CI | not worse than L1.5 | **PASS** [−0.031, +0.040] (best) |
| Fact ER | ≥ LR − 0.03 | **MISS** 0.917 vs 0.953 (need ≥0.923) |
| ctx_rel | report | 0.491 (↑ vs L1.5 0.474; <0.50) |
| overall ER | ≥ LR − 0.03 | **PASS** 0.943 ≈ LR |
| Acc `publish/latest` | untouched | **PASS** P0 |

## Verdict

- [x] **KEEP** for Acc CI among L1.5/E1/E2 · Fact ER packing stop (gate miss)
