# Ablation — LR_IDENTITY_FACT_L2_v1 (medical-mid n=200)

**Step:** lr-identity-fact-l2  
**Stage:** medical-mid  
**Pins:** L0 identity + `L2_BM25 fact_replace` · `enable_rerank=0` · RR · bfs · VECTOR+LR budget · retrieval rank  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `medical-mid-20260722T125400Z`  
**Peer pack:** `publish/peers/LR_IDENTITY_FACT_L2_v1/`  
**Memo:** [076](../../../../001-edgquake-improvements/076-mix-law-remaining-after-l15.md)

## Results vs Acc headline (P0 medical-mid)

| Metric | Acc headline EQ | L1.5 peer EQ | LR (peer run) | Gate |
|--------|-----------------|--------------|---------------|------|
| Acc | 0.706 | **0.746** | 0.768 | Δ −0.022 · CI [−0.061, +0.013] → **tie** |
| ctx_rel | 0.396 | **0.474** | 0.493 | ≥0.50 **MISS** (improved vs Acc) |
| evidence_recall | 0.887 | **0.946** | 0.960 | ≥LR−0.03 **PASS** |
| Fact ER | 0.790 | **0.919** | 0.960 | ≥LR−0.03 **MISS** (−0.041) |
| empty answers | 3.5% | **0%** | 0% | R5 cleared |

## Honesty

- [x] No “EQ beats LightRAG” — CI includes 0
- [x] Acc `publish/latest` unchanged (P0 medical-mid SSOT)
- [x] Labeled peer only — not Acc headline promote
- [ ] Phase 4: `lr-nf-fact-l2` (naive-first RR) — ctx_rel / Fact ER residual

## Verdict

L1.5 closes Acc CI to a **tie** at n=200 and nearly matches overall ER. Remaining: ctx_rel packing (076 R1 RR order) + Fact ER −0.041. Next confound: `EDGEQUAKE_RR_ORDER=naive_first`.
