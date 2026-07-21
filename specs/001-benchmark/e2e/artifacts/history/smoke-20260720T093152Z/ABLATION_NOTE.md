# Ablation — A1LRL2_p2b_rr_cer_lr_budget_l2_union_v1

**Step:** a1lrl2  
**Archive:** `smoke-20260720T093152Z`  
**Pins:** 034 decision — A1 + `KG_CHUNK_PICK_LR_BUDGET=1` + `L2_SOURCES_UNION=1` (+ citation top_k skip)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Concurrency:** EQ=4

## Results

| Metric | EQ | LR | Gate |
|--------|---:|---:|------|
| Acc | 0.7178 | 0.7805 | Δ−0.063 · CI [−0.158, +0.030] includes 0 ✓ |
| ctx_rel | **0.525** | 0.531 | ≥0.50 ✓ |
| evidence_recall | **0.9325** | 0.9625 | ≥LR−0.03 (0.9325) ✓ |
| Fact ER | **0.85** | — | ↑ from 0.80 |

## Verdict

- [x] **Parity** — CI includes 0 ∧ ctx≥0.50 ∧ recall≥LR−0.03
- [ ] Beat — CI does not exclude 0 favoring EQ
- Acc point estimate still lags LR (dual-list tax / variance) — do not claim Acc win
