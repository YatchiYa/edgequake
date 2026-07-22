# Ablation — S0_p2b_l2_sources_union_v1

**Step:** s0  
**Pins:** 026 S0: P2b + L2_SOURCES_UNION=1 (Mix∪CE citations; Acc from CE prompt)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T031658Z`

## Gates (from SUMMARY)

| Gate | Target | Result |
|------|--------|--------|
| Δ Acc 95% CI | Beat: excludes 0 EQ / Parity: includes 0 | includes 0 `[-0.1135, +0.0198]` |
| EQ ctx_rel | ≥0.50 | **0.4875 MISS** |
| evidence_recall | ≥ LR−0.03 (0.9355) | **0.9293 MISS** (+0.016 vs Q4) |
| Acc floor | ≥ Q4−0.02 (0.736) | **0.726 MISS** |
| Fact ER | improve vs 0.85 | **0.85 flat** (union did not move Fact) |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote)

**Reading:** Mix∪CE inflated L2 blob (~+80k chars) and lifted overall recall slightly via Complex, but Fact stayed 0.85. ctx_rel taxed by Mix noise. Next: [027 Fact→BM25 intent](../../../001-edgquake-improvements/027-fact-bm25-intent-rerank.md).
