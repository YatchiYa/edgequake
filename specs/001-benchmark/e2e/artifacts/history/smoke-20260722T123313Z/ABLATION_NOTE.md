# Ablation — LR_IDENTITY_FACT_L2_v1

**Step:** lr-identity-fact-l2  
**Pins:** 075 L0 identity + `L2_BM25_UNION=1` `MODE=fact_replace`  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `smoke-20260722T123313Z`  
**Memo:** [075](../../../../001-edgquake-improvements/075-lr-pack-bm25-close-gap.md)

## Results (smoke n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | **0.806** | 0.772 | **+0.034** · CI [−0.050, +0.120] (**tie**) |
| context_relevancy | 0.488 | 0.531 | −0.044 |
| evidence_recall | **0.962** | 0.967 | **−0.006** |
| Fact ER | **1.000** | 0.900 | **+0.100** |

### Ladder comparison

| Pack | Acc Δ | ctx_rel | ER | Fact ER | Verdict |
|------|-------|---------|-----|---------|---------|
| Acc headline medical-mid | −0.068 (CI≠0) | 0.396 | 0.887 | 0.790 | publish SSOT |
| L0 identity | −0.048 (tie) | **0.506** | 0.904 | 0.700 | ctx OK · Fact ER miss |
| L1 pack+BM25 | −0.101 (LR) | 0.419 | **0.941** | **0.950** | Fact OK · ctx tax |
| **L1.5 identity+Fact L2** | **+0.034 (tie)** | **0.488** | **0.962** | **1.000** | **best L2+Acc smoke** |

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Honesty | No “beats LightRAG” | **PASS** — CI includes 0; smoke n=40 ≠ publish |
| Fact ER | ≥LR−0.03 | **PASS** (1.0) |
| overall ER | ≥LR−0.03 | **PASS** (−0.006) |
| ctx_rel | ≥0.48 | **PASS** (0.488) |
| Acc promote | medical-mid gates only | **no promote** — labeled peer candidate |

## Verdict

- [x] Best query-only pack to close L2 + Acc point gap vs LR on smoke
- [x] Keep labeled; do not merge into Acc headline without medical-mid n=200
- [ ] Optional: re-run medical-mid under this pack (new labeled publish peer, not silent Acc)
