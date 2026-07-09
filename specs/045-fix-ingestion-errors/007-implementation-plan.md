# SPEC-045 — Implementation Plan

**Status:** `SHIPPED` — P0-SRE + P1-SRE battle-proof mitigations implemented  
**Cross-ref:** [010-sre-engineering-review](./010-sre-engineering-review.md) · [011-battle-proof-first-principles](./011-battle-proof-first-principles.md)

---

## Phase P0 — Shipped ✅

| ID | Item | Status |
| -- | ---- | ------ |
| P0-1 | `GraphMerge` failure class | ✅ |
| P0-2 | Operator runbook + health SQL | ✅ |
| P1-1 | Permanent error fast-fail | ✅ |
| P1-2 | Embedding 429 retry | ✅ |
| P2-1 partial | `EDGEQUAKE_AUTO_ORPHAN_DOCUMENT_RECOVER_MINUTES` | ✅ |

---

## Phase P0-SRE — Battle-proof blockers (ship before next prod deploy)

> From [010-sre-engineering-review](./010-sre-engineering-review.md). These are **higher priority** than P1-3/P1-4 because they cause silent post-migration split-brain.

### P0-SRE-1 — Unify vector dimension heal (SRE-Q02) 🔴 CRITICAL

**Problem:** Query evicts cache on dimension mismatch; ingestion cache-hits stale storage.

**Files:**
- `edgequake-core/src/workspace_vector_resolve.rs` L97–98
- Mirror logic from `handlers/query/workspace_resolve.rs` L149–168

**Change:** On `get_or_create` failure OR cache hit with `cached_dim != workspace.embedding_dimension`, evict + retry.

**Acceptance:** Contract test `spec045_vector_resolve_parity.rs` — ingest and query paths behave identically.

**First principle:** I2 (embedding dim == table dim) across all pipelines.

---

### P0-SRE-2 — Periodic orphan syncs document KV (SRE-I01) 🔴

**Problem:** `periodic_orphan_check` marks task `Failed` but document stays `processing` in UI.

**Files:**
- `edgequake/src/main.rs` L440–451
- Extract shared helper from `processor/task_impl.rs` `on_permanent_failure` L81–131

**Change:** After task marked failed, call document status update with `failure_class: orphan_heartbeat`.

**Acceptance:** Battle test simulates dead heartbeat → doc metadata `failed`.

---

### P0-SRE-3 — Paginate `requeue_pending_tasks` (SRE-I02) 🔴

**Problem:** Only first 1000 pending tasks requeued after outage.

**Files:** `main.rs` L355–412

**Change:** Mirror pagination loop from `recover_orphaned_tasks` L104–159.

**Acceptance:** Unit test with >1000 pending mock tasks.

---

### P0-SRE-4 — M080 post-hook cache invalidation (SRE-M05) 🔴

**Problem:** halfvec schema change without registry cache clear → ingest/query divergence.

**Files:**
- `migration_bootstrap/mod.rs` post-M080 hook
- `state/postgres.rs` vector registry

**Change:** After M080 reconcile, `vector_registry.clear_cache()` + log `operator_action: verify embeddings`.

**Acceptance:** Health field `schema.halfvec_conversion_applied: true`.

---

## Phase P1-SRE — Reliability parity (1–2 sprints)

### P1-SRE-1 — `failure_class` Prometheus metrics (SRE-I06)

**Files:** `edgequake-observability/src/metrics.rs`, `status_updates.rs` L12–17

```text
edgequake_ingestion_failures_total{failure_class, workspace}
edgequake_compensation_quarantine_total
```

### P1-SRE-2 — `QueryFailureClass` taxonomy (SRE-Q01)

**Files:** New `edgequake-query/src/query_reliability.rs`

Mirror `ingestion_reliability.rs` pattern: `VectorEmpty`, `DimensionMismatch`, `GraphUnavailable`, `LlmAuth`, `Timeout`.

Wire into `error.rs` `From<QueryError>` diagnostic JSON.

### P1-SRE-3 — Enrich `/ready` with JSON blockers (SRE-M03)

**Files:** `handlers/health.rs` L380–387

```json
{"ready": false, "blockers": ["migration_038"], "operator_action": "apply_038.sh --concurrent"}
```

Keep HTTP 503 for K8s; add body for operators.

### P1-SRE-4 — Extend `MigrationBootstrapReport` for M080/M081 (SRE-M02)

**Files:** `migration_bootstrap/mod.rs`, `handlers/health.rs`

Expose halfvec + RLS state in `/health.migration_bootstrap`.

### P1-SRE-5 — PDF path in `recover-stuck` (SRE-I05)

**Files:** `handlers/documents/recovery/stuck.rs`

Re-enqueue `PdfProcessing` when `pdf_id` present in metadata.

### P1-SRE-6 — Wire timeout to all ingest tasks (P1-3 / SRE-I03)

**Files:** `document_admission.rs`, `large_document_profile.rs`

Set `metadata.processing_timeout_secs` for text + PDF at enqueue.

---

## Phase P1 — Original backlog (still open)

| ID | Item | Priority vs P0-SRE |
| -- | ---- | ------------------ |
| P1-4 | `POST /documents/reprocess-failed` bulk | P2 (operator convenience) |

---

## Phase P2-SRE — Automation & multi-replica

| ID | Item | SRE ref |
| -- | ---- | ------- |
| P2-SRE-1 | Leader-elected startup recovery (advisory lock) | SRE-I09 |
| P2-SRE-2 | Periodic M038/M047 re-audit cron | SRE-M04 |
| P2-SRE-3 | Real LLM/embeddings health probe | SRE-Q04 |
| P2-SRE-4 | Post-migration vector+graph smoke before workers | 011 acceptance |
| P2-SRE-5 | `EDGEQUAKE_AUTO_RECOVER_STUCK_MINUTES` → `run_recover_stuck` | G6 full |
| P2-SRE-6 | Enable rate limits in production profile | SRE-Q03 |
| P2-SRE-7 | WebUI readiness banner | P2-3 |
| P2-SRE-8 | CI: `make spec045-battle-test-all` required | P2-2 |

---

## Phase P3 — Hardening backlog

| ID | Item | SRE ref |
| -- | ---- | ------- |
| P3-1 | 0-entity extraction fail-fast | SRE-I08 |
| P3-2 | Empty PDF content threshold | SPEC-011 EC-006 |
| P3-3 | Born-digital admission routing | SPEC-038 |
| P3-4 | IRC orchestration module | 006-bulletproof |
| P3-5 | Postgres-scale merge load test | EC-045-03 |
| P3-6 | DLQ for failed queue.send | SRE-I10 |
| P3-7 | Ingest SLO histograms | 010 §7 |

---

## Updated rollout order (SRE-informed)

1. **P0-SRE-1** — Vector resolve parity (closes #1 post-migration incident class)
2. **P0-SRE-2/3** — Orphan doc sync + pending pagination
3. **P0-SRE-4** — M080 cache invalidation
4. Deploy + run `make spec045-battle-test-all` + `make spec044-battle-test-all`
5. Apply M038 concurrent indexes if degraded
6. **P1-SRE-1/2** — Metrics + query taxonomy
7. **P1-SRE-3/4** — `/ready` JSON + M080 health
8. Enable `EDGEQUAKE_AUTO_ORPHAN_DOCUMENT_RECOVER_MINUTES=15` in prod

---

## Test plan (extended)

```bash
# Existing battle tests
make spec045-battle-test-all

# New (after P0-SRE-1)
cargo test -p edgequake-core --test spec045_vector_resolve_parity -- --nocapture

# New (after P0-SRE-2)
cargo test -p edgequake-api --test spec045_periodic_orphan_doc_sync -- --nocapture

# Cross-pipeline contract
cargo test -p edgequake-core workspace_vector_resolve -- --nocapture

# SRE regression
cargo test -p edgequake-tasks spec045 -- --nocapture
cargo test -p edgequake-api --test spec045_ingestion_reliability -- --nocapture
```

---

## Requirements traceability (updated)

| REQ | Status after SRE review |
| --- | ----------------------- |
| REQ-045-01 | ✅ metadata has failure_class |
| REQ-045-02 | ✅ graph_merge |
| REQ-045-08 | ✅ permanent fast-fail |
| REQ-045-10 | ✅ make target; ❌ CI mandatory |
| **REQ-045-11** (new) | Cross-pipeline vector resolve parity | ✅ P0-SRE-1 |
| **REQ-045-12** (new) | Task/doc terminal sync at runtime | ✅ P0-SRE-2 |
| **REQ-045-13** (new) | Query failure taxonomy | ✅ P1-SRE-2 |
| **REQ-045-14** (new) | SLO metrics per failure_class | ✅ P1-SRE-1 |
