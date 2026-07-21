# Ablation — A1FPSEL_p2b_rr_cer_fact_protect_topic_admit_v1

**Step:** a1fpsel  
**Pins:** 038 a1fpsel: A1 + FACT_PROTECT_BM25 + TOPIC_ENTITY_ADMIT=1 (Exploratory SELECT)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`

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
