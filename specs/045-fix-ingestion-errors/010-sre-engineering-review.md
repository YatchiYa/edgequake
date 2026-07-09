# SPEC-045 — SRE Engineering Review (Code is Law)

**Lens:** Site Reliability Engineering  
**Method:** Code is law — every claim maps to live source, July 2026  
**Scope:** Ingestion pipeline · Auto-migration · Query pipeline · Cross-pipeline consistency  
**Cross-ref:** [002-first-principles](./002-first-principles.md) · [011-battle-proof-first-principles](./011-battle-proof-first-principles.md)

---

## Executive verdict

EdgeQuake is **production-capable but not battle-proof**. Post-SPEC-045 work closed ingestion taxonomy and permanent-error fast-fail gaps. The **dominant remaining SRE risk** is **asymmetric self-healing**: query paths auto-evict dimension mismatches; ingestion and migration paths do not share the same guarantees. Secondary risks: **task/doc state divergence** on runtime orphan detection, **opaque `/ready`**, and **missing SLO/metrics hooks**.

```
Battle-proof score (SRE lens, 0–10)
───────────────────────────────────
Ingestion pipeline     7/10  (strong startup recovery; runtime gaps)
Auto-migration         7/10  (M038/M042 gates good; M080/M046 silent)
Query pipeline         6/10  (dim heal yes; no failure taxonomy)
Cross-pipeline SSOT    4/10  ← PRIMARY GAP
Observability/SLO      5/10  (coarse metrics; no failure_class counters)
```

---

## 1. Ingestion pipeline — SRE assessment

### 1.1 What works (verified)

| Mechanism | File | Lines |
| --------- | ---- | ----- |
| Persist-then-deliver task queue | `edgequake-tasks/src/delivery/mod.rs` | L21–35 |
| Worker backoff + tenant fairness | `edgequake-tasks/src/worker.rs` | L292–335, L464–499 |
| Heartbeat + RAII guard | `worker.rs` | L359–388, L51–64 |
| Circuit breaker (3 timeouts) | `edgequake-tasks/src/types/task.rs` | L206–229 |
| Permanent failure SSOT | `edgequake-tasks/src/ingestion_reliability.rs` | full module |
| Structured task failures | `failure.rs` | `from_processing_error` L123–141 |
| Saga merge compensation | `ingestion_persister.rs` | L343–370 |
| Quarantine logging | `compensation.rs` | L86–104 |
| Pipeline checkpoints | `pipeline_checkpoint.rs` | L118–273 |
| Startup orphan recovery (paginated) | `main.rs` | L87–172, L178–353 |
| PDF per-doc timeout metadata | `pdf_upload/helpers.rs` | L193–195 |
| failure_class in KV metadata | `status_updates.rs` | L10–17 |

### 1.2 SRE gaps (missed by SPEC-045 until this review)

| ID | Gap | Severity | Evidence |
| -- | --- | -------- | -------- |
| **SRE-I01** | Periodic orphan marks task `Failed` but **never updates document KV** | **P0** | `main.rs` L440–451 — no `on_permanent_failure` |
| **SRE-I02** | `requeue_pending_tasks` capped at **1000** (no pagination) | **P0** | `main.rs` L367–372 |
| **SRE-I03** | Text ingest lacks `processing_timeout_secs` in task metadata | **P1** | PDF only: `pdf_upload/helpers.rs` L193–195; worker default 7200s `main.rs` L670–684 |
| **SRE-I04** | Worker wall-clock timeout → **immediate permanent** (no retry) | **P1** | `worker.rs` L525–548 |
| **SRE-I05** | `recover-stuck` is **text-only** (no PDF re-enqueue) | **P1** | `recovery/stuck.rs` L156–224 |
| **SRE-I06** | No Prometheus counter by `failure_class` | **P1** | `edgequake-observability/src/metrics.rs` L212–255 |
| **SRE-I07** | Compensation quarantine is **log-only** (no metric/alert) | **P1** | `compensation.rs` L86–104 |
| **SRE-I08** | 0-entity extraction → `partial_failure` not operator alert | **P2** | `persist.rs` L276–285 |
| **SRE-I09** | Multi-replica startup recovery assumes single fleet | **P2** | `main.rs` L117–121 comment |
| **SRE-I10** | No DLQ for failed `queue.send` on retry | **P3** | `worker.rs` L490–497 |

### 1.3 Ingestion reliability scorecard

| Layer | Strength | Weak spot |
| ----- | -------- | --------- |
| Admission | ID stability, PDF single-flight | Text timeout not profiled |
| Queue | Persist-first | 1000 pending requeue cap; no DLQ |
| Worker | Backoff, fairness, taxonomy | Timeout=permanent; periodic orphan incomplete |
| Pipeline | Checkpoints, resilient extract | 0-entity partial success |
| Persist | Merge compensation saga | Quarantine not metered |
| Recovery | Strong **startup** hooks | Runtime task/doc desync |

---

## 2. Auto-migration — SRE assessment

### 2.1 What works (verified)

| Mechanism | File | Evidence |
| --------- | ---- | -------- |
| sqlx advisory lock | `migration_bootstrap/mod.rs` | L647–703 |
| L1 checksum repair (M071, M078) | `mod.rs` | L674–687 |
| M038 size-aware deferral | `reconcile/m038.rs` | L76–89 |
| M042 pgvector gate | `reconcile/m042.rs` + `is_degraded` | `mod.rs` L290–292 |
| M047 idempotent every boot | `reconcile/m047.rs` | L20–29 |
| M040 background backfill | `mod.rs` | L813–822 |
| `/ready` blocks M038 + M042 | `health.rs` | L380–387 |
| Startup: migrations before workers | `postgres.rs` L214–215 → `main.rs` L687–758 |

### 2.2 SRE gaps

| ID | Gap | Severity | Evidence |
| -- | --- | -------- | -------- |
| **SRE-M01** | Only M038/M042 block `/ready`; M080 halfvec, M046, M040 in-flight do not | **P0** | All `is_degraded()` false except 038/042: `mod.rs` L305–599 |
| **SRE-M02** | M080/M081 not in `MigrationBootstrapReport` health snapshot | **P1** | Report ends M065; M080/M081 run L747–760 but not exposed |
| **SRE-M03** | `/ready` returns bare 503 — **no blocker body** | **P1** | `health.rs` L380–387 |
| **SRE-M04** | Reconcile runs **once at boot** only (no periodic audit) | **P1** | No cron for M038/M047 re-audit |
| **SRE-M05** | M080 runs when `VectorStorageMode::Half` with no cache invalidation | **P0** | `reconcile/m080.rs` L18–32; no `vector_registry.evict` |
| **SRE-M06** | M081 silently skipped if AGE < 1.7 | **P2** | `reconcile/m081.rs` L27–34 |
| **SRE-M07** | `is_ready_for_traffic(None) → true` masks missing report | **P2** | `mod.rs` L604–605 |
| **SRE-M08** | M040 backfill failure warn-only, no health counter | **P2** | `reconcile/m040.rs` L41–47 |

### 2.3 First principle violation

> **Invariant I4** (002-first-principles): metadata consistent at commit.  
> **Violation:** M080 schema change + M040 backfill lag + M047 async backfill can leave list/query/ingest seeing different document sets **without blocking traffic**.

---

## 3. Query pipeline — SRE assessment

### 3.1 What works (verified)

| Mechanism | File | Evidence |
| --------- | ---- | -------- |
| Workspace fail-closed | `workspace_resolve.rs` | L36–48 |
| Dimension mismatch evict+retry | `workspace_resolve.rs` | L136–168 (OODA-225) |
| Query execution SSOT | `services/query_execution.rs` | L49–110 |
| LLM auth fallback | `query_execution.rs` | L125–143 |
| QueryError → HTTP mapping | `error.rs` | L718–735 |
| Result cache + ingest invalidation | `query_result_cache.rs` | L84–88 |
| Rate limit middleware (wired) | `routes.rs` | L97–100 |

### 3.2 SRE gaps

| ID | Gap | Severity | Evidence |
| -- | --- | -------- | -------- |
| **SRE-Q01** | **No `QueryFailureClass`** (ingestion has taxonomy; query does not) | **P1** | `edgequake-query/src/error.rs` L10–42 |
| **SRE-Q02** | Ingestion vector resolve **cache hit without dimension validate** | **P0** | `workspace_vector_resolve.rs` L97–98 vs query L149–168 |
| **SRE-Q03** | Rate limiting **off by default** | **P1** | `security_config.rs` L37,55 |
| **SRE-Q04** | LLM health probe stubbed `true` | **P2** | `health.rs` L89 |
| **SRE-Q05** | Graph init non-fatal → opaque query failures | **P2** | `postgres.rs` L279–285 |
| **SRE-Q06** | No query SLO metrics (latency, empty-result rate) | **P2** | No dedicated counters |
| **SRE-Q07** | Result cache `context_only` — full RAG never cached | **P3** | `query_result_cache.rs` L99–120 |

---

## 4. Cross-pipeline inconsistency (post-migration)

This is the **#1 battle-proof gap** — ingestion and query do not share vector storage self-healing.

| Scenario | Query behavior | Ingestion behavior | User-visible symptom |
| -------- | -------------- | ------------------ | -------------------- |
| Provider dimension switch | Evict cache + retry (`workspace_resolve.rs` L149–168) | Cache hit, no validate (`workspace_vector_resolve.rs` L97–98) | Query works; new ingest fails |
| M080 halfvec conversion | No special handling | Writes may hit wrong schema | Search empty / insert error |
| Startup `ensure_dimension` table recreate | Query re-creates storage | Existing docs show `completed`, vectors gone | "Ghost completed" docs |
| M038 deferred indexes | Slow but returns results | Merge timeouts → `graph_merge` fail | Query OK, ingest fails |
| M040 entity backfill lag | Graph query OK | List shows `entity_count: 0` | Dashboard mismatch |

**First principle:** One workspace → one vector storage instance → one dimension.  
**Reality:** Three resolution paths (query handler, ingest resolver, startup ensure_dimension) with **different retry semantics**.

---

## 5. What SPEC-045 shipped vs what SRE review found missed

### Shipped (confirmed in code)

- REQ-045-02 GraphMerge class
- REQ-045-08 permanent 400 fast-fail
- EC-045-09 embedding 429 retry (pipeline)
- EC-045-06 partial auto-repair (`EDGEQUAKE_AUTO_ORPHAN_DOCUMENT_RECOVER_MINUTES`)
- 22 battle tests (009-battle-test-results.md)

### Missed (this SRE review)

| Area | Gap IDs |
| ---- | ------- |
| Ingestion runtime | SRE-I01, SRE-I02, SRE-I05 |
| Cross-pipeline | SRE-Q02, SRE-M05 |
| Migration visibility | SRE-M01, SRE-M02, SRE-M03 |
| Observability | SRE-I06, SRE-I07, SRE-Q01 |
| Query hardening | SRE-Q03, SRE-Q04 |

### Doc drift fixed by this review

| Doc claim | Reality |
| --------- | ------- |
| `003-code-is-law.md` §4 "GraphMerge GAP" | **Fixed** — SSOT in `ingestion_reliability.rs` |
| `004-edge-cases-matrix` EC-045-09 429 OPEN | **Fixed** — `embeddings.rs` retry |
| `006-bulletproof` G1/G3/G4 OPEN | **Fixed** in code; docs stale |
| `/ready` documents blocking migration (005-runbook) | **Partial** — 503 only, no JSON body |

---

## 6. Battle-proof roadmap (SRE-prioritized)

See [007-implementation-plan.md](./007-implementation-plan.md) Phase **P0-SRE** through **P3-SRE**.

### P0-SRE (ship before next production deploy)

1. **SRE-Q02** — Unify dimension-mismatch evict+retry in `workspace_vector_resolve.rs`
2. **SRE-I01** — Periodic orphan: sync document KV on task heartbeat death
3. **SRE-I02** — Paginate `requeue_pending_tasks`
4. **SRE-M05** — Post-M080: `vector_registry.clear_cache()` + health flag

### P1-SRE (next sprint)

5. **SRE-I06/I07** — `failure_class` + `quarantine_compensation_total` metrics
6. **SRE-Q01** — `QueryFailureClass` + diagnostic JSON on query errors
7. **SRE-M02/M03** — Extend bootstrap report; enrich `/ready` JSON body
8. **SRE-I05** — PDF path in `recover-stuck`
9. **SRE-I03** — Wire `LargeDocumentProfile` timeout to all ingest tasks

### P2-SRE

10. Leader-elected startup recovery (multi-replica)
11. Periodic M038/M047 re-audit cron
12. Real LLM/embeddings health probe
13. Post-migration vector+graph smoke before workers start

---

## 7. SLO proposals (not yet instrumented)

| SLO | Target | Measurement needed |
| --- | ------ | ------------------ |
| Ingest success rate | ≥ 99% (excl. user errors) | `failure_class` counter / terminal status |
| Time-to-indexed p95 | < 10 min (docs < 100 pages) | histogram task create → finalize |
| Stuck doc age | 0 docs processing > 30 min | KV scan or metadata gauge |
| Query availability | ≥ 99.9% | HTTP 5xx rate on `/api/v1/query` |
| Migration readiness | `/ready` 200 within 5 min post-deploy | `migration_degraded` gauge |
| Compensation quarantine | 0 per day | `quarantine_compensation_total` |

---

## 8. Key code citations

**Query heals dimension mismatch:**

```136:168:edgequake/crates/edgequake-api/src/handlers/query/workspace_resolve.rs
    // OODA-225: Auto-evict and retry on dimension mismatch
    let storage = match state.storage.vector_registry.get_or_create(config.clone()).await {
        Ok(s) => s,
        Err(e) => {
            if error_msg.contains("Dimension mismatch") || error_msg.contains("cached=") {
                state.storage.vector_registry.evict(&workspace_uuid).await;
                state.storage.vector_registry.get_or_create(config).await?
```

**Ingestion does NOT heal (cache hit bypass):**

```97:99:edgequake/crates/edgequake-core/src/workspace_vector_resolve.rs
    if let Some(storage) = registry.get(&workspace_uuid).await {
        return Ok(storage);
    }
```

**Periodic orphan task/doc desync:**

```443:451:edgequake/src/main.rs
            if age > orphan_threshold {
                task.status = TaskStatus::Failed;
                task.error_message = Some(format!(
                    "Task heartbeat lost (no update for {} minutes). \
                     The worker may have crashed. Please retry.",
```

---

## 9. Related agents

- Ingestion deep-dive: [explore agent 23ef7960](23ef7960-9b25-4406-a276-8a5dadc4e6c9)
- Migration + query: [explore agent 4b174eca](4b174eca-f70c-4d0a-bada-52badd458370)
