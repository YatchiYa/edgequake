# 11 — Code Smells & Multi-Lens Reassessment (v0.16.0)

**Supplements:** [05-MULTI-LENS-ASSESSMENT.md](./05-MULTI-LENS-ASSESSMENT.md)  
**Law:** Code is law — scores below reflect **shipped** EdgeQuake **v0.16.0** (OPS-P0–P3 + Science P4), not the 2026-07-09 assessment snapshot.

---

## Updated Capability Ladder (Retrieval Science)

```text
 1. Dense retrieval                 ✅
 2. Sparse + dense fusion (RRF)     ✅
 3. Entity/relation dual-level      ✅
 4. Difficulty-aware routing        ✅ (intent→mode + Mix/Hybrid arm gate)
 5. Associative graph walk (PPR)    ✅ default PPR (`bfs` escape)
 6. Dual-node phrase↔passage        ✅ bipartite PPR (entity∪chunk); AGE store still lite
 7. Path / flow pruning             ✅
 8. Community reports (optional)    ⚠️ opt-in extractive (LLM depth open)
 9. Full-link eval (graph→ret→gen)  ✅ ACC CI + mini corpus + LLM-judge opt-in
10. Ops: fail-loud + heal + OTel    ✅ P0–P3 complete
```

---

## Code Smell Register (Severity-Ordered) — v0.16.0

| ID | Smell | Evidence | Lens | Sev | Status |
|----|-------|----------|------|-----|--------|
| CS-01 | Silent Semantic→Recursive | `chunker/semantic.rs` | AI Eng | **P0** | **FIXED** fail-loud |
| CS-02 | Full-graph community load | `community.rs` | DB Exp | **P0** | **FIXED** `load_graph_bounded` |
| CS-03 | HNSW DDL `.ok()` swallow | `vector/ddl.rs` | DB Exp | **P0** | **FIXED** + readiness |
| CS-04 | retry-chunks stub | `recovery/chunks.rs` | SRE | **P0** | **FIXED** + graph merge |
| CS-05 | Mix/Hybrid always 3 arms | `mix.rs` / `hybrid.rs` | AI Eng | **P1** | **FIXED** intent gate |
| CS-06 | Unbounded gleaning + high extract concurrency | API admission | AI Eng | **P1** | **FIXED** ≤2 / ≤32 |
| CS-07 | Silent embedding truncation | `helpers/embeddings.rs` | AI Eng | **P1** | **FIXED** policy flag |
| CS-08 | KV before merge → orphan risk | `ingestion_persister.rs` | SRE | **P1** | **FIXED** KV compensate |
| CS-09 | Graph quality not in Prometheus | `graph_metrics.rs` | SRE | **P1** | **FIXED** `record_graph_quality` |
| CS-10 | QueryStats missing arm metrics | `types.rs` QueryStats | SRE | **P1** | **FIXED** arm_* fields |
| CS-11 | No OTel GenAI / rag.* attrs | `rag_span.rs` | SRE | **P1** | **FIXED** helpers + arm wiring |
| CS-12 | PPR default off | `parse_graph_walk_mode` | AI Eng | **P1** | **FIXED** PPR default + ACC |
| CS-13 | iterative_scan `strict_order` only | `search_tuning.rs` | DB Exp | **P1** | **FIXED** relaxed default |
| CS-14 | Readiness blockers incomplete vs is_ready | `migration_bootstrap` | SRE | **P1** | **FIXED** SSOT |
| CS-15 | Popular-node fallback silent | `local.rs` / `global.rs` | SRE | **P2** | **FIXED** telemetry |
| CS-16 | FTS fallback invisible | `sparse_retrieval.rs` | SRE | **P2** | **FIXED** outcome |
| CS-17 | RlsContext still exported | `postgres/mod.rs` | SRE | **P2** | **FIXED** unexported |
| CS-18 | Full HF GraphRAG-Bench ACC | `eval/graphrag_corpus.rs` | AI Eng | **P2** | **OPEN** mini corpus only |
| CS-19 | True cross-encoder rerank | `bootstrap.rs` | AI Eng | **P2** | **OPEN** BM25 path |
| CS-20 | Density YAML / LLM community depth | various | AI Eng | **P2** | **OPEN** |

---

## Lens Scorecard (v0.16.0 — replaces pre-ship tables)

### Database Expert

| Criterion | Score /5 | Note |
|-----------|:--------:|------|
| ANN correctness under filters | **4.5** | iterative_scan relaxed; fail-closed `/ready` |
| AGE index hygiene | **4.5** | `ensure_indexes`; community bounded |
| Migration safety | **4.0** | Reconcile + checksum repair; forward-only + PITR law |
| Fail-closed readiness | **4.5** | `readiness_blockers` SSOT incl. HNSW |
| Multi-major (16/17/18) | **4.5** | extension-pins + `ops17-smoke` |
| **Overall DB** | **4.4** | Substrate + fail-closed shipped |

### AI Engineer

| Criterion | Score /5 | Note |
|-----------|:--------:|------|
| Hybrid retrieval physics | **4.6** | Mix+RRF+BM25+PPR bipartite+prune |
| Router / graph tax control | **4.5** | Factual→Naive; arm gate |
| Extraction quality controls | **4.0** | Gleaning/concurrency clamps; truncate policy |
| Eval / faithfulness | **4.0** | ACC CI + mini corpus + LLM-judge opt-in |
| Observability for tuning | **4.5** | Arm timings + rag spans + graph gauges |
| **Overall AI** | **4.3** | Brain + eyes; HF/CE deferred |

### SRE / Reliability

| Criterion | Score /5 | Note |
|-----------|:--------:|------|
| Saga / compensation | **4.0** | KV + merge compensate; quarantine metrics |
| Auto-repair coverage | **4.0** | Inspector + failed_chunks retry→merge |
| Corruption / checksum story | **3.5** | PG18 + app structural; PITR process |
| Tracing depth | **4.5** | Prometheus + rag spans (OTLP opt-in) |
| Runbooks as code | **4.0** | [13-OPS-RUNBOOKS.md](./13-OPS-RUNBOOKS.md) + `/ready` |
| **Overall SRE** | **4.0** | P0 holes closed |

---

## Honesty Update vs 05 (2026-07-09 → v0.16.0)

| Statement in 05 (2026-07-09) | Update (v0.16.0) |
|------------------------------|------------------|
| "Observability Medium; missing graph quality metrics" | **High** — Prometheus graph quality + arm timings + rag spans |
| "Resume after crash Medium" | **Improved** — retry-chunks + merge; fingerprint stale purge |
| Science ladder rung 4–6 | Rungs **4–7 + 9–10 shipped**; rung 8 LLM depth open |
| "Would I bet production" enterprise Yes | **Yes** for mid/large workspaces with `/ready` green; measure 100k+ separately |
| No PPR / dual-node | **PPR default + bipartite pick** |
| No GraphRAG-Bench harness | **ACC CI + mini corpus** (full HF download deferred) |

---

## Anti-Patterns Explicitly Rejected

| Anti-pattern | Why rejected |
|--------------|--------------|
| Multi-DB (Neo4j + Milvus + PG) | Breaks single-engine consistency moat |
| Fake 2PC across AGE+vector | Latency + deadlock; keep saga + measure drift |
| Full Louvain every ingest | O(V+E) tax; use incremental/sample |
| Cargo-cult `ef_construction=64` without measure | SPEC-034 already measured 32; re-validate don't guess |
| Logging-only "repair" APIs | Stubs that return 200 with `implemented:false` erode trust |

---

## Definition of Done for "Ops-Complete Hybrid RAG" (v0.16.0)

1. No silent strategy/index downgrades (CS-01, CS-03) — ✅  
2. No unbounded `get_all_nodes` on ingest path (CS-02) — ✅  
3. failed_chunks write + list + retry + graph merge (CS-04) — ✅  
4. QueryStats arms + fallbacks + faithfulness + OTel rag + graph Prometheus — ✅  
5. `/ready` fails closed on missing ANN + critical migrations SSOT — ✅  
6. PG pin matrix smoke (OPS-17) — ✅  
7. ACC CI + PPR default (Science P4) — ✅  

**Post-0.16 open:** CS-18…20 (HF corpus, cross-encoder, density/LLM community depth).
