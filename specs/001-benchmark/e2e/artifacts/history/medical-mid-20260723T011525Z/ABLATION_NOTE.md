# Ablation — LR_UNIFY_FACT_L2_v1

**Step:** lr-unify-fact-l2  
**Stage:** medical-mid  
**Pins:** 080 D1 R6: E2 + `L2_BM25_MODE=unified`; not Acc Beat  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `medical-mid-20260723T011525Z`  
**Peer:** `LR_UNIFY_FACT_L2_v1` (Acc `publish/latest` skipped)  
**Memo:** [080](../../../../001-edgquake-improvements/080-beat-lightrag-evidence-roadmap.md)  
**Baseline keep:** E2 occ [`medical-mid-20260722T133053Z`](../medical-mid-20260722T133053Z/)

## Gates vs E2 keep

| Gate | Target | Result |
|------|--------|--------|
| Honesty | No Beat; Acc latest frozen | PASS |
| Acc CI not worse | not clearly LR-ahead vs E2 tie | **FAIL** — CI [−0.084, −0.022]; EQ 0.734 / LR 0.787 (E2 was tie 0.765/0.760) |
| ctx_rel | ≥0.50 or ≥E2+0.02 | PASS — EQ 0.503 (E2 0.491) |
| Fact ER | ≥LR−0.03 or ≥E2+0.02 | **FAIL** — EQ 0.903 / LR 0.943 (need ≥0.913); E2 Fact ER 0.917 |
| overall ER | report | EQ 0.943 / LR 0.945 (≈tie) |

## Verdict

- [ ] KEEP
- [x] **REJECT** — do not run medical-full unify; keep E2 `fact_replace` dual-list as query base

**Insight:** Unifying Acc prompt list with L2 citation list improved ctx_rel slightly but regressed Acc (−3.1pp vs E2) and Fact ER (−1.3pp vs E2). Dual-list `fact_replace` remains the better gap-close packing.

**Next:** D2 `make bench001-lr-intent-w-fact-l2` on E2 base (`L2_BM25_MODE=fact_replace`, `MIX_INTENT_WEIGHTS=1`) — do **not** stack rejected unify.
