# Ablation — A1FP_p2b_rr_cer_fact_protect_bm25_v1

**Step:** a1fp  
**Archive:** `smoke-20260720T095809Z`  
**Pins:** 035 — A1 + `FACT_PROTECT_BM25=1` (no dual-list, no LR budget)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4` · EQ concurrency=4

## Results

| Metric | EQ | LR | Gate |
|--------|---:|---:|------|
| Acc | **0.7753** | 0.7874 | Δ−0.012 · CI includes 0 · ≥0.753 ✓ |
| ctx_rel | **0.500** | — | ≥0.50 ✓ |
| evidence_recall | 0.9258 | 0.9647 | ≥LR−0.03 (0.935) ✗ |
| Fact ER | **0.85** | — | ↑ from 0.80 · matches dual-list Fact ER |

## Verdict

- [x] Acc point restored (no dual-list tax; ≥ T090743Z)
- [x] Fact ER 0.85 without `L2_SOURCES_UNION`
- [ ] Parity — recall still short → next **a1fplr**
