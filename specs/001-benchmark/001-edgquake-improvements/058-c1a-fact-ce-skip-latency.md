# 058 — C1a Fact CE-skip (product latency; not Acc promote)

**Status:** Measured ✓ · Acc Fact peer unchanged  
**Date:** 2026-07-21  
**Archive:** [`T012849Z`](../e2e/artifacts/history/smoke-20260721T012849Z/)  
**Invalid discard:** [`T012604Z`](../e2e/artifacts/history/smoke-20260721T012604Z/) (pin not forwarded)  
**Cross-ref:** [057](./057-latency-horizon-c-baseline.md) · [055](./055-post-acc-ceiling-first-principles.md) · Acc peer [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/)

---

## 1. First principles (stage budget)

Acc Fact peer EQ p50 **6876** ms vs LR **1350** (**5.09×**):

| Stage | EQ p50 | Share | Lever |
|-------|-------:|------:|-------|
| embed | 2485 | 36% | C1c / provider |
| generate | 2316 | 34% | model / prefill |
| **rerank (CE)** | **1100** | **16%** | **C1a Fact skip** |
| retrieve | 487 | 7% | arm pool |

**Ceiling:** zero CE on all rows → ~4.3× still fails 1.5×. C1a is necessary, not sufficient.

---

## 2. Product vs Acc peer

| Path | Env | CE? | Use |
|------|-----|-----|-----|
| **Acc Fact peer** | `FACT_PROTECT_BM25=1` (`a1fp`) | Yes | Acc / Fact claims |
| **Product latency** | `FACT_CE_SKIP=1` + `FACT_RERANKER=bm25` (`c1a`) | Fact→BM25 | Ops / UX |

---

## 3. Measured (`T012849Z`)

| Metric | Acc Fact | C1a | Delta |
|--------|--------:|----:|------:|
| EQ Acc | 0.801 | 0.729 | Acc tax — do not promote |
| ctx_rel | 0.519 | 0.494 | below 0.50 |
| EQ p50 ms | 6876 | **6299** | −8% |
| EQ/LR p50 | 5.09× | **4.35×** | still ✗ 1.5× |
| rerank p50 factual (n=17) | ~CE | **9** | **−99%** |
| rerank p50 other (n=23) | ~CE | 1136 | unchanged |

**Law proof:** Fact CE-skip works. Overall Mix p50 still CE-dominated by non-Fact rows + embed/generate.

---

## 4. Implementation

| Piece | Location |
|-------|----------|
| Alias | `EDGEQUAKE_FACT_CE_SKIP=1` → `fact_bm25_rerank_enabled()` |
| Acc pin forward | `start_acc_backend.py` ACC_EXPORTS + override (`FACT_CE_SKIP`, `FACT_RERANKER`) |
| Ladder | `make bench001-c1a` |
| C1c | reuse `query_vec` when high/low == query |

---

## 5. Reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
export BENCH001_ACC_QUERY_CONCURRENCY=4
make bench001-c1a
# Confirm pins: fact_ce_skip=true, fact_reranker=bm25 in SUMMARY
```
