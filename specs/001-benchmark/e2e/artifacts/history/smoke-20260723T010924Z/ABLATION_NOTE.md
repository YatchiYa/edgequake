# Ablation — LR_UNIFY_FACT_L2_v1

**Step:** lr-unify-fact-l2  
**Stage:** smoke  
**Pins:** 080 D1 R6: E2 + `L2_BM25_MODE=unified` (citation_chunks = Acc prompt); not Acc Beat  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `smoke-20260723T010924Z`  
**Memo:** [080](../../../../001-edgquake-improvements/080-beat-lightrag-evidence-roadmap.md)

## Gates (from SUMMARY)

| Gate | Target | Result |
|------|--------|--------|
| Honesty | No Beat claim; Acc latest frozen | PASS (`not_acc_headline`, skip publish latest) |
| Acc CI vs LR | not clearly worse (smoke underpowered) | PASS — tie CI [−0.067, +0.103]; EQ 0.780 / LR 0.765 |
| ctx_rel | ≥0.48 smoke / ≥0.50 preferred | PASS — EQ 0.519 |
| Fact ER | ≥LR−0.03 or ≥E2 OCC smoke | MISS smoke — EQ 0.80 / LR 0.90 (E2 OCC smoke Fact ER 0.95) |
| overall ER | ≥LR−0.03 preferred | MISS — EQ 0.921 / LR 0.962 |
| vs E2 OCC smoke | Acc/ctx not collapse | Acc −1.4pp · ctx −4.3pp · Fact ER −15pp (n=40 noisy) |

## Verdict

- [x] Smoke proceed → medical-mid (directional; Fact ER needs n=200)
- [ ] Gate met for KEEP (defer to mid)
- [ ] Gate missed REJECT (defer to mid)

**Next:** `make bench001-medical-mid-lr-unify-fact-l2`
