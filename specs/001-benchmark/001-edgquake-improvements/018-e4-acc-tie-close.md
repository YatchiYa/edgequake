# 018 — Acc-win E4: Persistent Acc Tie (Honesty Close)

**Date:** 2026-07-19  
**Status:** **Closed** — persistent statistical Acc **tie**; no headline promotion  
**Cross-ref:** [017 Beat LightRAG](./017-beat-lightrag.md) · [020 Roadmap](./020-roadmap.md) · [011 Publication Acc Report](../011-publication-acc-report.md) · [019 Business brief](../019-business-eq-vs-lightrag-and-rag.md)

---

## 1. Claim (publishable)

Under fair dual-SUT Acc pins (mistral-small + mistral-embed, Mix arms on, RRF, chunk 1200/100, top-k 30, n=40 medical smoke):

> **EdgeQuake Mix and LightRAG Mix are statistically tied on Acc.**  
> Every Acc-win Δ Acc 95% bootstrap CI **includes 0**.  
> EdgeQuake does **not** beat LightRAG on this task under these pins.

Best **labeled** retrieval package remains S1 CE+protect (`T151125Z`): clears ctx_rel ≥ 0.50 with Acc/recall budgets vs baseline BM25. That package is **not** the Acc headline default (headline stays BM25 / `PRUNE=0` / `PROTECT_FIRST=0`).

---

## 2. Best labeled pins (reference profile — unpromoted)

```text
EDGEQUAKE_MIX_RELEVANCY_PRUNE=0
EDGEQUAKE_RERANKER=cross_encoder
EDGEQUAKE_RERANKER_PROVIDER=aliyun
EDGEQUAKE_RERANKER_MODEL=qwen3-rerank
EDGEQUAKE_PATH_PRUNE=0
EDGEQUAKE_RERANK_PROTECT_FIRST=12
EDGEQUAKE_ENTITY_RANK=degree
EDGEQUAKE_RELATED_CHUNK_NUMBER=5
EDGEQUAKE_MIX_LOCAL_WEIGHT=1
EDGEQUAKE_MIX_GLOBAL_WEIGHT=1
EDGEQUAKE_MIX_NAIVE_WEIGHT=1
BENCH001_EQ_RERANK_TOP_K=30
# fairness: MIX_ARM_GATE=false, MIX_FUSION=rrf, chunk 1200/100
```

**Authoritative archive:** [`smoke-20260719T151125Z`](../e2e/artifacts/history/smoke-20260719T151125Z/)  
**Confirmatory:** [`smoke-20260719T151836Z`](../e2e/artifacts/history/smoke-20260719T151836Z/) (ctx_rel variance 0.481)

---

## 3. Acc CI ledger (all include 0)

| Archive | Labeled pins | EQ Acc | LR Acc | Δ Acc 95% CI | EQ ctx_rel |
|---------|--------------|--------|--------|--------------|------------|
| `T124903Z` | BM25 baseline (headline) | 0.765 | 0.754 | [−0.061, +0.083] | 0.375 |
| `T151125Z` | **S1** CE+protect | 0.760 | 0.780 | [−0.105, +0.061] | **0.519** |
| `T151836Z` | S1 confirm | 0.751 | 0.771 | [−0.112, +0.069] | 0.481 |
| `T153436Z` | S1 + path 0.4 | 0.742 | 0.774 | [−0.129, +0.064] | 0.519 |
| `T153959Z` | S1 + entity_rank=query_score | 0.734 | 0.751 | [−0.108, +0.076] | 0.519 |
| `T154427Z` | S1 + related_chunk=8 | 0.752 | 0.743 | [−0.085, +0.111] | 0.506 |
| `T155350Z` | S1 + MIX_NAIVE_WEIGHT=2 | 0.734 | 0.786 | [−0.132, +0.032] | 0.500 |

**Decision rule:** CI excludes 0 ⇒ reliable Δ; includes 0 ⇒ **tie**. No row excludes 0 in EQ’s favor.

---

## 4. What Acc-win proved / falsified

| Hypothesis | Result |
|------------|--------|
| Soft path + CE protect stabilizes L2 ≥0.50 | **Supported** (E1 / S1 discovery) |
| Query-score entity order closes Complex ΔF1 ≤0.03 | **Falsified** (E2: ΔF1 −0.094) |
| `related_chunk` 5→8 closes Summarize recall ≥0.95 | **Falsified** (E3: flat 0.863) |
| Mix naive RRF weight ×2 closes Summarize ≥0.95 | **Falsified** (E3b: 0.882) |
| Soft Mix knobs yield Acc CI win | **Falsified** — persistent tie |

**Shipped labeled knobs (defaults unchanged for headline):**  
`EDGEQUAKE_ENTITY_RANK` · `EDGEQUAKE_MIX_{LOCAL,GLOBAL,NAIVE}_WEIGHT` · CE/protect/path (existing).

---

## 5. Remaining gaps (not Acc-win soft knobs)

| Gap | Evidence | Deferred harder path |
|-----|----------|----------------------|
| Complex F1 vs LR | ~−9 to −11pp under S1; recall often 1.0 | Relation/path serialization, query-conditioned graph packing beyond entity order ([012](./012-lens-multihop-graph.md)) |
| Summarize evidence_recall | EQ ~0.86–0.88 vs LR ~0.98 | **Truncation / chunk token budget** (`truncation.rs` `balance_context`) — HippoRAG2-compact vs prompt-heavy ([013](./013-lens-latency-ops.md)); ingest/keyword audit |
| Latency ~3× | EQ p50 ~10s vs LR ~2–3s | Phase 3 ops ([013](./013-lens-latency-ops.md)) |
| Acc CI win | Never observed on n=40 | Requires a gap above to move F1 enough for CI to exclude 0 |

**E4 choice:** Document the tie now. Do **not** run another soft Mix Acc. Truncation/budget is a **new ladder** (one confound, labeled), not part of Acc-win E0–E4 soft knobs.

---

## 6. Publish / product language

Stakeholder-facing narrative: **[019 Business brief](../019-business-eq-vs-lightrag-and-rag.md)**.

| Allowed | Forbidden |
|---------|-----------|
| “Peer / statistical tie with LightRAG on Acc under fair pins” | “Beats LightRAG” / “wins Acc” |
| “Labeled CE+protect improves ctx_rel to ~0.50” | Silent Acc headline = CE/protect |
| “Soft Mix ablations did not close Complex/Summarize gaps” | Claiming Acc-win from point estimates |

---

## 7. Program handoff

1. Acc-win E0–E4 **complete** (honesty).  
2. Acc headline defaults unchanged.  
3. Next program work (outside Acc-win soft ladder): Phase 3 latency **or** truncation/budget Summarize experiment **or** research PPR — each as a new labeled profile.  
4. Core ladder / `P0_paper` still blocked on an Acc CI win or an explicit waived claim.
