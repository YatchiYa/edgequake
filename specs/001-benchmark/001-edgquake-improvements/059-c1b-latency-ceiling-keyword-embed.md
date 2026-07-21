# 059 — C1b latency ceiling (keyword vs embed honesty + BM25-all)

**Status:** Measured ✓ · Acc Fact peer unchanged  
**Date:** 2026-07-21  
**Archive:** [`T013842Z`](../e2e/artifacts/history/smoke-20260721T013842Z/)  
**Cross-ref:** [057](./057-latency-horizon-c-baseline.md) · [058](./058-c1a-fact-ce-skip-latency.md) · [055](./055-post-acc-ceiling-first-principles.md)

---

## 1. First principles

| Claim | Law |
|-------|-----|
| Latency SLO | EQ/LR p50 ≤ **1.5×** — **not** an Acc Beat gate |
| Stage honesty | Keyword LLM ≠ embed. Mislabeling hides the tax |
| Ceiling | Under Acc Mistral pins, **generate alone can exceed 1.5× LR** |

**Industry (2026):** true query embed ~35–150 ms local; generation 60–85% of budget.  
**EQ Acc (pre-059):** “embed” p50 ~2.5s was mostly `max(keyword_llm, embed_one)`.

---

## 2. Measured (`T013842Z`)

| Stage | p50 ms | Note |
|-------|-------:|------|
| **keyword** | **1782** | Newly isolated |
| embed (pure) | 2212 | Mistral remote + keyword-level embeds |
| retrieve | 539 | |
| **rerank** | **9** | BM25-all — CE gone |
| **generate** | **2421** | **> 1.5× LR (≈2221)** |
| **EQ total** | **5791** | ratio **3.91×** vs LR 1481 |

| | Acc Fact | C1a | C1b |
|--|--------:|----:|----:|
| EQ Acc | **0.801** | 0.729 | 0.712 tax |
| EQ/LR p50 | 5.09× | 4.35× | **3.91×** |

**Verdict:** CE removal is done. Warm Acc EQ/LR **3.91×** was later shown to be **LR LLM-cache-aided** ([063](./063-why-lightrag-faster-cache-fairness.md) cold ≈ **1.01×**). Stage honesty (keyword ≠ embed) still stands.

---

## 3. Implementation

| Piece | Location |
|-------|----------|
| `keyword_time_ms` | `QueryStats` + API mapper + harness SUMMARY |
| Prepare split | `pipeline_prepare` times keyword ≠ embed futures |
| Pack | `make bench001-c1b` → `EDGEQUAKE_RERANKER=bm25` |

---

## 4. Product next (not Acc promote)

1. ~~Heuristic KEYWORD skip~~ — [060](./060-c1d-heuristic-keyword-latency.md) stage✓ / wall flat  
2. Fast KEYWORD **LLM** (nano/local, LR role pattern) that keeps Mix quality  
3. Stream TTFT / faster generate  
4. Local / faster embed provider

```bash
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
make bench001-c1b
```
