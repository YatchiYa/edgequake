# Ablation — C1E_a1_rr_cer_bm25_fast_keyword_v1

**Step:** c1e  
**Pins:** 062 c1e: A1 + BM25-all + KEYWORD=ministral-3b — latency peer; not Acc promote  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`

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
