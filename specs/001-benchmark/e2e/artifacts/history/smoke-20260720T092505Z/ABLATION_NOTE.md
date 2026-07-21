# Ablation — A1L2_p2b_rr_cer_l2_union_v1

**Step:** a1l2  
**Archive:** `smoke-20260720T092505Z`  
**Pins:** 034 — A1 + `L2_SOURCES_UNION=1` + citation `rerank_top_k` skip when dual-list  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Concurrency:** EQ=4

## Results

| Metric | EQ | LR | Gate |
|--------|---:|---:|------|
| Acc | 0.7239 | 0.7583 | Acc ≥0.753 ✗ (tax vs T090743Z 0.773) |
| Δ Acc 95% CI | [−0.123, +0.049] | — | includes 0 (tie) |
| ctx_rel | **0.506** | 0.525 | ≥0.50 ✓ |
| evidence_recall | 0.915 | 0.964 | ≥LR−0.03 (0.934) ✗ |
| Fact ER | 0.80 | — | flat vs T090743Z |

## Verdict

- [ ] Beat / Parity — **missed** (recall + Acc)
- [x] Dual-list citation budget fix verified (Fact ctx chars ~88k→142k)
- [x] Fact ER not CE-membership-bound under full WS — Mix lacks gold → next **a1lr**
