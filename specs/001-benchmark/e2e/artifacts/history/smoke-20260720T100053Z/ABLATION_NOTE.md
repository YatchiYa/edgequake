# Ablation — A1FPLR_p2b_rr_cer_fact_protect_lr_budget_v1

**Step:** a1fplr  
**Archive:** `smoke-20260720T100053Z`  
**Pins:** 035 — A1 + `FACT_PROTECT_BM25=1` + `KG_CHUNK_PICK_LR_BUDGET=1`  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`

## Results

| Metric | EQ | Gate |
|--------|---:|------|
| Acc | 0.7384 | ≥0.753 ✗ (tax vs a1fp 0.775) |
| ctx | 0.519 | ≥0.50 ✓ |
| recall | 0.918 | ≥LR−0.03 ✗ |
| Fact ER | 0.85 | flat vs a1fp |

## Verdict

- [ ] Reject stack — LR budget + Fact protect Acc-toxic; keep **a1fp** as Acc peer
