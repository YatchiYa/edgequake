# Ablation — P2b_lr_pack_s1_v1

**Step:** p2b  
**Pins:** S1 CE+protect + `ENTITY_RANK=retrieval` + `CONTEXT_FORMAT=path` + soft path0.4 + `CONTENT_HEADINGS=1`  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T014814Z`

## Results

| Metric | P2b | Gate | Result |
|--------|-----|------|--------|
| EQ Acc | **0.752** | ≥ P0 −0.02 | **+0.008 vs P0** ✅ |
| LR Acc | 0.780 | — | — |
| Δ Acc 95% CI | [−0.111, +0.047] | — | tie |
| EQ ctx_rel | **0.500** | ≥0.48 | ✅ (P4 bar too) |
| Complex Acc EQ/LR | 0.803 / 0.826 | Δ≤0.05 | **−0.023** ✅ |
| Complex F1 EQ/LR | 0.750 / 0.779 | ΔF1≤0.03 | **−0.029** ✅ |
| evidence_recall EQ/LR | 0.939 / 0.962 | ≥LR−0.03 | **−0.023** ✅ |

## Verdict

- [x] Gate met — P2 Complex packing closed on S1 lr_pack
- [ ] Gate missed (do not promote)

**Note:** Best Acc so far on this ladder. Soft path only with CE (not BM25). Carry pack into P3/P4; do **not** promote Acc headline until P4 CI excludes 0.
