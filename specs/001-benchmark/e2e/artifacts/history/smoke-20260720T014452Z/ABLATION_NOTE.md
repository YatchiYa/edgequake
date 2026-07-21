# Ablation — P2a_round_robin_fusion_v1

**Step:** p2a  
**Pins:** `MIX_FUSION=round_robin` (LightRAG parity; Acc path off)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T014452Z`

## Results

| Metric | P2a | vs P0 (0.744) | Gate | Result |
|--------|-----|---------------|------|--------|
| EQ Acc | 0.723 | −0.021 | Acc tax ≤0.02 preferred | **borderline tax** |
| LR Acc | 0.791 | — | — | — |
| Δ Acc 95% CI | [−0.162, +0.024] | — | — | tie |
| EQ ctx_rel | 0.363 | — | ≥0.48 (P1/P2 pack) | **miss** |
| Complex Acc EQ/LR | 0.677 / 0.830 | Δ −0.154 | Δ≤0.05 | **miss** |
| Complex F1 EQ/LR | 0.580 / 0.785 | Δ −0.205 | ΔF1≤0.03 | **miss** |

## Verdict

- [ ] Gate met
- [x] Gate missed — round_robin fusion alone does **not** close Complex packing; Acc tax vs P0

**Note:** Keep Acc headline on RRF. Continue P2b (S1 + `ENTITY_RANK=retrieval` + path format + headings).
