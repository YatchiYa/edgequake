# Ablation — LR_INTENT_W_FACT_L2_v1

**Step:** lr-intent-w-fact-l2  
**Stage:** smoke  
**Pins:** 080 D2: E2 + `MIX_INTENT_WEIGHTS=1` + `L2_BM25_MODE=fact_replace`; not Acc Beat  
**Archive:** `smoke-20260723T011911Z`  
**Memo:** [080](../../../../001-edgquake-improvements/080-beat-lightrag-evidence-roadmap.md)

## Gates

| Gate | Result |
|------|--------|
| Pins | PASS — `fact_replace` + intent_w (ladder bugfix: force fact_replace) |
| Acc CI | PASS — tie-ish [−0.107, +0.053]; EQ 0.764 / LR 0.793 |
| ctx ≥0.48 | PASS — 0.500 |
| Fact ER | PASS smoke — EQ 0.95 / LR 0.90 |

## Verdict

- [x] Proceed → medical-mid
