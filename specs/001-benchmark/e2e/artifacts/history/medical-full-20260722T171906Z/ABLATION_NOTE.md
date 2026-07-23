# Ablation — LR_OCC_FACT_L2 medical-full

**Step:** lr-occ-fact-l2  
**Stage:** medical-full (n=2062)  
**Pins:** 079 E2 keep pack · occurrence_sort=1 · skip Acc publish/latest  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `medical-full-20260722T171906Z`  
**Peer:** `publish/peers/LR_OCC_FACT_L2_medical_full_v1`

## Results vs mid E2 (n=200)

| Metric | Mid E2 | Full E2 |
|--------|--------|---------|
| EQ Acc | 0.765 | 0.739 |
| LR Acc | 0.760 | 0.784 |
| Acc Δ CI | [−0.031, +0.040] **tie** | **[−0.069, −0.017] LR** |
| ctx_rel | 0.491 | 0.472 |
| Fact ER | 0.917 | 0.918 |

## Verdict

- Scale check: mid Acc **tie** does **not** hold at full medical n=2062 — LR ahead with CI excluding 0.
- Acc `publish/latest` untouched (P0 mid SSOT).
