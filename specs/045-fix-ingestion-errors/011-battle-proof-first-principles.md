# SPEC-045 — Battle-Proof First Principles Assessment

**Purpose:** Define what "battle-proof" means for EdgeQuake and map every invariant to code evidence + gap.  
**Cross-ref:** [010-sre-engineering-review](./010-sre-engineering-review.md) · [002-first-principles](./002-first-principles.md)

---

## First principle: battle-proof definition

> A system is **battle-proof** when every failure mode has exactly one of: **auto-heal**, **fail-closed gate**, or **actionable operator signal** — with **no silent divergence** between ingestion, query, and storage layers.

---

## Core invariants (extended from 002-first-principles)

| ID | Invariant | Ingestion | Migration | Query | Battle-proof? |
| -- | --------- | --------- | --------- | ----- | ------------- |
| **I1** | Content extracted before persist | ✅ resilient extract | N/A | N/A | ✅ |
| **I2** | Embedding dim == table dim | ⚠️ no evict retry | ⚠️ M080 silent | ✅ OODA-225 | **❌ SPLIT** |
| **I3** | Graph merged; errors == 0 | ✅ compensate saga | N/A | N/A | ✅ |
| **I4** | Metadata consistent (KV/wsdoc/PG) | ✅ write-path + M047 | ⚠️ M040 lag | N/A | **⚠️** |
| **I5** | Task terminal ↔ doc terminal | ✅ startup | N/A | N/A | **❌ runtime** |
| **I6** | Traffic blocked when unsafe | N/A | ✅ M038/M042 | N/A | **⚠️ partial** |
| **I7** | Failure is typed + actionable | ✅ ingestion_reliability | N/A | ❌ none | **❌ query** |
| **I8** | Retries bounded and purposeful | ✅ permanent fast-fail | N/A | ⚠️ partial | **⚠️** |
| **I9** | Observability → SLO/alert | ⚠️ coarse metrics | ⚠️ health only | ⚠️ audit only | **❌** |
| **I10** | Idempotent at every storage write | ✅ checkpoints/upsert | ✅ reconcile | ✅ cache inv | **✅** |

**Score: 4/10 invariants fully battle-proof across all three pipelines.**

---

## DRY violations (cross-pipeline)

| Duplication | Path A | Path B | Fix |
| ----------- | ------ | ------ | --- |
| Vector dimension heal | `handlers/query/workspace_resolve.rs` | `workspace_vector_resolve.rs` (ingest) | Extract `VectorRegistryResolve` SSOT in `edgequake-core` |
| Failure taxonomy | `ingestion_reliability.rs` | (none for query) | Add `query_reliability.rs` mirroring pattern |
| Orphan recovery | `recover_orphaned_tasks` (startup) | `periodic_orphan_check` (runtime) | Shared `OrphanRecoveryService` with doc sync |
| Timeout policy | PDF `processing_timeout_secs` | Text global 7200s | `LargeDocumentProfile` for all task types |

---

## SOLID violations (SRE lens)

| Principle | Violation | Evidence |
| --------- | --------- | -------- |
| **SRP** | `main.rs` owns startup recovery + periodic repair + server config | 800+ lines mixing concerns |
| **OCP** | New migration reconcile requires editing `mod.rs` orchestrator (~1600 LOC) | `migration_bootstrap/mod.rs` |
| **LSP** | Query and ingest both "resolve vector storage" but different contracts | Asymmetric heal |
| **ISP** | Health endpoint mixes liveness, readiness, migration, schema | `health.rs` monolith |
| **DIP** | Worker calls `TaskFailureInfo::from_processing_error` directly | OK — but no injectable classifier for tests at scale |

**Recommendation:** Extract `IngestionReliabilityController` (006-bulletproof) **and** `VectorStorageResolver` trait — not new abstractions for their own sake, but to close proven split-brain.

---

## Auto-migration battle-proof matrix

```
  Migration × Failure mode × Auto-heal
  ┌────────┬─────────────────────┬──────────────┬─────────────┐
  │ Mig    │ Failure             │ Auto-heal?   │ Gate?       │
  ├────────┼─────────────────────┼──────────────┼─────────────┤
  │ M038   │ Missing indexes     │ Inline/defer │ /ready 503  │ ✅
  │ M042   │ pgvector < 0.8      │ apply.sql    │ /ready 503  │ ✅
  │ M047   │ wsdoc gap           │ Every boot   │ None        │ ⚠️
  │ M080   │ halfvec mismatch    │ apply.sql    │ None        │ ❌
  │ M040   │ CQRS lag            │ Background   │ None        │ ⚠️
  │ M046   │ Slow scoped merge   │ Every boot   │ None        │ ⚠️
  │ M081   │ AGE RLS missing     │ If AGE≥1.7   │ None        │ ⚠️
  └────────┴─────────────────────┴──────────────┴─────────────┘
```

**First principle gap:** Only 2 of 7 migration risks block traffic. Post-migration ingest/query failures from M080/M046/M040 are **silent degradation**, not fail-closed.

---

## Query pipeline battle-proof matrix

| Failure | Auto-heal | Typed class | Operator signal |
| ------- | --------- | ----------- | --------------- |
| Dimension mismatch | ✅ evict+retry | ❌ | Generic 500 |
| Empty vector store | ❌ | ❌ | Empty results (silent) |
| Graph unavailable | ❌ | ❌ | 502 opaque |
| LLM auth fail | ✅ fallback LLM | ❌ | Generic error |
| Rate limit | ⚠️ 429 if enabled | ❌ | Off by default |
| Context too long | ❌ | ❌ | Truncation opaque |

---

## What we missed in SPEC-045 (original scope)

The original SPEC-045 focused on **ingestion failures after migration**. SRE review reveals **three missed dimensions**:

### 1. Cross-pipeline SSOT (biggest miss)

Ingestion and query use **different vector resolution paths** with different self-healing. Post-migration provider switches are the #1 production trigger — query "works" while ingest fails.

### 2. Runtime vs startup recovery asymmetry

Startup recovery is excellent (paginated, ordered, documented). **Runtime** `periodic_orphan_check` creates task/doc split-brain — not covered in original edge-case matrix.

### 3. Migration visibility beyond M038/M042

Auto-migration is robust at boot but **opaque at runtime**: M080, M040, M046 progress invisible; `/ready` doesn't name blockers.

### 4. Query as equal citizen

No `QueryFailureClass`, no SLO hooks, no post-migration query smoke in battle tests.

### 5. Observability → alerting gap

`failure_class` in KV metadata helps operators manually — but **no Prometheus counter** means no paging, no SLO dashboards.

---

## Battle-proof target state

```
┌─────────────────────────────────────────────────────────────┐
│                    EdgeQuake Reliability Plane              │
├─────────────────────────────────────────────────────────────┤
│  VectorStorageResolver (SSOT)                               │
│    ├── ingest path                                          │
│    ├── query path                                           │
│    └── post-M080 cache invalidation                         │
├─────────────────────────────────────────────────────────────┤
│  Failure Taxonomy                                           │
│    ├── IngestionFailureClass (✅ shipped)                   │
│    └── QueryFailureClass (❌ needed)                        │
├─────────────────────────────────────────────────────────────┤
│  OrphanRecoveryService                                      │
│    ├── startup (✅)                                         │
│    ├── periodic task (⚠️ doc sync missing)                  │
│    └── periodic document (✅ env-gated)                     │
├─────────────────────────────────────────────────────────────┤
│  MigrationHealth                                            │
│    ├── /ready JSON blockers (❌)                            │
│    ├── M080/M081 in report (❌)                             │
│    └── periodic re-audit (❌)                               │
├─────────────────────────────────────────────────────────────┤
│  Metrics/SLO                                                │
│    ├── failure_class counters (❌)                          │
│    ├── quarantine counter (❌)                              │
│    └── time-to-indexed histogram (❌)                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Acceptance criteria for "battle-proof" release

- [ ] SRE-Q02: Ingest dimension heal parity with query
- [ ] SRE-I01: No task/doc split-brain after periodic orphan
- [ ] SRE-I02: Pending requeue paginated
- [ ] SRE-M05: M080 triggers cache clear + health flag
- [ ] SRE-I06: Prometheus `ingestion_failures_total{failure_class}`
- [ ] SRE-Q01: QueryFailureClass on all query 4xx/5xx
- [ ] SRE-M03: `/ready` returns JSON blockers
- [ ] Battle tests: cross-pipeline vector resolve contract test
- [ ] `make spec045-battle-test-all` in CI (no continue-on-error)
