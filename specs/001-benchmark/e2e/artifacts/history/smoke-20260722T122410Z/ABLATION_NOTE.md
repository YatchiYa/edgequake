# Ablation — LR_IDENTITY_v1

**Step:** lr-identity  
**Pins:** 074 lr-identity: RR fuse · enable_rerank=0 · GRAPH_WALK=bfs · VECTOR+LR_BUDGET · ENTITY_RANK=retrieval — L2 confound pack; not Acc Beat  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `smoke-20260722T122410Z`  
**Memo:** [074](../../../../001-edgquake-improvements/074-why-eq-lags-lightrag-medical-mid.md)

## Results (smoke n=40)

| Metric | EQ | LR | Δ (EQ−LR) |
|--------|----|----|-----------|
| Acc | 0.751 | 0.799 | −0.048 · CI [−0.121, +0.037] (tie) |
| context_relevancy | **0.506** | 0.519 | **−0.013** |
| evidence_recall | 0.904 | 0.967 | −0.062 |

### vs Acc headline medical-mid n=200 (publish SSOT)

| Metric | Acc headline Δ | LR-identity smoke Δ | Signal |
|--------|----------------|---------------------|--------|
| ctx_rel | −0.095 | **−0.013** | L2 noise confounds (R1/R3/R4) largely closed on gate |
| evidence_recall | −0.064 | −0.062 | R2 Fact/evidence miss remains (ingest / provenance) |
| Acc | −0.068 (CI excludes 0) | −0.048 (CI includes 0) | Smoke only — **not** Acc Beat; do not replace medical-mid |

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Honesty | No “EQ beats LightRAG” claim | **PASS** — CI includes 0; point estimate still LR ahead |
| Not Acc headline | Acc pin remains RRF+BM25+PPR+degree | **PASS** — labeled pack only |
| ctx_rel Δ vs LR | shrinks vs Acc headline (−0.095) | **PASS** (−0.013) |
| evidence_recall Δ vs LR | shrinks vs Acc headline (−0.064) preferred | **MISS** (−0.062 ≈ same) → L1 ingest |
| Δ Acc 95% CI | report only; do **not** promote as Acc Beat | **PASS** — no promote |

## Verdict

- [x] L2 confound signal useful (keep labeled) — ctx_rel nearly ties LR under identity pins
- [x] No Acc promote / no peer merge with a1fp or a1lrl2
- [x] `publish/latest` remains medical-mid n=200 (restored; future `lr-identity` sets `BENCH001_SKIP_PUBLISH_LATEST=1`)

**Next:** L1 ingest / provenance (Horizon B) for Fact ER; do not fish Acc on soft Mix.
