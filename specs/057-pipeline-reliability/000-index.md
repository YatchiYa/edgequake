# SPEC-057 — Pipeline Reliability & Scalability Assessment

**Spec:** `057-pipeline-reliability`  
**Date:** 2026-07-17  
**Status:** `P0+P1 IMPLEMENTED` (2026-07-17) — cancel/status truth + claim/lease delivery SSOT shipped; P2 Convert/Ingest split deferred  
**Method:** First principles → 5 Whys → code is law → cause register → phased plan  
**Trigger:** Controllability (cancel/fairness), restart durability, multi-tenant scale, and store-level reliability of the ingestion pipeline

---

## TL;DR

> Mid-pipeline reliability is strong (cooperative cancel SSOT, tenant park-not-churn, failure taxonomy, extract checkpoints, idempotent persist + saga compensation). **P1 shipped:** workers **claim** via `FOR UPDATE SKIP LOCKED` + leases; channel/NOTIFY is wake-only; **Pending survives boot** without `STARTUP_AUTO_RESUME`; Cancelled is never claimed; stale Processing → Interrupted/Failed (Reprocess). Remaining gaps: PDF convert and KG ingest still run **inline in one worker slot** (P2), and multi-replica Bridged default (P3). Cancel intents remain process-local accelerators; DB `Cancelled` is restart SSOT. PDF rows use `PdfProcessingStatus::Cancelled` (P0).

**Operator cancel/fairness SSOT (do not fork):** [docs/ingestion-cancel-and-fairness.md](../../docs/ingestion-cancel-and-fairness.md)

---

## End-to-end pipeline (ASCII)

```text
  HTTP admit (PDF / text / file)
           │
           ▼
  ┌────────────────────┐     persist task row + PDF bytes
  │ edgequake-api      │──────────────────────────────────► Postgres (tasks, pdfs, KV)
  │ upload / admission │     enqueue → wake (channel/NOTIFY) — not delivery SSOT
  └─────────┬──────────┘
            │
            ▼
  ┌────────────────────┐
  │ WorkerPool         │  wake / poll → claim_next (SKIP LOCKED + lease)
  │ try_acquire        │──► at cap? release_claim + PARK (no 500ms requeue storm)
  └─────────┬──────────┘
            │ permit
            ▼
  ┌────────────────────┐     CancellationRegistry (token + intent)
  │ DocumentTaskProc   │──► PdfProcessing: convert → INLINE process_text_insert
  └─────────┬──────────┘     OR Insert: prepare → extract → persist → finalize
            │
            ▼
  ┌────────────────────┐
  │ edgequake-pipeline │  chunk → extract (±glean) → embed → merge
  └─────────┬──────────┘
            │ DefaultIngestionPersister
            ▼
  ┌────────────────────────────────────────────────────────┐
  │ Postgres: KV upsert → pgvector upsert → AGE merge      │
  │ On merge fail: compensate vectors / KV / partial graph │
  └────────────────────────────────────────────────────────┘
            │
            ▼
     Terminal: Indexed | Failed | Cancelled
     (PDF row: Completed | Failed | Cancelled — P0)
```

---

## Documents

| File | Lens | Key question |
| ---- | ---- | ------------ |
| [001-five-whys.md](./001-five-whys.md) | 5 Whys | Why does ingest feel unreliable at the edges? |
| [002-first-principles.md](./002-first-principles.md) | First principles | What must be true for reliable ingest? |
| [003-code-is-law.md](./003-code-is-law.md) | Code is law | Exact symbols proving current behavior |
| [004-product-owner-lens.md](./004-product-owner-lens.md) | Product Owner | Controllability, trust, multi-tenant SLOs |
| [005-ux-expert-lens.md](./005-ux-expert-lens.md) | UX | Cancel / Stopping… / reprocess mental model |
| [006-ui-expert-lens.md](./006-ui-expert-lens.md) | UI | Badge matrix, ASCII states, track_id identity |
| [007-fullstack-expert-lens.md](./007-fullstack-expert-lens.md) | Full Stack | API matrix, DRY cancel, SOLID delivery seams |
| [008-on-expert-lens.md](./008-on-expert-lens.md) | O(n) | Cost classes and worker-slot asymptotics |
| [009-postgres-relational-lens.md](./009-postgres-relational-lens.md) | Postgres relational | Durable rows vs ephemeral channel |
| [010-age-pgvector-lens.md](./010-age-pgvector-lens.md) | AGE + pgvector | Persist order, saga, contention |
| [011-ai-engineer-lens.md](./011-ai-engineer-lens.md) | AI Engineer | Provider clamps, retries, checkpoints |
| [012-unreliability-causes-matrix.md](./012-unreliability-causes-matrix.md) | Cause register | Cause → roadblock → mitigation |
| [013-cross-reference-matrix.md](./013-cross-reference-matrix.md) | Cross-ref | REQ ↔ code ↔ env ↔ test ↔ prior SPEC |
| [014-improvement-plan.md](./014-improvement-plan.md) | Engineering | DRY/SOLID P0–P3 plan |

---

## Requirements (REQ-057-xx)

| ID | Requirement | Priority |
| -- | ----------- | -------- |
| REQ-057-01 | Postgres task row is delivery SSOT; channel is wake signal only | P1 |
| REQ-057-02 | Cancel intent survives process restart (DB status or durable intent column) | P1 |
| REQ-057-03 | `PdfProcessingStatus` includes `Cancelled`; cancel paths never map cancel → Failed | P0 |
| REQ-057-04 | One status mapper SSOT for task / doc KV / PDF / unified stage | P0 |
| REQ-057-05 | UI cancel uses `POST /tasks/{track_id}/cancel` and shows Stopping… until terminal | P0 |
| REQ-057-06 | Cancelled is permanent — never auto-retry | P0 (mostly shipped) |
| REQ-057-07 | Split convert vs KG ingest (separate tasks or hard checkpoint barriers) | P2 |
| REQ-057-08 | Adaptive timeout / `LargeDocumentProfile` drives worker timeout + UX ETA | P2 |
| REQ-057-09 | Fairness clamp keys off runtime extraction provider, not only env LLM | P2 |
| REQ-057-10 | Multi-instance delivery (`Bridged`/`NotifyOnly` + SKIP LOCKED claim) is production path | P3 |
| REQ-057-11 | Saga compensation is idempotent; failed compensate → operator-visible DLQ/metric | P3 |
| REQ-057-12 | AGE/pgvector contention budgets + queue-metrics SLOs documented and enforced | P3 |
| REQ-057-13 | Every terminal failure surfaces `failure_class` + `recommended_action` | P0 (SPEC-045 lineage) |
| REQ-057-14 | Slim checkpoint resume re-embeds honestly; checkpoint size bounded | P2 |
| REQ-057-15 | Contract tests cover cancel, fairness park, restart skip of Cancelled | P0–P1 |

---

## Env knobs (reliability / fairness)

| Variable | Role | Default / note |
| -------- | ---- | -------------- |
| `WORKER_THREADS` | Worker pool size | `4` in `.env.example`; IO default ≈ cpus×4 |
| `MAX_TASKS_PER_TENANT` | Fairness cap | ≈ ¾ workers; `0` disables; local clamp → 1 |
| `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY` | Lift local clamp | off |
| `TASK_PROCESSING_TIMEOUT_SECS` | Whole-task timeout | 7200 (min 60) |
| `EDGEQUAKE_STARTUP_AUTO_RESUME` | Hydrate pending on boot | **off** (SPEC-054) |
| `EDGEQUAKE_STARTUP_RECONCILE_MAX` | Orphan reconcile cap | 32 |
| `EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS` | Parallel chunk extract | provider-clamped locally |
| `EDGEQUAKE_CHUNK_TIMEOUT_SECS` / `EDGEQUAKE_CHUNK_RETRY_DELAY_MS` | Per-chunk resilience | see `.env.example` |
| `EDGEQUAKE_NATIVE_GRAPH_WRITES` | Native AGE upserts | on |
| `EDGEQUAKE_HNSW_*` | Vector index/query tuning | construction + iterative_scan |
| `EDGEQUAKE_QUEUE_PENDING_WARN` / `_CRITICAL` | Queue pressure | ops thresholds |

---

## Related specs

| Spec | Relationship |
| ---- | ------------ |
| [docs/ingestion-cancel-and-fairness.md](../../docs/ingestion-cancel-and-fairness.md) | Operator SSOT for cancel / fairness / restart |
| [SPEC-010](../010-ingestion-reliability/) | Historical token/JSON reliability |
| [SPEC-011](../011-pipeline-reliabilty/) | Embedding limits / edge cases (folder spelling preserved) |
| [SPEC-038](../038-ingestion-large-pdf/000-index.md) | Large PDF / vision vs EdgeParse asymptotics |
| [SPEC-045](../045-fix-ingestion-errors/000-index.md) | Failure taxonomy SSOT (`ingestion_reliability.rs`) |
| [SPEC-047-016](../047-rag-evaluation/016-ingest-speed-reliability-battle-plan.md) | Ingest speed/reliability battle plan |
| [SPEC-048](../048-improve-ux/) | Progress / controllability UX |
| [SPEC-050](../050-pipeline-and-delete/README.md) | Pipeline UX parity / delete / SRE |
| [SPEC-051](../051-reprocess/) | Reprocess parity |
| [SPEC-054](../054-fix-bugs-17/) | Startup auto-resume semantics |
| [SPEC-056](../056-issue-release-17/) | track_id progress identity |
| [SPEC-026 delivery](../026-egdequake-vs-lightrag/) | Horizontal delivery mode lineage |

---

## Proof pointers (existing)

```bash
# Cancel + fairness contract
cargo test -p edgequake-api --test contract_cancel_and_fairness

# SPEC-045 ingestion reliability contracts
cargo test -p edgequake-api --test spec045_ingestion_reliability

# SPEC-026 delivery modes
cargo test -p edgequake-tasks --test contract_spec026_task_delivery
```

---

## How to read this pack

1. Start with [001-five-whys.md](./001-five-whys.md) and [002-first-principles.md](./002-first-principles.md).  
2. Verify claims in [003-code-is-law.md](./003-code-is-law.md).  
3. Skim the lens that matches your role (004–011).  
4. Use [012-unreliability-causes-matrix.md](./012-unreliability-causes-matrix.md) as the decision register.  
5. Execute from [014-improvement-plan.md](./014-improvement-plan.md); trace IDs via [013-cross-reference-matrix.md](./013-cross-reference-matrix.md).
