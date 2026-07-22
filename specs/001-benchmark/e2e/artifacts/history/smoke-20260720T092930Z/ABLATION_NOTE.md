# Ablation — A1LR_p2b_rr_cer_kg_lr_budget_v1

**Step:** a1lr  
**Archive:** `smoke-20260720T092930Z`  
**Pins:** 034 — A1 + `KG_CHUNK_PICK_LR_BUDGET=1` (L2 union off)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Concurrency:** EQ=4

## Results

| Metric | EQ | LR | Gate |
|--------|---:|---:|------|
| Acc | **0.7583** | 0.7570 | Δ+0.001 · CI includes 0 |
| ctx_rel | **0.506** | 0.531 | ≥0.50 ✓ |
| evidence_recall | 0.9275 | 0.9625 | ≥LR−0.03 (0.932) ✗ by **0.005** |
| Fact ER | 0.80 | — | still flat |

## Verdict

- [ ] Parity — **miss by hair** on recall
- [x] Acc stable (no dual-list tax)
- [x] ctx cleared under full WS + LR VECTOR budget
- Next: **a1lrl2** decision (budget + dual-list)
