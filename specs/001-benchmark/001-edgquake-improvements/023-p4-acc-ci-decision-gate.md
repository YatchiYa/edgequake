# 023 — P4 Acc CI Decision Gate

**Status:** Decision complete — **do not promote** (`smoke-20260720T015647Z`)  
**Date:** 2026-07-20  
**Parent:** [022 Deep top-performance plan](./022-deep-top-performance-plan.md)

### Outcome (`T015647Z`)

| Gate | Result |
|------|--------|
| Δ Acc CI excludes 0 (EQ) | **Fail** — CI [−0.216, −0.007] favors LR |
| EQ ctx_rel ≥ 0.50 | Pass (0.500) |
| evidence_recall ≥ LR−0.03 | **Fail** (0.899 vs 0.969) |

Headline stays P0 Acc pins. Best labeled peer pack remains **P2b** (`T014814Z` Acc 0.752; confirmed `T024233Z` Acc 0.756, ctx_rel 0.506 — recall still blocks promote). See [024](./024-acc-parity-beat-plan.md) Q0–Q4 close.

---

## Package under test

Profile id: `P4_acc_ci_decision_v1`

```text
# S1 CE+protect
EDGEQUAKE_RERANKER=cross_encoder
EDGEQUAKE_RERANKER_PROVIDER=aliyun
EDGEQUAKE_RERANKER_MODEL=qwen3-rerank
EDGEQUAKE_RERANK_PROTECT_FIRST=12
EDGEQUAKE_PATH_PRUNE=0
# 022 hard levers
EDGEQUAKE_GRAPH_WALK_COMPRESS=1
EDGEQUAKE_ENTITY_RANK=retrieval
EDGEQUAKE_KEYWORD_LEXICAL_BOOST=1
EDGEQUAKE_POPULAR_NODE_FALLBACK=0
```

Requires `DASHSCOPE_API_KEY` + release binary + warm full-corpus workspace.

---

## Promote to Acc headline only if

1. Δ Acc 95% CI **excludes 0** in EQ’s favor  
2. EQ `context_relevancy` ≥ **0.50**  
3. EQ evidence_recall ≥ LR − 0.03  
4. Archive has `ABLATION_NOTE.md` + valid scorecard  

**If any fail:** keep headline BM25 / `PATH_PRUNE=0` / `PROTECT=0`. Document “improved peer / still tie.”

---

## Launch

```bash
export DASHSCOPE_API_KEY=...
cargo build --release --bin edgequake
make bench001-p4
```
