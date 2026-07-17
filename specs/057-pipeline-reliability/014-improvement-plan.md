# 014 — Improvement Plan (DRY / SOLID)

**Spec:** SPEC-057  
**Status:** `P0+P1+P2+P3+P4 IMPLEMENTED` (2026-07-17) 
**Principles:** First principles ([002](./002-first-principles.md)), code is law ([003](./003-code-is-law.md)), causes ([012](./012-unreliability-causes-matrix.md))

---

## North-star architecture (before → after)

```text
  BEFORE (today)                         AFTER (target)
  ────────────────                       ────────────────
  admit → INSERT task                    admit → INSERT task (Pending)
       → channel.send                         → NOTIFY + optional channel wake
  worker ← recv only                     worker ← SKIP LOCKED claim (+ wake)
  cancel intent = memory                 cancel = DB Cancelled (+ memory accel)
  PDF status ⊆ {…, Failed}               PDF status ⊆ {…, Failed, Cancelled}
  PdfProcessing = convert+KG             Convert task → checkpoint → Ingest task
  Bridged optional                       Bridged/claim required if replicas>1
```

---

## DRY consolidations (do these once)

| SSOT module | Owns | Absorbs duplication from |
| ----------- | ---- | ------------------------ |
| `services::task_cancel` | Task row + registry cancel | Keep; extend restart-safe dequeue checks |
| `IngestionStatusMapper` (new) | Task/doc/PDF/unified → API DTO | `status_updates`, stage_bridge projections, UI badge maps |
| `IngestionFailureClass` | Permanent/retry policy | Stringly error handling in workers |
| `LargeDocumentProfile` (SPEC-038) | Timeout, concurrency, UX ETA, backend hint | Scattered page-count branches |
| `DefaultIngestionPersister` | Persist + compensate | Keep; add compensate failure telemetry |
| `TaskQueue` + claim store | Delivery | Channel-only wake special cases |

---

## SOLID touchpoints

| Principle | Application in this plan |
| --------- | ------------------------ |
| **S**RP | Split Convert vs Ingest processors; mapper ≠ cancel ≠ persister |
| **O**CP | New claim backend / delivery mode without rewriting `WorkerPool` loop body |
| **L**SP | Bridged/NotifyOnly/Local all satisfy `TaskQueue`; claim store behind trait |
| **I**SP | Narrow `TaskProcessor` per task type; stop fat PdfProcessing doing KG |
| **D**IP | Worker depends on `TaskStorage` + `TaskQueue` + `CancellationRegistry` traits/handles |

---

## Phase P0 — Controllability & status truth

**Status:** `IMPLEMENTED` (2026-07-17) — narrow P0 shipped; P0.3b closed by Phase P4  
**Causes:** 03, 10, 12 · **REQs:** 03, 04, 05, 06, 13, 15

| Step | Work | DoD | Done |
| ---- | ---- | --- | ---- |
| P0.1 | `PdfProcessingStatus::Cancelled` + mig `087_pdf_processing_status_cancelled.sql` | Round-trip parse/store | [x] |
| P0.2 | Cancel writers: `operations.rs`, vision abort in `pdf_processing.rs`, cancel→Cancelled in `task_impl.rs` | PDF cancel ≠ Failed | [x] |
| P0.3a | Thin `services/ingestion_status.rs` + HTTP cancel `failure_class` | Doc KV cancel taxonomy | [x] |
| P0.3b | Full `IngestionStatusMapper` / stage_bridge collapse | Fixture matrix | [x] (Phase P4) |
| P0.4 | UI: `failedCount` excludes cancelled (`document-manager.tsx`) | Separate chips | [x] |
| P0.5 | `is_cancel_failure_message` + vision cancel golden | Classifier covers vision | [x] |
| P0.6 | `contract_cancel_and_fairness` extended | 7+ tests green w/ `--features postgres` (PDF+doc KV) | [x] |
| P0.7 | All cancel paths sync doc KV (`sync_doc_cancelled_for_task`) | HTTP/WS/PDF/pipeline | [x] |

**P0 file map (code-assessed):**

| Area | Path |
| ---- | ---- |
| Enum | `edgequake-storage/src/pdf_storage.rs` |
| Migration | `edgequake/migrations/087_pdf_processing_status_cancelled.sql` |
| PDF cancel HTTP | `edgequake-api/src/handlers/pdf_upload/operations.rs` |
| Vision abort | `edgequake-api/src/processor/pdf_processing.rs` |
| Permanent-fail gate | `edgequake-api/src/processor/task_impl.rs` |
| Doc cancel helper | `edgequake-api/src/services/ingestion_status.rs` |
| HTTP task cancel | `edgequake-api/src/handlers/tasks.rs` |
| Cancel string DRY | `edgequake-tasks/src/ingestion_reliability.rs` → `is_cancel_failure_message` |
| UI | `edgequake_webui/.../document-manager.tsx` |
| Contracts | `edgequake-api/tests/contract_cancel_and_fairness.rs` |

**Non-goals P0:** SQL claim loop, task split, full stage_bridge collapse.

---

## Phase P1 — Restart durability (Postgres as delivery SSOT)

**Status:** `IMPLEMENTED` (2026-07-17) — `feat/spec057-p1-claim-lease`  
**Causes:** 01, 02, 05 · **REQs:** 01, 02, 05, 15

### P1 product rule (resolves SPEC-054 conflict)

```text
  Boot with claim path live:
    Pending          → leave Pending (claimable) — never auto-Failed
    Processing       → if lease expired → Interrupted/Failed (Reprocess)
    Cancelled        → never claim
  EDGEQUAKE_STARTUP_AUTO_RESUME:
    only controls whether stale Processing is reset to Pending (reclaim)
    vs Forced Failed — NOT whether Pending is discarded
  Channel           → wake only; claim is SSOT
```

| Step | Work | DoD | Done |
| ---- | ---- | --- | ---- |
| P1.1 | Worker claim path: `FOR UPDATE SKIP LOCKED` on Pending (and stale Processing); mig `088_task_lease_columns.sql` | Kill process with Pending → restart processes without manual requeue / without `STARTUP_AUTO_RESUME=1` | [x] |
| P1.2 | Keep channel/NOTIFY as wake only (`enqueue_with_delivery`) | Wake latency preserved; claim still works if wake missed | [x] |
| P1.3 | Dequeue/claim always re-reads task status; Cancelled ⇒ drop | Cancel Pending → restart → never runs | [x] |
| P1.4 | Lease TTL + reaper for stale Processing → Interrupted/Failed | No infinite orphans | [x] |
| P1.5 | UX: Interrupted empty state → Reprocess; keep auto-resume default **off** | Matches SPEC-054 product policy | [x] |
| P1.6 | Contract: restart-after-cancel + restart-pending-claim | Memory + Postgres SKIP LOCKED e2e | [x] |
| P1.7 | Boot recovery extract + Pending-survive tests | `orphan_task_recovery` unit tests | [x] |

**P1 verification**

```bash
cargo test -p edgequake-api --features postgres --test contract_cancel_and_fairness
cargo test -p edgequake-api --test contract_claim_and_restart
cargo test -p edgequake-tasks --lib claim
DATABASE_URL=... cargo test -p edgequake-tasks --features postgres --test postgres_claim_lease
cargo test -p edgequake-api orphan_task_recovery
```

**P1 file map:**

| Area | Path |
| ---- | ---- |
| Migration | `edgequake/migrations/088_task_lease_columns.sql` |
| Lease TTL | `edgequake-tasks/src/lease.rs` (`EDGEQUAKE_TASK_LEASE_TTL_SECS`, `lease_expires_at`) |
| Storage API | `claim_next` / `refresh_lease` / `release_claim` in `storage.rs`, `memory.rs`, `postgres.rs` |
| Worker | `edgequake-tasks/src/worker.rs` — wake→claim→fairness release→`refresh_lease` |
| Boot / reaper | `orphan_task_recovery.rs` + `main.rs` thin caller; lease-aware periodic check |
| Memory contracts | `contract_claim_and_restart.rs` |
| Postgres e2e | `postgres_claim_lease.rs` (SKIP LOCKED dual race) |
| Ops | `docs/ingestion-cancel-and-fairness.md` |

```text
  P1 claim loop
  ┌──────────┐   wake    ┌─────────────┐  SKIP LOCKED  ┌──────────┐
  │ enqueue  │──────────►│ wait / poll │──────────────►│ process  │
  │ + NOTIFY │           │ claim row   │               │ + lease  │
  └──────────┘           └─────────────┘               └──────────┘
         ▲                      │ status=Cancelled?
         │                      └── drop (no process)
         └──────── durable row is always source of truth
```
---

## Phase P2 — Stage split & asymptotics

**Status:** `IMPLEMENTED` (2026-07-17)  
**Causes:** 04, 06, 08, 11 · **REQs:** 07, 08, 09, 14

| Step | Work | DoD | Done |
| ---- | ---- | --- | ---- |
| P2.1 | `PdfProcessing` convert-only → enqueue `TaskType::Insert`; barrier = `markdown_content` | Convert success survives extract timeout | [x] |
| P2.2 | `LargeDocumentProfile` `convert_timeout_secs` + `ingest_timeout_secs` on task metadata | SPEC-038 budgets under stage split | [x] |
| P2.3 | Fairness clamp from **runtime extract** (`EDGEQUAKE_EXTRACT_PROVIDER` / hybrid) | Hybrid OpenAI LLM + Ollama extract clamp | [x] |
| P2.4 | Slim checkpoint bound + `re_embedding` stage (API/UI) | Honest resume progress | [x] |
| P2.5 | Single-flight + cancel chain Convert∪Insert; contracts | `contract_pdf_convert_ingest_split` + cancel extend | [x] |

**P2 verification**

```bash
cargo test -p edgequake-api --features postgres --test contract_pdf_convert_ingest_split
cargo test -p edgequake-api --features postgres --test contract_cancel_and_fairness
cargo test -p edgequake-pipeline --lib hybrid_openai_llm_ollama_extract
cargo test -p edgequake-api --lib p2_phase_timeouts
```

**P2 product rule**

```text
  PdfProcessing  = convert only → PDF Completed + markdown_content
  TaskType::Insert = KG ingest (extract/embed/merge)
  Ingest fail/timeout/cancel  → PDF Completed + markdown kept
  Cancel Convert              → also cancel linked Pending/Processing Insert
```

---

## Phase P3 — Scale-out & store hardening

**Status:** `IMPLEMENTED` (2026-07-17)

**Causes:** 07, 09 · **REQs:** 10, 11, 12

| Step | Work | DoD | Status |
| ---- | ---- | --- | ------ |
| P3.1 | `EDGEQUAKE_REPLICAS` + delivery gate; hydrating workers use `claim_next`; dual WorkerPool contract | Two pools never double-process | ✅ |
| P3.2 | Partial `MergeArtifacts` on vector upsert fail; KV DLQ + metric; double-compensate idempotent | Injected fail visible in metrics + KV | ✅ |
| P3.3 | `store_contention` assessor on queue-metrics + `/ready` blockers | Real pool util + quarantine SLOs | ✅ |
| P3.4 | Ops runbook in `docs/ingestion-cancel-and-fairness.md` + `.env.example` | Alert hooks documented | ✅ |

---

## Phase P4 — Status Truth SSOT

**Status:** `IMPLEMENTED` (2026-07-17)

**Causes:** 10 · **REQs:** 04, 05 (UI Stopping… → Cancelled) · closes **P0.3b**

| Step | Work | DoD | Status |
| ---- | ---- | --- | ------ |
| P4.1 | `IngestionStatusMapper` + ≥12-row fixture matrix | `cargo test -p edgequake-api --lib ingestion_status` | ✅ |
| P4.2 | Wire `display_status` / `ui_phase` on list/detail/track (+ cancel intent) | `contract_ingestion_status_mapper` | ✅ |
| P4.3 | Absorb `status_updates` legacy→unified into mapper helpers | Write path calls mapper | ✅ |
| P4.4 | FE prefers `display_status`; `ui_phase=stopping` → Stopping… | bun status unit tests | ✅ |
| P4.5 | Playwright cancel Stopping… → Cancelled (not Failed) | `e2e/spec057-cancel-status-ssot.spec.ts` | ✅ |

**P4 file map:**

| Area | Path |
| ---- | ---- |
| Mapper SSOT | `edgequake-api/src/services/ingestion_status_mapper.rs` |
| List/detail/track | `handlers/documents/query/{list,detail,track_status}.rs` |
| Write-path DRY | `processor/status_updates.rs` |
| FE badge | `edgequake_webui/src/components/documents/status-badge.tsx` |
| Contracts | `edgequake-api/tests/contract_ingestion_status_mapper.rs` |
| Playwright | `edgequake_webui/e2e/spec057-cancel-status-ssot.spec.ts` |

**Non-goals P4:** Temporal; Postgres `LISTEN`/`NOTIFY`; rewrite of core `DocumentStatus` enum.

---

## Dependency graph

```text
  P0.1 PDF Cancelled ──► P0.2 writers ──► P0.3 mapper ──► P0.4 UI
         │
         └──► P0.6 contracts

  P1.1 claim ──► P1.2 wake ──► P1.3 cancel re-read ──► P1.4 lease
         │
         └──► P3.1 multi-replica (requires claim)

  P2.1 split ──► P2.2 profile (timeout per phase)
  P0/P1 stable ──► P2 ──► P3
```

---

## Test plan (phased)

| Phase | Commands / artifacts |
| ----- | -------------------- |
| P0 | `cargo test -p edgequake-api --test contract_cancel_and_fairness`; PDF status unit tests; FE badge unit if present |
| P1 | New restart claim/cancel contracts; `EDGEQUAKE_STARTUP_AUTO_RESUME=0` Interrupted path |
| P2 | SPEC-038 repro script; hybrid provider clamp test; slim resume contract |
| P3 | `contract_multi_replica_claim`; `contract_compensate_observability`; `edgequake-storage --lib compensate`; `contract_spec026_task_delivery` |
| P4 | `cargo test -p edgequake-api --lib ingestion_status`; `contract_ingestion_status_mapper`; FE status tests; `pnpm exec playwright test e2e/spec057-cancel-status-ssot.spec.ts` |

---

## Explicit non-goals

- Adopting Temporal/external workflow engine  
- Renaming historical `011-pipeline-reliabilty` folder  
- Changing GraphRAG extraction prompts for quality  
- Enabling `EDGEQUAKE_STARTUP_AUTO_RESUME=1` by default  

---

## Success criteria (spec complete when implemented)

1. Cancel ⇒ Cancelled on task + doc + PDF; UI Stopping… then Cancelled.  
2. Process restart does not lose Pending work (claim) and does not revive Cancelled.  
3. Large born-digital PDF completes via EdgeParse under adaptive timeout.  
4. Multi-replica safe with SQL claim.  
5. Merge failure never leaves silent orphans; compensate failures are observable.  
6. All REQ-057-xx checked in [013](./013-cross-reference-matrix.md) with green proofs.

---

## Immediate next engineering mission (suggested)

P0–P4 closed. Residual non-blocking items (P3): hard-`Err` still uses `MergeArtifacts::default()` in some paths; hydrating workers lack lease heartbeat. Optional follow-up: Temporal / real Postgres `LISTEN`/`NOTIFY` (explicit non-goals).
