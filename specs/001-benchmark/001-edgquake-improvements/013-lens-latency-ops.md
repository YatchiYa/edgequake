# 013 — Lens: Latency & Ops (P3)

**Priority:** P3 — product parity after quality gates  
**Cross-ref:** [010 Retrieval noise](./010-lens-retrieval-noise.md) · [010 Runbook](../010-smoke-then-core-runbook.md)

---

## 1. Observation

| Metric | EQ | LR | Gap |
|--------|----|----|-----|
| Query p50 | ~9.6 s | ~3.0 s | **~3×** |
| Query p95 | ~12.9 s | ~6.8 s | ~2× |
| Ingest wall (full medical) | ~637 s (~10.6 min) | parallel / cached path | — |

Quality can be tied while UX loses. Compact context (P0) often **reduces** prefill latency — quality and speed are aligned if prune works.

---

## 2. First-principles diagnosis

- Mix runs three arms (local, global, naive) plus keyword LLM work, graph expand, embed calls, optional rerank.
- **Arms are already parallel** (`tokio::join!` + `EDGEQUAKE_QUERY_ARM_CONCURRENCY`); keyword extraction is cached. Remaining wall time is CE HTTP, large prefill, arm-semaphore contention under Acc concurrency, and generation.
- HippoRAG2 lesson: compact ~1k-token contexts beat LightRAG-class 10⁴–10⁵ token dumps on cost/latency.
- Post-E4 program: [021 F3](./021-grounded-improvement-plan.md).

---

## 3. July 2026 practice

- Parallelize independent retrieval arms; cache query embeddings and keyword extraction.
- Rerank top-50 → top-5–8 in ~30–100 ms — acceptable if it cuts generation prefill more than it costs.
- Agentic skip-retrieval for non-corpus questions (product path) — not Acc (Acc always retrieves).
- Instrument p50/p95 per stage (retrieve vs generate) in scorecard / LIVE progress.

---

## 4. EQ insertion points

| Area                    | Likely site                                                      | Action                                              | Status                                             |
| -------------------------| ------------------------------------------------------------------| -----------------------------------------------------| ----------------------------------------------------|
| Arm concurrency         | `modes/mix.rs` + `arm_timed.rs`                                  | Parallel local/global/naive with bounded join       | **Shipped**                                        |
| Keyword cache           | `CachedKeywordExtractor`                                         | Process LRU + TTL; prepare once per query           | **Shipped**                                        |
| Token budget            | `truncation.rs` `balance_context`                                | F1 Summarize floor + F3 compact prefill             | Active ([021](./021-grounded-improvement-plan.md)) |
| Rerank cost             | `reranking.rs`                                                   | Prefer local CE or fast API; record ms in stats     | Acc CE labeled; product BM25                       |
| Acc harness concurrency | `BENCH001_QUERY_CONCURRENCY` + `EDGEQUAKE_QUERY_ARM_CONCURRENCY` | Fair vs LR; arm pool ≥ 3× query concurrency         | F3a                                                |
| Stage timing            | Acc predictions / SUMMARY                                        | Surface retrieve / rerank / generate / arm_ms       | F3a                                                |
| Backend lifecycle       | Acc detached backend / Postgres                                  | Warm indexes; `ORPHAN_RETRACT_ON_RECOVER=0` for Acc | Ops                                                |

---

## 5. Experiments (one confound each)

| # | Change | Success | Status |
|---|--------|---------|--------|
| L1 | Parallel Mix arms only | EQ p50 ≤ **1.5×** LR under same concurrency; Acc/L2 unchanged within noise | **Shipped** (remeasure under F3a) |
| L2 | Keyword extraction cache | Keyword stage p50↓ ≥ 30%; quality flat | **Shipped** |
| L3 | Post-P0 tighter token budget | p50↓; ctx_rel ≥ 0.50 maintained | F1 + F3b ([021](./021-grounded-improvement-plan.md)) |
| L4 | Stage timing export in predictions/meta | Retrieve vs generate split visible in SUMMARY | F3a |

**SLO (product claim):** EQ query p50 ≤ 1.5× LR under matched concurrency and pins before claiming product latency parity.

---

## 6. Horizon C deferral (028) — not an Acc CI blocker

**028 C1** keeps latency as a **parallel track** beside Horizon A Acc ablations:

| Track | Goal | Blocks Acc Beat/Parity CI? |
|-------|------|----------------------------|
| Horizon A (query/prompt) | Complex Acc + L2 gates | **Yes** |
| Horizon C (ops) | EQ/LR p50 ≤ **1.5×** | **No** |

Suggested C1 levers (still one confound each, outside Acc promote):

1. Fact / closed-intent **CE skip** — [058](./058-c1a-fact-ce-skip-latency.md) `FACT_CE_SKIP=1` / `make bench001-c1a` (**shipped**; Acc peer keeps CE).  
2. Raise `EDGEQUAKE_QUERY_ARM_CONCURRENCY` / match EQ↔LR query concurrency (C1b).  
3. Embed reuse when keywords == query (C1c **shipped** in `compute_with_query_vec`); cache hit-rate under Acc concurrency.

Do **not** hold A4 Acc CI on latency. Publish latency separately under this lens after Parity/Beat quality gates.

---

## 7. Non-goals

- Do not disable Mix arms or graph retrieval solely for speed on Acc headline.
- Do not raise EQ concurrency while leaving LR sequential and claim a fair latency win.
- Do not skip L2 quality gates to ship a “fast” noisy retriever.
- Ingest wall is secondary for Acc smoke; optimize ingest under a separate ops epic.
- Do not treat p50 ≤ 1.5× as a Beat Acc CI gate ([028](./028-first-principles-beat-roadmap.md)).
