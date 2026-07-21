# Ablation — T0d_p2b_l2_bm25_fact_replace_v1

**Step:** t0d  
**Pins:** 027 T0d: P2b + L2_BM25_MODE=fact_replace (Fact BM25 L2; else CE)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`

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
