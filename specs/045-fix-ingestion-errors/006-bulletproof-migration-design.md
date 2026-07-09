# SPEC-045 — Bulletproof Migration Design

**Goal:** Make post-migration ingestion **self-healing** without operator intervention for common cases; **fail-closed** for unsafe states; **actionable** for everything else.

**Cross-ref:** `edgequake/docs/migrations/bootstrap-first-principles.md` · [007-implementation-plan](./007-implementation-plan.md)

---

## Design principles

| # | Principle | Implementation |
| - | --------- | -------------- |
| P1 | **Marker ≠ done** | sqlx markers + bootstrap `apply.sql` reconcile |
| P2 | **Idempotent every boot** | M045–M047, M046 audit re-run safely |
| P3 | **Size-aware deferral** | M038 large graph → CONCURRENTLY ops script |
| P4 | **Fail-closed readiness** | Only M038 + M042 block `/ready` |
| P5 | **Saga compensation** | Merge failure → rollback vectors + graph orphans |
| P6 | **Startup recovery** | Orphan tasks/docs before workers |
| P7 | **Typed failures** | `failure_class` + `recommended_action` SSOT |
| P8 | **Defense in depth** | L1 checksum repair + L2 migration + L3 reconcile |

---

## Current auto-migration inventory (✅ shipped)

### Layer 1 — Pre-sqlx repair

| Hook | Purpose |
| ---- | ------- |
| `repair_migration_078_checksum_if_needed` | M078 typo skip-path (#273) |
| `repair_migration_071_checksum_if_needed` | HNSW dim guard checksum |
| `reconcile_migration_041` | `documents.cost_usd` columns |

### Layer 2 — sqlx markers

65+ embedded migrations; blocking DDL avoided in markers (M038, M042 pattern).

### Layer 3 — Post-sqlx reconcile (every boot)

| Migration | Auto action | Blocks traffic? |
| --------- | ----------- | --------------- |
| M038 | Index verify + inline repair if small | **Yes** if missing |
| M040 | Background CQRS entity backfill | No |
| M042 | pgvector upgrade + HNSW rebuild | **Yes** if < 0.8 |
| M043 | AGE extension upgrade | No |
| M045 | Vector FTS indexes | No |
| M046 | Graph tenant perf indexes | No |
| M047 | **wsdoc KV index backfill** | No |
| M071 | HNSW checksum repair | No |
| M078/M079 | AGE child Node indexes | No |
| M080 | halfvec conversion (when enabled) | No |

### Layer 4 — Runtime self-healing

| Mechanism | Trigger | Action |
| --------- | ------- | ------ |
| `recover_orphaned_tasks` | Startup | `processing` → `pending` |
| `recover_orphaned_documents` | Startup | non-terminal → pending/failed |
| `requeue_pending_tasks` | Startup | DB → memory queue |
| `upsert_metadata_kv_with_index` | Every metadata write | wsdoc sync |
| `reconcile_entity_counts_with_graph` | List API | Fix 0-entity display |
| `dimension_mismatch` retry | Query | Cache eviction + retry |
| `recover-stuck` API | Operator / cron | Stuck → pending |
| `reprocess` API | Operator | Clean + requeue |

---

## Gaps → bulletproof targets

### G1 — Failure taxonomy incomplete ✅ FIXED

**Was:** Graph merge → `unknown` / `retry`  
**Now:** `ingestion_reliability.rs` SSOT — `graph_merge` / `reprocess_full`

### G3 — Embedding 429 not retried ✅ FIXED

**Now:** `embed_batched_with_retry()` in `embeddings.rs`

### G4 — Permanent 400 retried wastefully ✅ FIXED

**Now:** `TaskFailureInfo::from_processing_error` + worker `mark_failed_with_details`

### G9 — Cross-pipeline vector resolve split-brain 🔴 NEW (SRE-Q02)

**Today:** Query evicts on dim mismatch; ingest cache-hits stale storage  
**Target:** `VectorStorageResolver` SSOT in `edgequake-core`  
**Priority:** P0-SRE-1

### G10 — Runtime task/doc desync on periodic orphan 🔴 NEW (SRE-I01)

**Today:** Task `Failed`, doc still `processing`  
**Target:** Shared orphan helper syncs KV  
**Priority:** P0-SRE-2

### G11 — Query failure taxonomy missing 🔴 NEW (SRE-Q01)

**Today:** Query errors are flat HTTP; no `failure_class`  
**Target:** `QueryFailureClass` mirroring ingestion  
**Priority:** P1-SRE-2

### G5 — Large PDF timeout not adaptive

**Today:** 7200s global cap  
**Target:** `LargeDocumentProfile::worker_timeout_secs()` wired to worker  
**Auto?** Per-document timeout from page count

### G6 — No cron recover-stuck

**Today:** Manual API call  
**Target:** Optional `EDGEQUAKE_AUTO_RECOVER_STUCK_MINUTES` env  
**Auto?** Background ticker every N minutes

### G7 — M038 deferred state silent in UI

**Today:** `/health` only  
**Target:** WebUI banner when `ready_for_traffic: false`  
**Auto?** Frontend polls health

### G8 — Failed doc batch reprocess

**Today:** Per-doc API  
**Target:** `POST /documents/reprocess-failed?workspace_id=` with rate limit  
**Auto?** Operator one-click

---

## Proposed: Ingestion Reliability Controller (IRC)

Single orchestration module (future — P2 in implementation plan):

```
IngestionReliabilityController
├── on_startup()
│   ├── verify_readiness_report()
│   ├── recover_orphans()          # existing main.rs
│   └── schedule_auto_recover()    # G6
├── on_document_failed(doc_id, class)
│   ├── classify()                 # existing
│   ├── if retriable(class) → schedule_retry()
│   └── emit_metric(failure_class)
└── on_migration_complete(report)
    ├── if was_degraded(M038) → log "safe to reprocess failed merge docs"
    └── trigger_wsdoc_verify_sample()
```

**SRP:** IRC coordinates; does not replace persister/merger/bootstrap.

---

## Auto-migration decision tree

```
New migration needed?
├── DDL < 30s, idempotent?
│   └── YES → reconcile/mNNN.rs + apply.sql every boot
├── DDL on large table/graph?
│   └── YES → marker + defer + /ready degrade + ops script
├── Data backfill > 1M rows?
│   └── YES → background tokio::spawn + progress in /health
└── One-time breaking change?
    └── checksum repair L1 + safety-net migration L2
```

---

## Operator zero-touch path (target state)

After upgrade deploy:

1. API starts → bootstrap runs all reconciles
2. If M038/M042 degraded → `/ready` 503 → K8s holds traffic (correct)
3. Ops script OR auto-concurrent completes indexes
4. `/ready` 200 → uploads resume
5. Startup recovers stuck `processing` docs automatically
6. Failed merge docs from incident → **one-click** `reprocess-failed` or cron
7. CI proved Cypher + ingest smoke before version shipped

---

## Metrics to add (observability)

| Metric | Labels | Alert |
| ------ | ------ | ----- |
| `edgequake_ingestion_failures_total` | `failure_class`, `workspace` | rate > 5/min |
| `edgequake_migration_degraded` | `migration_id` | == 1 for > 10 min |
| `edgequake_merge_errors_total` | `phase=entity\|relationship` | spike post-deploy |
| `edgequake_compensation_quarantine_total` | — | any increment |
| `edgequake_orphan_recovered_total` | `type=task\|document` | info |

---

## Compliance with existing patterns

This design **extends** — does not replace:

- SPEC-038 `LargeDocumentProfile` SSOT
- SPEC-041 L1/L2/L3 checksum repair
- SPEC-044 Cypher bare `$1` contract
- SPEC-027 wsdoc index write-path + M047 backfill
- AGENTS.md `make dev-bg` / `make status` workflow
