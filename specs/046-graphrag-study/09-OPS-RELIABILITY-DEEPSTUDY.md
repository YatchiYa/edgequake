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
| **retry-chunks** | ✅ **DONE (v0.16)** | `retry_failed_chunks` → extract + `KnowledgeGraphMerger::merge` |
| **failed-chunks list** | ✅ **DONE (v0.16)** | `failed_chunks.rs` + recovery handlers |
| Cross-store 2PC | ❌ by design | compensation best-effort (`compensate_orphan_kv`) |

### D. Tracing / Metrics

| Layer | Status | Evidence |
|-------|--------|----------|
| Prometheus | ✅ | `edgequake-observability/src/metrics.rs` |
| Query duration/mode | ✅ | `edgequake_query_duration_seconds`, `record_query_completed` |
| LLM / ingest / compensation | ✅ | counters listed in metrics.rs |
| Optional OTLP | ⚠️ off by default | `OTEL_EXPORTER_OTLP_ENDPOINT` / `EDGEQUAKE_OTEL_ENABLED` |
| Graph quality Prometheus | ✅ **v0.16** | `record_graph_quality` from `log_graph_quality` |
| Per-arm query latency | ✅ **v0.16** | `QueryStats.arm_*_ms` + `run_arm_timed` |
| GenAI / rag retrieval spans | ✅ **v0.16** | `rag_span.rs` + arm/mode wiring |
| Faithfulness (heuristic + LLM-judge) | ✅ **v0.16** | `faithfulness.rs`, `faithfulness_judge.rs` |
| Token histograms | ❌ | tokens in response stats only |
| Retrieval fingerprint | ✅ header | `X-Retrieval-Fingerprint` |

---

## Gaps vs July 2026 Best Practice

### Gap matrix — **post v0.16.0 code-is-law refresh**

| Concern | 2026 practice | EdgeQuake **v0.16.0** | Severity |
|---------|---------------|----------------------|----------|
| Cross-store drift | Measure + heal | Inspector + compensate + **drift_* metrics** | ✅ P1 closed |
| Chunk-level retry | Persist + retry + merge | **failed_chunks + retry → merge** | ✅ P0 closed |
| Silent strategy downgrade | Fail loud | **Semantic fail-loud** | ✅ P0 closed |
| HNSW DDL failure | Fail boot / readiness | **fail-closed + `missing_hnsw_index`** | ✅ P0 closed |
| Full-graph community | Incremental / sampled | **`load_graph_bounded`** | ✅ P0 closed |
| Observability | OTel GenAI + arm metrics | **rag spans + arm timings + graph gauges** | ✅ P1 closed |
| Migration rollback | Forward-only + PITR | Unchanged — PITR is law | P1 (ops process) |
| PG checksums | PG18 default | Depends on image init | P2 |
| Faithfulness online eval | Sampled LLM-as-judge | **Heuristic + opt-in judge + ACC CI** | ✅ P1 closed |
| Full HF GraphRAG-Bench ACC | Nightly corpus | Mini corpus only (no HF download) | ⏳ deferred |
| True cross-encoder rerank | Prod default | BM25 path; CE hook open | ⏳ deferred |

### Complexity (O(N)) hotspots

| Operation | Complexity | Path | Risk |
|-----------|------------|------|------|
| `pg_get_all_nodes` | **O(N)** unbounded | `nodes_ops.rs` | **Admin/legacy only** — not community hot path |
| Community Louvain | **O(sample)** capped | `community.rs` `load_graph_bounded` | ✅ ingest-safe |
| Scoped BFS/PPR + per-node fetch | O(frontier×degree) | `graph_expand` / `graph_ppr` | OK with caps |
| Vector ANN | O(log N) | HNSW + `ef_search` | OK if index exists (`/ready`) |
| Filtered ANN | O(log N) capped | `iterative_scan` relaxed | OK ≥ pgvector 0.8 |
| Mix/Hybrid arms | 1–3× retrieval | intent-gated | L1 can skip graph |

---

## Lens: Database Expert

**Verdict (v0.16.0):** Storage substrate is **production-grade Postgres GraphRAG** with **fail-closed ANN readiness**, bounded community load, and multi-major pin smoke. Remaining work is **measured perf artifacts** (100k/1M benches) and PITR runbook discipline — not missing extensions.

**Shipped must-fixes:**

1. ✅ Never swallow HNSW create errors — surface on `/ready`.
2. ✅ Ban unbounded `get_all_nodes` from ingest hot path — `load_graph_bounded`.
3. ✅ Wire `failed_chunks` end-to-end (write → list → retry → merge).
4. Document PITR / `pg_basebackup` as the only true rollback for major upgrades (process).

## Lens: AI Engineer

**Verdict (v0.16.0):** Retrieval brain is LightRAG-class **plus** PPR-default bipartite dual-node pick, intent-gated arms, path prune, ACC CI, and optional LLM-judge faithfulness. Deferred: full HF GraphRAG-Bench download, true cross-encoder, density YAML, LLM community report depth.

**Shipped must-fixes:**

1. ✅ Strategy downgrade → fail-loud + metric.
2. ✅ `QueryStats` arm timings + fusion/walk/pick telemetry.
3. ✅ OTel/rag spans on arms + single modes.

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

## Synthesis (v0.16.0)

EdgeQuake has the **skeleton and the P0–P3 flesh** of enterprise reliability (saga, inspector, migrations, Prometheus, fail-closed ANN, chunk retry, ACC).

**Honest label:** production Hybrid RAG with fail-closed ops substrate — not "defense stubs." Remaining gaps are **science depth** (HF corpus, cross-encoder, LLM community reports) and **perf measurement artifacts**, not silent heal paths.

Next document: [10 — Postgres / pgvector / AGE Performance](./10-POSTGRES-PGVECTOR-AGE-PERFORMANCE.md)  
Implementation status: [12 — Ops Plan](./12-IMPLEMENTATION-PLAN-OPS.md) (all tickets DONE).
