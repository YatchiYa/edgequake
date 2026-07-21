# Ablation — A1FP_p2b_rr_cer_fact_protect_bm25_v1

**Step:** a1fp  
**Pins:** 035 a1fp: A1 + FACT_PROTECT_BM25=1 (BM25 Mix→CE protect; no dual-list)  
**Workspace:** `b4f595be-08aa-4e75-abd9-7cfc5663b039`

## Gates (fill from SUMMARY)

| Gate | Target | Result |
|------|--------|--------|
| path_prune_fraction pin | 0 for P0/P1a/P3/P5 | |
| Δ Acc 95% CI | includes 0 (P0) / excludes 0 EQ (P4) | |
| EQ ctx_rel | ≥0.48 (P1) / ≥0.50 (P4) | |
| Complex Acc Δ vs LR | ≤0.05 (P1/P2) | |
| Summarize evidence_recall | ≥0.95 or ≥LR−0.03 (P3) | |
| EQ/LR p50 ratio | ≤1.5× (P5) | |

## Verdict

- [ ] Gate met
- [ ] Gate missed (do not promote)
