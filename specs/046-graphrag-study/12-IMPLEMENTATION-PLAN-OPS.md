# 12 — Implementation Plan: Ops · Storage · Defense · Observability

**Date:** 2026-07-10 (updated after OPS-P3 implementation)  
**Goal:** Make EdgeQuake **fail-closed, O(N)-safe, healable, and observable** on PG16/17/18 + pgvector 0.8.3 + AGE 1.6/1.7 — without abandoning the Mix spine.  
**Law:** Every ticket names files/symbols. No vapor.

**Companion science plan:** [07-IMPROVEMENT-PLAN.md](./07-IMPROVEMENT-PLAN.md) (retrieval physics).  
**This plan owns:** defense, migration, auto-repair, tracing/metrics, Postgres performance.  
**Runbooks:** [13-OPS-RUNBOOKS.md](./13-OPS-RUNBOOKS.md).

---

## Implementation status (2026-07-10)

| Ticket | Status | Evidence |
|--------|--------|----------|
| EQ-046-OPS-01…10,12,13,21 | **DONE** | See OPS-P0 / OPS-P1 sections |
| EQ-046-OPS-11 | **DONE** | `rag_span.rs` GenAI retrieval/generation spans + attrs |
| EQ-046-OPS-14 | **DONE** | `mark_popular_node_fallback` + QueryStats + Prometheus |
| EQ-046-OPS-15 | **DONE** | `SparseRetrievalOutcome` + `fts_fallback` in QueryStats |
| EQ-046-OPS-16 | **DONE** | `RlsContext` removed from `postgres::` re-exports |
| EQ-046-OPS-17 | **DONE** | `run_ops17_perf_smoke.sh` + nightly workflow + `make ops17-smoke` |
| EQ-046-OPS-18 | **DONE** | [13-OPS-RUNBOOKS.md](./13-OPS-RUNBOOKS.md) |
| EQ-046-OPS-19 | **DONE** | `record_storage_drift` / `set_storage_drift_critical` from inspector |
| EQ-046-OPS-20 | **DONE** | `eval/faithfulness.rs` heuristic sampler + QueryStats field |
| EQ-046-OPS-22 | **DONE** | LLM-judge + ACC harness (`faithfulness_judge.rs`, `acc_harness.rs`) |
| EQ-046-OPS-23 | **DONE** | `record_graph_quality` from `log_graph_quality` (observability feature) |
| EQ-046-OPS-24 | **DONE** | `run_arm_timed` + `pipeline_retrieve` GenAI retrieval spans |
| EQ-046-OPS-09 | **DONE** | Graph quality now Prometheus gauges (was tracing-first) |

**Tests run (non-flaky):**
- query lib: PPR parse default, ACC harness, judge parsers, MockProvider judge
- query e2e: `e2e_spec046_ops_p3_acc`, `e2e_spec046_hybrid_rag` (+ ACC)
- storage: `graph_metrics` (observability feature)
- observability: `rag_span`
- **live (opt-in):** `e2e_ops_p3_mistral_small_embed_faithfulness_live` — mistral-small-latest + mistral-embed ✅

---

## North Star Metrics (Ops)

| Metric | Baseline | Now | Target |
|--------|----------|-----|--------|
| Silent strategy downgrades | Warn-only | **Fail-loud** | 0 unflagged |
| Community refresh | Full scan | **Bounded** | p95 < 30s @ 100k |
| ANN missing | Possible | **`/ready` 503** | same |
| retry-chunks | stub | **extract + merge** | ACC on retry |
| Cross-store orphans | Inspector | **KV+vector+graph compensate** | SAFE heal |
| Query arm visibility | Aggregate | **Per-arm + gated + spans** | dashboards |
| OTel GenAI spans | Off | **Wired on arms + single modes** | opt-in OTLP |
| Popular-node / FTS fallback | Debug logs | **QueryStats + Prometheus** | alerts |
| Drift SLO | Logs only | **drift_* metrics** | page on critical |
| Faithfulness | Offline only | **Heuristic + LLM-judge opt-in** | ACC CI ✅ |
| Graph quality | Tracing | **Prometheus gauges** | alert on sparse |
| Graph walk default | BFS | **PPR** (`bfs` escape) | ACC-gated ✅ |
| PG matrix CI | Images | **Nightly pin smoke** | battle on schedule |

---

## Phase Map

```text
OPS-P0 (DONE)  Fail-loud + O(N) kill + chunk retry
OPS-P1 (DONE)  Arm gate + clamps + KV compensate + readiness SSOT + merge-on-retry
OPS-P2 (DONE)  OTel rag spans + fallback telemetry + RlsContext unexport + PG smoke + runbooks + drift + faithfulness
OPS-P3 (DONE)  LLM-judge faithfulness; ACC CI gate; PPR default; graph-quality Prometheus; arm span wiring
```

---

## OPS-P3 — COMPLETED

### OPS-24 Wire rag_span on arm hot paths — DONE
- `modes/arm_timed.rs` — `with_rag_retrieval_span` + `record_rag_retrieval_outcome` per Mix/Hybrid arm
- `query_pipeline.rs` `pipeline_retrieve` — top-level span for Local/Global/Naive (Mix/Hybrid avoid double-nest)

### OPS-23 Graph quality → Prometheus — DONE
- `edgequake-observability::record_graph_quality` gauges (`nodes`, `edges`, `avg_degree`, `orphan_rate`, `empty_description_rate`, `sparse`)
- `log_graph_quality` emits Prometheus when storage `observability` feature on

### OPS-22 LLM-judge + ACC CI — DONE
- `eval/faithfulness_judge.rs` — `EDGEQUAKE_FAITHFULNESS_JUDGE`; pure parsers; MockProvider unit tests
- `eval/acc_harness.rs` — `run_spec046_acc_report` (routing + PPR default + prune + truncation + heuristic floor)
- Pipeline: judge preferred when enabled, else heuristic sampler
- E2E: `tests/e2e_spec046_ops_p3_acc.rs` (+ ignored Mistral live)

### PPR default (science/ops bridge) — DONE
- `parse_graph_walk_mode` / `GraphWalkMode::default()` → **Ppr**; `EDGEQUAKE_GRAPH_WALK=bfs` escape
- ACC check `graph_walk_default_ppr` gates the flip

---

## Ticket Backlog (final)

```text
[x] EQ-046-OPS-01 … OPS-10,12,13,21   (P0+P1)
[x] EQ-046-OPS-11  OTel GenAI + rag.* spans
[x] EQ-046-OPS-14  Popular-node fallback telemetry
[x] EQ-046-OPS-15  FTS fallback flag in QueryStats
[x] EQ-046-OPS-16  Remove RlsContext from postgres:: exports
[x] EQ-046-OPS-17  PG16/17/18 pin smoke + nightly workflow
[x] EQ-046-OPS-18  Ops runbooks
[x] EQ-046-OPS-19  Drift SLO metrics
[x] EQ-046-OPS-20  Online faithfulness sampler (heuristic)
[x] EQ-046-OPS-22  LLM-judge faithfulness + ACC CI
[x] EQ-046-OPS-23  Graph quality → Prometheus
[x] EQ-046-OPS-24  Wire rag_span into every mode arm hot path
```

---

## Success Definition

1. **Fail-closed** Semantic/ANN — ✅  
2. **O(N)-safe** community — ✅  
3. **Healable** chunk retry + merge — ✅  
4. **Observable** arms + spans + fallbacks + drift + faithfulness + graph quality — ✅  
5. **Multi-major** pin smoke — ✅  
6. **ACC CI** deterministic harness — ✅  
7. **PPR default** with BFS escape — ✅  

**Honest label:** production Hybrid RAG with fail-closed ops substrate, intent-gated Mix/Hybrid, PPR-default graph walk, GenAI retrieval spans on arms, graph-quality Prometheus, and ACC + optional LLM-judge faithfulness (Mistral live verified).

**Science P4 bridge (tracked in [07](./07-IMPROVEMENT-PLAN.md)):** EQ-046-16…18 — `make spec046-acc` + `.github/workflows/spec046-acc.yml` JSON artifact; bipartite dual-node PPR on Mix/Local/Global chunk pick; GraphRAG-Bench-style mini corpus ACC. Remaining: full HF corpus download ACC, true cross-encoder, density YAML, LLM community reports.
