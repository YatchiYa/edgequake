# Ablation — LR_PACK_BM25_v1

**Step:** lr-pack-bm25  
**Pins:** 075 lr-pack-bm25: RR fuse · BM25 on · GRAPH_WALK=bfs · VECTOR+LR_BUDGET · ENTITY_RANK=retrieval — L1 Fact ER; not Acc Beat  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `smoke-20260722T123021Z`  
**Memo:** [075](../../../../001-edgquake-improvements/075-lr-pack-bm25-close-gap.md)

## Results (smoke n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | 0.701 | 0.802 | −0.101 · CI [−0.181, −0.021] (LR ahead) |
| context_relevancy | 0.419 | 0.556 | −0.137 |
| evidence_recall | **0.941** | 0.963 | **−0.022** (≥ LR−0.03) |
| Fact ER | **0.950** | 0.900 | **+0.050** |

### vs L0 / Acc headline

| Metric | Acc headline | L0 identity | L1 pack+BM25 |
|--------|--------------|-------------|--------------|
| ctx_rel | 0.396 | **0.506** | 0.419 |
| overall ER | 0.887 | 0.904 | **0.941** |
| Fact ER | 0.790 | 0.700 | **0.950** |

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Honesty | No Beat claim | **PASS** — CI excludes 0 for LR |
| Fact ER | ≥0.80 / ≥LR−0.10 | **PASS** (0.95 ≥ LR) |
| overall ER | ≥LR−0.03 | **PASS** (−0.022) |
| ctx_rel | ≥0.48 or better than Acc Δ | **MISS** (0.419; BM25 re-noise) |
| Acc promote | forbidden | **PASS** — no promote |

## Verdict

- [x] Fact ER recovered by keeping BM25 under LR packing
- [x] ctx_rel tax confirms R3 — do **not** promote global BM25 as identity pack
- [ ] Next: L1.5 `lr-identity-fact-l2` (Fact-only L2 BM25; Mix prompt stays L0-clean)
