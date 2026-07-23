# 076 — Mix-law remaining after L1.5 (LR code ↔ EQ)

**Status:** Medical-mid peers done · **keep E2 occ** · NF/dense Acc REJECT  
**Date:** 2026-07-22  
**Prior:** [074](./074-why-eq-lags-lightrag-medical-mid.md) · [075](./075-lr-pack-bm25-close-gap.md) · [077](./077-dense-arms-fact-l2.md) · [078](./078-eq-vs-lightrag-first-principles-next.md)  
**Keep peer:** [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/) E2 occ · Acc CI tie · `publish/peers/LR_OCC_FACT_L2_v1/`  
**L1.5 baseline:** [`T125400Z`](../e2e/artifacts/history/medical-mid-20260722T125400Z/)  
**NF reject:** [`T130356Z`](../e2e/artifacts/history/medical-mid-20260722T130356Z/) · dense REJECT [`T132147Z`](../e2e/artifacts/history/medical-mid-20260722T132147Z/)  
**Acc SSOT:** P0 [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) · `publish/latest`

---

## 1. What L1.5 already matches

| Law | LightRAG | EQ under `LR_IDENTITY_FACT_L2_v1` |
|-----|----------|-----------------------------------|
| Mix fusion | round-robin chunks | `MIX_FUSION=round_robin` |
| Post-fuse rerank | `enable_rerank=False` (fair pin) | `BENCH001_EQ_ENABLE_RERANK=0` |
| Graph walk | no PPR | `GRAPH_WALK=bfs` |
| KG→chunk | VECTOR + `related×N/2` | `KG_CHUNK_PICK=vector` + `LR_BUDGET=1` |
| Entity order | VDB / retrieval | `ENTITY_RANK=retrieval` |
| Fact L2 lexical | (n/a — one list) | `L2_BM25_MODE=fact_replace` (citations only) |
| Arms / chunk / tokens | always-on · 1200/100 · 6k/8k/30k | matched |

---

## 2. Ranked remaining divergences (code authority)

Primary LR path: `LightRAG/lightrag/operate.py` — `kg_query` → `_perform_kg_search` → `_merge_all_chunks` → `rag_response`.

### R1 — RR arm order (Acc/ctx packing) — **next confound if medical-mid peer misses**

LR `_merge_all_chunks` interleaves **naive → entity → relation** (vector first).  
EQ [`hybrid_merge.rs`](../../../edgequake/crates/edgequake-query/src/hybrid_merge.rs) `round_robin_merge_chunks` uses **local → global → naive**.

With post-fuse rerank off, order = prompt order. LR favors naive evidence early; EQ favors KG local first → different ctx_rel / Complex Acc.

**Locked follow-up:** `EDGEQUAKE_RR_ORDER=naive_first` (default keep local-first for Acc headline).

### R2 — In-arm BM25 still on → **REJECT Acc (077 E1)**

Fair LR Mix arms are dense-only when rerank is off. EQ `EDGEQUAKE_BM25_RETRIEVAL` defaults **on** inside naive/local/global even when `enable_rerank=false`. Residual R3-like noise after L1.5. **Status:** mid Acc CI [−0.083, −0.010] worse than L1.5 → REJECT; L2 ctx/Fact ER improved — see [077](./077-dense-arms-fact-l2.md).

### R3 — KG→chunk timing → **REJECT Acc (078)**

LR: truncate entities/relations → **one** VECTOR pick → merge.  
EQ Mix: each arm VECTOR(+LR budget) → fuse → truncate later ([`mix.rs`](../../../edgequake/crates/edgequake-query/src/engine_impl/modes/mix.rs), [`chunk_retrieval.rs`](../../../edgequake/crates/edgequake-query/src/engine_impl/modes/chunk_retrieval.rs)). Can keep chunks from entities later dropped and double-apply budgets. **Status:** mid Acc CI [−0.076, −0.001] worse than E2 → REJECT; Fact ER improved (0.930) — keep E2; see [078](./078-eq-vs-lightrag-first-principles-next.md).

### R4 — Relation select

LR local: incident edges sorted `(rank, weight)`.  
EQ L1.5: BFS depth-2 (`RELATION_SELECT=default`). Law exists as `RELATION_SELECT=lightrag` but stays off under this pack.

### R5 — Occurrence sort → **keep Acc CI (077 E2); Fact ER still open**

LR always sorts each entity/relation’s `source_id` parts by global citation count before VECTOR take.  
EQ: `KG_CHUNK_OCCURRENCE_SORT=0` under Acc/L1.5. **Status:** mid Acc CI [−0.031, +0.040] best keep; Fact ER 0.917 still < LR−0.03 → stop packing fishing.

### R6 — fact_replace Acc/L2 split (by design)

[`query_pipeline.rs`](../../../edgequake/crates/edgequake-query/src/engine_impl/query_entry/query_pipeline.rs): Acc prompt = Mix (no post-fuse BM25); Fact L2 citations = BM25 replace of Mix. Smoke Fact ER≠Acc proof. LR uses one chunk list for both.

### Explicitly not next

- Copy LR sequential arms (EQ parallel is better)
- Soft Mix Acc fishing / TOPIC_* / silent Acc headline replace
- Force re-ingest before medical-mid peer (B5 zero-chunk already 0%)

---

## 3. Medical-mid peer experiment

```bash
make bench001-medical-mid-lr-identity-fact-l2
# → n=200 · profile LR_IDENTITY_FACT_L2_v1 · publish/peers/… · SKIP publish/latest
```

| Gate | Target |
|------|--------|
| Acc Beat | CI excludes 0 **and** EQ ahead — else tie / LR ahead |
| ctx_rel | ≥0.50 preferred |
| evidence_recall | ≥ LR − 0.03 |
| Fact ER | ≥ LR − 0.03 |
| Acc `publish/latest` | unchanged P0 medical-mid |

### Medical-mid L1.5 result (triggers Phase 4)

| Gate | Result |
|------|--------|
| Acc CI tie | **PASS** [−0.061, +0.013] (was LR-ahead Acc headline) |
| overall ER ≥ LR−0.03 | **PASS** (0.946 vs 0.960) |
| ctx_rel ≥ 0.50 | **MISS** (0.474) → **R1 naive-first RR** |
| Fact ER ≥ LR−0.03 | **MISS** (0.919 vs 0.960) |

### Phase 4 result (naive-first RR)

| Stage | Acc CI | ctx_rel | Verdict |
|-------|--------|---------|---------|
| Smoke NF [`T125634Z`](../e2e/artifacts/history/smoke-20260722T125634Z/) | tie · EQ point ahead | **0.513** (=LR) | green |
| Mid NF [`T130356Z`](../e2e/artifacts/history/medical-mid-20260722T130356Z/) | **[−0.080, −0.007] LR** | 0.474 (flat vs L1.5) | **REJECT** |

**Keep (post-077):** E2 occ medical-mid peer (`LR_OCC_FACT_L2_v1`) — best Acc CI. L1.5 = prior baseline. NF/dense = REJECT Acc. Code for `EDGEQUAKE_RR_ORDER` stays (escape hatch); default `local_first`.

**Done:** [077](./077-dense-arms-fact-l2.md) — E1 dense REJECT Acc · E2 occ keep Acc CI; Fact ER packing stop.

---

## 4. Reading order

1. This memo (remaining Mix laws)  
2. [075 L1.5 pack](./075-lr-pack-bm25-close-gap.md)  
3. [074 why we lag](./074-why-eq-lags-lightrag-medical-mid.md)  
4. [061 LR-as-law](./061-lightrag-law-first-principles-eq.md)  
