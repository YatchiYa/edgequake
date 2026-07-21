# 036 — Close a1fp recall without dual-list / LR-budget Acc tax

**Status:** Closed — **no promote** (query-only levers exhausted; Mix ceiling)  
**Cross-ref:** [035](./035-fact-ce-bm25-protect.md) · [025](./025-recall-parity-under-p2b.md) · [034](./034-l2-dual-list-under-full-ws-graph.md)  
**Warm:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4` · query-only (disk &lt;15 Gi)

---

## 1. How close now? (split peers)

| Package | Acc | ctx | recall | Fact ER | Sum ER | Dual-list |
|---------|----:|----:|-------:|--------:|-------:|-----------|
| **a1fp** T095809Z | **0.775** | **0.500** | 0.926 ✗ | **0.85** | 0.86 | off |
| a1lrl2 T093152Z | 0.718 | **0.525** | **0.933** ✓ | 0.85 | 0.88 | on · Acc tax |
| LR (a1fp run) | 0.787 | — | 0.965 | 0.90 | **0.983** | — |

**Verdict:** Acc statistical tie on `a1fp`. L2 Parity only with dual-list tax. Binding without tax: **Contextual Summarize ER 0.86 vs LR 0.98** (Complex already 1.0).

---

## 2. First-principles diagnosis

1. GraphRAG-Bench Summarize = **coverage** of multi-facet evidence (Level 3).
2. CE + `min_rerank_score=0.1` optimizes **precision** → hard-drops Mix tails before protect.
3. Smoking gun: `Medical-0002d2de` EQ context **41k** chars vs LR **79k** (same short blob on a1fp/a1lr/a1lrl2 CE set).
4. Dual-list widens L2 blob but Acc-taxes. LR VECTOR budget Acc-taxes on a1fp stack.
5. Hard miss `Medical-0002d2de`: only **6** Mix parts (41k chars) even with dual-list — **ingest/retrieval ceiling**, not CE filter.
6. Typical Summarize has 22–30 Mix parts; CE + protect=12 drops coverage tails from the admitted set.

**Law (revised):** For Exploratory, CE may **reorder** but must not **shrink** Mix[:top_k] membership.  
`EDGEQUAKE_COVERAGE_PROTECT_FIRST=30` on Exploratory only (Fact keeps protect=12 + BM25 first-stage).

Forbidden: `L2_SOURCES_UNION`, `KG_CHUNK_PICK_LR_BUDGET`, FAQ, global protect↑ fishing.

---

## 3. Ladder

| Step | Pins |
|------|------|
| `a1fpm0` | a1fp + `MIN_RERANK_SCORE=0` — **reject** (Fact ER↓, Sum flat) |
| `a1fpcov` | a1fp + `COVERAGE_PROTECT_FIRST=30` |

---

## 4. Gates

| Outcome | Gate |
|---------|------|
| **Beat** | CI excludes 0 EQ ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| **Parity** | CI includes 0 ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| Step | Acc ≥ a1fp−0.02 (0.755) · Sum ER ↑ · no dual-list |

---

## 5. Results

| Run | Acc | recall | ctx | Fact ER | Sum ER | Note |
|-----|----:|-------:|----:|--------:|-------:|------|
| T095809Z a1fp | **0.775** | 0.926 | 0.500 | **0.85** | 0.86 | **keep Acc peer** |
| T100538Z a1fpm0 | 0.753 | 0.914 | 0.525 | 0.80 | 0.86 | **reject** |
| T101322Z a1fpcov | 0.748 | 0.916 | 0.519 | 0.80 | 0.86 | **reject** |

**Decision:** Query-only path to Parity without dual-list/LR-budget **blocked**. Binding Sum miss (`0002d2de`, 6 Mix parts) is a **first-stage Mix pool ceiling** (unchanged under dual-list and protect=30). Keep [035 `a1fp`](./035-fact-ce-bm25-protect.md) Acc peer; L2 Parity stays [034 `a1lrl2`](./034-l2-dual-list-under-full-ws-graph.md).  
**Follow-up [037](./037-summarize-chunk-link-audit.md):** law **SELECT** (not LINK) — `BONE_CANCER` already linked (5 vs 6); EQ Mix has **0** hits on question bigram `bone cancers`. Next confound = Mix topic-entity admission, **not** densify-all re-ingest.
