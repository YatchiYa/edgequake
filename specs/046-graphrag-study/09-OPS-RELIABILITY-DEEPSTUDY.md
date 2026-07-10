# 09 — Ops Reliability Deep Study (Defense / Migration / Auto-Repair / Observability)

**Date:** 2026-07-10  
**Lenses:** Database Expert · AI Engineer · SRE  
**Law:** Every claim maps to a path/symbol. Specs are hypotheses; code is law.

---

## First Principles (Ops)

### O1 — Consistency is a budget, not a slogan

```text
maximize  P(answer correct | context)
subject to  tokens ≤ B, latency ≤ L, cost ≤ C
AND         P(cross-store drift) ≤ ε
AND         P(silent degradation) ≈ 0
```

Hybrid RAG on Postgres (KV + pgvector + AGE) is a **multi-store system inside one engine**.  
A single `BEGIN` across all three is not free (AGE Cypher + HNSW + KV). EdgeQuake correctly uses **saga + compensation**, not fake 2PC. That is acceptable **only if** drift is measured and auto-repaired.

### O2 — Silent failure is worse than loud failure

July 2026 RAG ops consensus (OpenTelemetry GenAI + production RAG guides):

| Silent failure | Why it kills trust |
|----------------|--------------------|
| Empty retrieval | Answer looks fluent, is ungrounded |
| Context truncation | Faithfulness collapses without error |
| Strategy downgrade (Semantic→Recursive) | Chunk quality ceiling drops invisibly |
| HNSW index missing | Seq-scan latency / recall cliff |
| Stub repair API | Operators think heal worked |

### O3 — Migration is a product feature

Version upgrades (PG16→17→18, AGE 1.6→1.7, pgvector 0.8.x, sqlx schema) are **production events**.  
Checksum mismatch, CONCURRENT index builds, and AGE upgrade scripts that create indexes on large graphs are first-class failure modes (AGE 1.7.0 release notes warn upgrade may take a long time).

### O4 — Auto-repair must be tiered

```text
SAFE (auto)     → orphan vector delete, metadata id repair, INVALID index drop
CAUTION (admin) → stuck PDF reset, workspace table drop, AGE resync
MANUAL          → full graph rebuild, major PG upgrade, checksum surgery
```

---

## What Exists Today (Code-is-Law)

### A. Defense in depth

| Layer | Mechanism | Path / Symbol |
|-------|-----------|---------------|
| Admission hash | Content hash + workspace UNIQUE | `migrations/023_workspace_scoped_content_hash.sql` |
| PDF checksum | SHA-256 dedup | `pdf_storage.rs` `calculate_pdf_checksum` |
| Saga compensation | Orphan vector/graph cleanup | `compensation.rs` `compensate_merge_failure` |
| Persist order | KV → vectors → graph merge | `ingestion_persister.rs` `persist_processing_result_impl` |
| Process fingerprint | Stale extract purge on option change | `process_fingerprint.rs` |
| StorageInspector | INV-01..05 + repair tiers | `storage_inspector.rs` |
| Hourly monitor | `spawn_hourly_monitor` | `storage_inspector.rs` |
| Admin API | inspect / repair (dry_run default) | `handlers/admin.rs` |
| RLS (relational) | `set_tenant_context` | `migrations/009_add_rls_policies.sql`, `rls.rs` |
| AGE RLS (opt-in) | AGE ≥1.7 + `EDGEQUAKE_AGE_RLS` | `migrations/support/081/apply.sql` |

### B. Schema migration

| Mechanism | Path / Symbol |
|-----------|---------------|
| sqlx Migrator | `migration_bootstrap/mod.rs` `MIGRATOR`, `run_postgres_migrations` |
| Version table | `_sqlx_migrations` |
| Marker + reconcile | `migrations/0NN_*_marker.sql` + `migrations/support/NNN/apply.sql` |
| Checksum repair | `reconcile/m071.rs`, `reconcile/m078.rs` |
| Readiness gate | `is_ready_for_traffic`, `readiness_blockers` → `/ready` 503 |
| Health snapshot | `health_types.rs` `MigrationHealthSnapshot` |
| Triple-track PG | `docker/extension-pins.sh` pg16/pg17/pg18 |

**Upgrade flow (law):**

```text
Startup → repair M071/M078 checksums
       → MIGRATOR.run() (forward-only)
       → reconcile M038…M083 (indexes, halfvec, AGE RLS, UNIQUE)
       → MigrationBootstrapReport → /health + /ready
```

### C. Auto-repair inventory

| Feature | Status | Symbol |
|---------|--------|--------|
| Orphan vector delete | ✅ SAFE auto | `RepairAction::DeleteOrphanedVectors` |
| Resync entities from AGE | ✅ CAUTION | `ResyncEntitiesFromAge` |
| Rematerialize vector columns | ✅ | `RematerializeVectorColumns` |
| Reset stuck PDFs | ✅ CAUTION | `ResetStuckPdfs` |
| INVALID AGE index repair | ✅ boot | `graph_lifecycle.rs` `bootstrap_concurrent_indexes` |
| Knowledge rebuild lite | ✅ | `knowledge_rebuild.rs` |
| Community refresh | ✅ async | `schedule_community_index_refresh_with_extras` |
| **retry-chunks** | ❌ **STUB** | `handlers/documents/recovery/chunks.rs` `implemented: false` |
| **failed-chunks list** | ❌ empty | same file — does not query `failed_chunks` |
| Cross-store 2PC | ❌ by design | compensation best-effort |

### D. Tracing / Metrics

| Layer | Status | Evidence |
|-------|--------|----------|
| Prometheus | ✅ | `edgequake-observability/src/metrics.rs` |
| Query duration/mode | ✅ | `edgequake_query_duration_seconds`, `record_query_completed` |
| LLM / ingest / compensation | ✅ | counters listed in metrics.rs |
| Optional OTLP | ⚠️ off by default | `OTEL_EXPORTER_OTLP_ENDPOINT` / `EDGEQUAKE_OTEL_ENABLED` |
| Graph quality Prometheus | ❌ | `graph_metrics.rs` → tracing only |
| Per-arm query latency | ❌ | `QueryStats` lacks local/global/naive splits |
| GenAI semantic conventions | ❌ | No `gen_ai.*` / `rag.retrieval.*` attributes |
| Token histograms | ❌ | tokens in response stats only |
| Retrieval fingerprint | ✅ header | `X-Retrieval-Fingerprint` |

---

## Gaps vs July 2026 Best Practice

### Gap matrix

| Concern | 2026 practice | EdgeQuake today | Severity |
|---------|---------------|-----------------|----------|
| Cross-store drift | Measure + heal | Inspector + compensation; no drift SLO | **P1** |
| Chunk-level retry | Persist failures + retry | Table exists (M021); API stub | **P0** |
| Silent strategy downgrade | Fail loud / flag metadata | Semantic→Recursive warn-only | **P0** |
| HNSW DDL failure | Fail boot / readiness | `.ok()` swallow in `ddl.rs:63` | **P0** |
| Full-graph community | Incremental / sampled | `get_all_nodes` + `get_all_edges` | **P0** |
| Observability | OTel GenAI + arm metrics | Coarse Prometheus | **P1** |
| Migration rollback | Documented forward-only + PITR | Forward-only; M038 index rollback only | **P1** |
| PG checksums | PG18 default on new clusters | Depends on image init | **P2** |
| Faithfulness online eval | Sampled LLM-as-judge | Synthetic bench only | **P1** |

### Complexity (O(N)) hotspots

| Operation | Complexity | Path | Risk |
|-----------|------------|------|------|
| `pg_get_all_nodes` | **O(N)** unbounded | `nodes_ops.rs:394` | Community + reconcile |
| Community Louvain | **O(V+E)** full load | `community.rs:158-159` | Ingest refresh |
| Scoped BFS + per-node fetch | O(frontier×degree) + N+1 nodes | `query_ops.rs` | Admin KG view |
| Vector ANN | O(log N) | HNSW + `ef_search` | OK if index exists |
| Filtered ANN | O(log N) capped 20k | `iterative_scan` | OK ≥ pgvector 0.8 |
| Mix 3-arm retrieve | 3× retrieval | `mix.rs` | Cost on L1 if forced |

---

## Lens: Database Expert

**Verdict:** Storage substrate is **production-grade for a Postgres GraphRAG** (HNSW, halfvec policy, iterative_scan, AGE indexes, reconcile bootstrap). The weak points are **operator-visible failure modes** (swallowed DDL, stub repair) and **unbounded graph scans**, not missing extensions.

**Must-fix before claiming "defense in depth":**

1. Never swallow HNSW create errors — surface on `/ready`.
2. Ban `get_all_nodes` from ingest hot path; require workspace-scoped + LIMIT/sample.
3. Wire `failed_chunks` end-to-end (write on extract fail → list → retry).
4. Document PITR / `pg_basebackup` as the only true rollback for major upgrades.

## Lens: AI Engineer

**Verdict:** Retrieval brain (Mix+RRF+BM25+router) is LightRAG-class. Reliability of **evidence quality** is undermined by silent chunking downgrade, silent embedding truncation, and missing arm-level telemetry — you cannot tune what you cannot see.

**Must-fix:**

1. Strategy downgrade → document metadata flag + metric.
2. `QueryStats` arm timings + fusion/walk/pick method.
3. OTel spans: embed → retrieve(arm) → fuse → rerank → generate with `rag.retrieval.empty_result`, `rag.context.truncated`.

---

## External Evidence Anchors

| ID | Source | Takeaway for EdgeQuake |
|----|--------|------------------------|
| X-OPS-01 | [pgvector 0.8.0 release](https://www.postgresql.org/about/news/pgvector-080-released-2952/) | iterative_scan required for filtered ANN recall |
| X-OPS-02 | [Azure AGE performance](https://learn.microsoft.com/en-us/azure/postgresql/azure-ai/generative-ai-age-performance) (2026-01) | AGE creates **no** indexes by default; BTREE id/start/end + GIN properties mandatory |
| X-OPS-03 | [AGE 1.7.0 PG17 notes](https://github.com/apache/age/releases/tag/PG17/v1.7.0-rc0) | Upgrade scripts may be slow (index creation); RLS added |
| X-OPS-04 | [OTel GenAI spans](https://github.com/open-telemetry/semantic-conventions-genai) | Standardize retrieval/generation attributes |
| X-OPS-05 | RAG production 2026 guides | Faithfulness ≥0.9 gate; provenance of retrieved IDs |
| X-OPS-06 | PG18 checksums default | New clusters should keep checksums on |

---

## Synthesis

EdgeQuake already has the **skeleton** of enterprise reliability (saga, inspector, migrations, Prometheus).  
It is **not yet defense-in-depth complete** because several heal/observe paths are stubs or silent.

Next document: [10 — Postgres / pgvector / AGE Performance](./10-POSTGRES-PGVECTOR-AGE-PERFORMANCE.md)  
Plan: [12 — Implementation Plan (Ops + Storage)](./12-IMPLEMENTATION-PLAN-OPS.md)
