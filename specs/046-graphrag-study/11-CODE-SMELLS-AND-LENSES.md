# 11 — Code Smells & Multi-Lens Reassessment (2026-07-10)

**Supplements:** [05-MULTI-LENS-ASSESSMENT.md](./05-MULTI-LENS-ASSESSMENT.md)  
**Does not replace 05** — updates science/ops scores after EQ-046 lite + this ops deep study.

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

## Code Smell Register (Severity-Ordered) — updated 2026-07-10 post Science P4

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
| CS-12 | PPR default off | `GraphWalkMode::from_env` | AI Eng | **P1** | **FIXED** PPR default + ACC |
| CS-13 | iterative_scan `strict_order` only | `search_tuning.rs` | DB Exp | **P1** | **FIXED** relaxed default |
| CS-14 | Readiness blockers incomplete vs is_ready | `migration_bootstrap` | SRE | **P1** | **FIXED** SSOT |
| CS-15 | Popular-node fallback silent | `local.rs` / `global.rs` | SRE | **P2** | **FIXED** telemetry |
| CS-16 | FTS fallback invisible | `sparse_retrieval.rs` | SRE | **P2** | **FIXED** outcome |
| CS-17 | RlsContext still exported | `postgres/mod.rs` | SRE | **P2** | **FIXED** unexported |
| CS-18–19 | Naming / other deprecations | various | — | **P2** | OPEN |

## Lens Scorecard Update (post Science P4)

| Lens | Was | Now | Note |
|------|:---:|:---:|------|
| Database Expert | 4.5 | **4.5** | Unchanged (storage still sources edges from retrieved rels) |
| AI Engineer | 4.4 | **4.6** | Bipartite dual-node + mini corpus ACC |
| SRE | 4.7 | **4.8** | `make spec046-acc` + GH workflow JSON artifact |

---

## Lens Scorecard Update

### Database Expert

| Criterion | Score /5 | Note |
|-----------|:--------:|------|
| ANN correctness under filters | 4 | iterative_scan present; strict_order TBD |
| AGE index hygiene | 4 | Strong ensure_indexes; community O(N) hurts |
| Migration safety | 3.5 | Reconcile excellent; no down-migrate; stub heal |
| Fail-closed readiness | 2.5 | HNSW swallow + incomplete blockers |
| Multi-major (16/17/18) | 4.5 | extension-pins SSOT |
| **Overall DB** | **3.7** | Substrate strong; fail-closed weak |

### AI Engineer

| Criterion | Score /5 | Note |
|-----------|:--------:|------|
| Hybrid retrieval physics | 4.5 | Mix+RRF+BM25+prune |
| Router / graph tax control | 4 | Adaptive on; explicit Mix tax remains |
| Extraction quality controls | 3 | Gleaning unbounded; silent trunc |
| Eval / faithfulness | 2 | Synthetic bench; no CI ACC |
| Observability for tuning | 2.5 | Coarse metrics |
| **Overall AI** | **3.4** | Brain good; eyes weak |

### SRE / Reliability

| Criterion | Score /5 | Note |
|-----------|:--------:|------|
| Saga / compensation | 3.5 | Best-effort; quarantine metric exists |
| Auto-repair coverage | 3 | Inspector good; chunk retry stub |
| Corruption / checksum story | 3 | PG18 helps; app-level structural only |
| Tracing depth | 2.5 | Prometheus yes; GenAI spans no |
| Runbooks as code | 3.5 | /health rich; /ready partial |
| **Overall SRE** | **3.1** | Skeleton present; holes at P0 |

---

## Honesty Update vs 05

| Statement in 05 (2026-07-09) | Update (2026-07-10) |
|------------------------------|---------------------|
| "Observability Medium; missing graph quality metrics" | Metrics **collected** but **not Prometheus**; still Medium→Low for ops dashboards |
| "Resume after crash Medium" | Still true; **retry-chunks stub** confirms |
| Science ladder rung 4–6 | Rung 4–7 **lite shipped**; rung 10 (ops) now explicit gap |
| "Would I bet production" enterprise Yes | **Conditional Yes**: OK for mid-size workspaces; **No** for huge graphs until CS-02/CS-03 fixed |

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

## Definition of Done for "Ops-Complete Hybrid RAG"

1. No silent strategy/index downgrades (CS-01, CS-03) — ✅  
2. No unbounded `get_all_nodes` on ingest path (CS-02) — ✅  
3. failed_chunks write + list + retry + graph merge (CS-04, OPS-21) — ✅  
4. QueryStats arms + fallbacks + faithfulness (CS-10,15,16,20) — ✅; OTel rag helpers (CS-11) — ✅; graph-quality Prometheus (CS-09) — PARTIAL  
5. `/ready` fails closed on missing ANN + critical migrations SSOT (CS-03, CS-14) — ✅  
6. PG pin matrix smoke (OPS-17) — ✅; Nightly ACC / PPR default (science 07) — OPEN
