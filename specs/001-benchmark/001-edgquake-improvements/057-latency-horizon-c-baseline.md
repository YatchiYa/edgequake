# 057 — Latency Horizon C baseline (not Acc promote)

**Status:** Baseline locked · [058 C1a](./058-c1a-fact-ce-skip-latency.md) measured  
**Date:** 2026-07-21  
**Cross-ref:** [055](./055-post-acc-ceiling-first-principles.md) · [058](./058-c1a-fact-ce-skip-latency.md) · [013](./013-lens-latency-ops.md) · [peers.json](../e2e/artifacts/peers.json)

---

## 1. First principles

Latency = query-ops wall time. It does **not** gate Acc Beat. SLO: EQ/LR p50 ≤ **1.5×** under matched concurrency.

**Budget law (Acc Fact peer):** embed 36% + generate 34% + CE rerank 16% + retrieve 7%.  
Zeroing CE alone → ~4.3× — **fails** 1.5×. Must cut embed and/or generate next.

---

## 2. Baseline (frozen Acc fairness, n=40)

| Peer | EQ p50 / p95 | LR p50 / p95 | Ratio | Stage EQ p50 (ms) |
|------|-------------:|-------------:|------:|-------------------|
| Acc Fact [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) | 6876 / 9407 | 1350 / 1629 | **5.09×** ✗ | embed 2485 · retrieve 487 · rerank 1100 · generate 2316 |
| L2 Parity [`T093152Z`](../e2e/artifacts/history/smoke-20260720T093152Z/) | 7164 / 8885 | 1473 / 1925 | **4.86×** ✗ | embed 2444 · retrieve 954 · rerank 1092 · generate 2520 |
| **Latency C1a** [`T012849Z`](../e2e/artifacts/history/smoke-20260721T012849Z/) | 6299 / 8493 | 1449 / 1653 | **4.35×** ✗ | Fact rerank **9** · other ~1136 · Acc 0.729 tax |
| **Latency C1b** [`T013842Z`](../e2e/artifacts/history/smoke-20260721T013842Z/) | 5791 / 7336 | 1481 / 1904 | **3.91×** ✗ | keyword **1782** · embed 2212 · rerank **9** · gen **2421** · Acc 0.712 |
| **Latency C1d** [`T014632Z`](../e2e/artifacts/history/smoke-20260721T014632Z/) | 5995 / 7829 | 1468 / 1741 | **4.08×** ✗ | keyword **0** · gen **2985** · no wall win · Acc 0.736 |

LR Acc: `lr_enable_rerank=false`. EQ Acc peer pays labeled CE. Concurrency: EQ=4 / LR=2.

---

## 3. C1 experiments (one confound each)

| # | Change | Status / gate |
|---|--------|---------------|
| C1a | Fact CE-skip product (`FACT_CE_SKIP=1` / `c1a`) | **Measured** — Fact rerank 9ms; overall 4.35×; not Acc promote |
| C1b | BM25-all + keyword/embed timer split | **Measured** [059](./059-c1b-latency-ceiling-keyword-embed.md) — 3.91×; generate > 1.5× LR |
| C1c | Reuse `query_vec` when keywords == query (skip triple batch) | **Shipped** code |
| C1d | `KEYWORD_MODE=heuristic` on C1b | **Measured** [060](./060-c1d-heuristic-keyword-latency.md) — keyword 0; wall flat |

**Non-goals:** Soft Mix Acc fishing; claiming latency win without stage timers; replacing Acc Fact peer with `c1a`.

---

## 4. Reproduce measurement

```bash
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp   # Acc Fact peer
make bench001-c1a                                     # latency peer (not Acc promote)
```
