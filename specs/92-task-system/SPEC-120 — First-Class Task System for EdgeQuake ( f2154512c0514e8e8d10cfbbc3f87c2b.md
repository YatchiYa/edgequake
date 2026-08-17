# SPEC-120 — First-Class Task System for EdgeQuake (Ingestion · Deletion · Reprocess)

> **Status (2026-07-29):** DESIGN + ORPHANED WIP — not wired / not releasable. See [README.md](./README.md) for the implementation checklist.
> **Live path today:** SPEC-057 `POST /api/v1/tasks/{id}/cancel` ([ingestion-cancel-and-fairness.md](../../docs/ingestion-cancel-and-fairness.md)). `/api/v1/operations/*` is not mounted.
> Scope: every durable operation on documents — convert, ingest, reprocess, reindex, delete, batch delete, workspace wipe.
> This page is the hub; each lens sub-page is normative for its own layer and cross-references the others.
> 

## Reading this specification set

Each sub-page attacks the same system from one professional lens. Read the hub first for the shared vocabulary, the honest assessment, and the state machines; then read the lens you own.

| Lens | Sub-page | Owns |
| --- | --- | --- |
| Product Owner | Lens 1 — Product Owner | Operation taxonomy, invariants as acceptance criteria, roadmap |
| Full Stack | Lens 2 — Full Stack | HTTP/WS contract, idempotency, event schema |
| Database Expert | Lens 3 — Database Expert | Logical model, Job/Task/Attempt, fencing epochs |
| Postgres Expert | Lens 4 — Postgres Expert | Claim SQL, partitioning, index and vacuum strategy |
| System Engineer | Lens 5 — System Engineer | Multi-replica correctness, leases, capacity, SLOs |
| Rust Expert | Lens 6 — Rust Expert | Trait decomposition, typestate, DRY/SOLID refactors |
| AI Engineer | Lens 7 — AI Engineer | Cooperative cancel in vision/extract/embed, cost fairness |
| UX and UI Expert | Lens 8 — UX and UI Expert | Status vocabulary, Stopping affordance, queue transparency |

---

## Establishing the first principles

Everything below is derived from six axioms. Where the current branch violates an axiom, the assessment names the file and the method.

1. **One truth for delivery.** A task runs because a durable row says it may run, never because a message arrived. The branch already states this in `docs/ingestion-cancel-and-fairness.md` ("Postgres task rows are the delivery SSOT. The in-memory channel is a wake signal only").
2. **One truth for intent.** Cancellation is a *fact about the row*, not a fact about a process. Any process-local set of cancelled identifiers is a cache, never the truth.
3. **Terminal means terminal.** Once a task reaches `cancelled`, `succeeded`, `failed`, or `dead_letter`, no code path may write to its side effects again. This requires a *fence*, not politeness.
4. **Fairness is an accounting problem.** Fair sharing needs a durable ledger of what each tenant consumed; per-process semaphores cannot decide a global question.
5. **Deletion is a saga, not a call.** Destroying data that a concurrent writer still holds requires ordered steps with a barrier: request → cancel dependents → fence → purge → verify.
6. **Every state a user can see must be a state the system stores.** If the interface shows *Stopping…*, the database must contain the reason it is stopping.

```
┌──────────────────────────────────────────────────────────────────────┐
│ AXIOM MAP → WHERE THE BRANCH STANDS                                  │
│                                                                      │
│  A1 delivery SSOT      ████████████████████░░  strong  (claim+lease) │
│  A2 intent SSOT        ███████░░░░░░░░░░░░░░░  weak    (in-memory)   │
│  A3 terminal fence     █████░░░░░░░░░░░░░░░░░  weak    (no epoch)    │
│  A4 fairness ledger    █████████░░░░░░░░░░░░░  partial (per-process) │
│  A5 deletion saga      ████████░░░░░░░░░░░░░░  partial (best effort) │
│  A6 stored UI state    ███████░░░░░░░░░░░░░░░  weak    (derived)     │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Mapping the current implementation

The task system on this branch spans one core crate and a fan of API services.

```
┌───────────────────────────── edgequake-tasks ─────────────────────────────┐
│                                                                           │
│  types/status.rs      TaskStatus{Pending,Processing,Indexed,Failed,        │
│                                  Cancelled}                               │
│                       TaskType{Upload,Insert,Scan,Reindex,PdfProcessing,  │
│                                KnowledgeInjection,Deletion,BatchDeletion, │
│                                WorkspaceWipe}                             │
│                       TaskType::fairness_class() → Ingest | Lifecycle     │
│                                                                           │
│  storage.rs           trait TaskStorage (≈20 methods)                     │
│                         create/get/update/delete/list/statistics          │
│                         claim_next / claim_next_with_policy               │
│                         refresh_lease / release_claim                     │
│                         mark_fairness_hold / clear_fairness_hold          │
│                         find_active_pdf_processing_task                   │
│                         find_active_pdf_ingest_task                       │
│                         prune_terminal_tasks / ensure_month_partitions    │
│  postgres.rs          Postgres impl (SKIP LOCKED claim, JSONB queries)    │
│  memory.rs            in-process impl for tests                           │
│                                                                           │
│  cancellation.rs      CancellationRegistry                                │
│                         tokens:        RwLock<HashMap<String,Token>>      │
│                         cancel_intents:RwLock<HashSet<String>>            │
│  tenant_limiter.rs    TenantConcurrencyLimiter                            │
│                         ingest / lifecycle / workspace_ingest lanes       │
│                         handoffs: Mutex<HashMap<String,FairnessPermit>>   │
│  fairness_hold.rs     fairness_hold_until TTL 30s, ClaimFairnessPolicy    │
│  admission.rs         InFlightByteBudget (AtomicU64, default 512 MiB)     │
│  lease.rs             TTL 120s default, min 30s                           │
│  queue.rs             ChannelTaskQueue (mpsc) + Unbounded variant         │
│  worker.rs            claim → try_acquire → admit → run → heartbeat       │
└───────────────────────────────────────────────────────────────────────────┘
                                     │
┌──────────────────────── edgequake-api/src/services ───────────────────────┐
│  task_cancel.rs             apply_task_row_cancel, apply_cancel_all_active │
│                             apply_cancel_pdf_pipeline_tasks               │
│  cancel_facade.rs           cancel_track_with_doc_and_pdf_chain           │
│  cancel_retract.rs          retract_indexes_for_task / _for_document      │
│  task_document_sync.rs      sync_doc_cancelled_for_task                   │
│  document_task_cleanup.rs   cancel_and_delete_task, purge_*_except        │
│  workspace_wipe_admission.rs single-flight wipe registry                  │
│  ingestion_status_mapper.rs display_status + ui_phase (derived)           │
│  orphan_task_recovery.rs    stale Processing reclaim                      │
│  reprocess_admission.rs     reprocess entry (no Reprocess task type)      │
└───────────────────────────────────────────────────────────────────────────┘
```

### The current task state machine, as coded

```
                 ┌─────────────────────────────────────────┐
                 │            (no row yet)                 │
                 └────────────────┬────────────────────────┘
                                  │ Task::new  (status = Pending)
                                  ▼
           ┌────────────────► ┌─────────┐ ◄──── release_claim
           │                  │ PENDING │ ◄──── boot reclaim (Processing→Pending)
           │                  └────┬────┘
mark_fairness_hold                 │ claim_next_with_policy
(fairness_hold_until = now+30s)    │   sets lease_token + worker_id
invisible to claim_next            ▼
           │                ┌────────────┐   refresh_lease every 60s
           └────────────────┤ PROCESSING ├──────────────┐
                            └──┬───┬───┬─┘              │ lease expiry
             mark_success ─────┘   │   └───── mark_cancelled
                                   │                    │
                 ┌─────────┐  mark_failed          ┌───────────┐
                 │ INDEXED │  (retry_count++)      │ CANCELLED │
                 └─────────┘       │               └───────────┘
                                   ▼                (never claimed)
                              ┌────────┐
                              │ FAILED │ ──can_retry()──► PENDING
                              └────────┘  (circuit breaker may veto)
```

The machine has no state for *cancel requested but work still running*. The interface nonetheless shows **Stopping…**, and that phase is computed at read time by `IngestionStatusMapper` as `ui_phase`, from a process-local cancel intent. Axiom 6 is violated: the visible state is not a stored state.

---

## Assessing the implementation honestly

### What is genuinely good

| Strength | Evidence |
| --- | --- |
| Claim-and-lease delivery is correct in shape | `TaskStorage::claim_next_with_policy` with `FOR UPDATE SKIP LOCKED`, `refresh_lease(track_id, worker_id, lease_token, ttl) -> bool` returning ownership loss, `release_claim` on park |
| Cancel entry points were deliberately deduplicated | `task_cancel::apply_task_row_cancel` is the single row-level primitive; `cancel_facade::cancel_track_with_doc_and_pdf_chain` composes row cancel, document sync, index retract, and the Convert∪Insert chain |
| Cancel intent survives the token gap | `CancellationRegistry::register` pre-cancels a token when an intent already exists, so a task that starts after the cancel still stops immediately |
| Lifecycle work is isolated from model work | `TaskType::fairness_class()` puts `Deletion`, `BatchDeletion`, `WorkspaceWipe` on a `Lifecycle` lane so a delete cannot be starved by, or starve, vision and extraction |
| Fairness park is claim-invisible | durable `fairness_hold_until` (`fairness_hold.rs`, TTL 30 s) plus hold exclusion in `claim_next`, which removes the classic release/reclaim storm |
| Byte admission is separate from slot admission | `admission.rs::InFlightByteBudget` ordered *after* the fairness permit, so a few very large documents cannot exhaust memory even with free slots |
| Retry has a damper | `consecutive_timeout_failures` with `circuit_breaker_tripped`, and progress-aware timeouts that do not advance the breaker |
| Operational hygiene exists | `prune_terminal_tasks`, `ensure_month_partitions`, keyset cursor support in `Pagination::has_keyset_cursor` |

### Where it breaks under first principles

**G1 — Cancellation intent is process-local (violates A2).**

`CancellationRegistry` stores intent in `cancel_intents: Arc<RwLock<HashSet<String>>>`. With `EDGEQUAKE_REPLICAS>1` and `EDGEQUAKE_TASK_DELIVERY=bridged`, a cancel served by replica A cannot signal the in-flight `CancellationToken` living in replica B. The row becomes `Cancelled`, but the running extraction keeps burning provider quota until it finishes, because nothing in the run loop re-reads the row. The branch documentation concedes this ("Cancel intents are process-local"). The lease heartbeat is the natural carrier and currently returns only `bool`.

**G2 — There is no fence, so cancelled and deleted work can resurrect (violates A3).**

Cancel is cooperative at `.await` points, and `cancel_retract.rs` compensates by retracting indexes immediately. Both are best-effort. Nothing prevents a straggler task that passed its last checkpoint from writing chunks, vectors, or graph edges *after* the retract or after the delete cascade finished. There is no monotonic `epoch` on the document that writers must present.

**G3 — Deletion cancels first, but then destroys its own evidence (violates A5).**

`document_task_cleanup::cancel_and_delete_task` cancels the registry entry, marks the row cancelled, then calls `storage.delete_task(&task.track_id)`. Deleting the row removes the very guard that keeps the work from being claimed again, removes the lease that identifies the owner, and removes the audit trail. There is also no acknowledgement barrier: the cascade proceeds without waiting for the dependent ingest to confirm it stopped.

**G4 — The deletion pre-scan does not scale.**

The same function pages with `page_size: 10_000` over all tasks in the workspace and then filters in Rust via `task_references_document`, which pokes at `task_data.existing_document_id`, `task_data.document_id`, and `task_data.metadata.document_id`. This is an O(tasks-in-workspace) scan with JSON inspection outside the database, on the latency path of every single delete, while `Pagination` already offers a keyset cursor that this call site ignores.

**G5 — Fairness is decided with per-process state (violates A4).**

`TenantConcurrencyLimiter` holds `HashMap<Uuid, Arc<Semaphore>>` per lane, and `ClaimFairnessPolicy` compares active leases plus holds against a static cap. Consequences: lane accounting is best-effort across replicas; the policy expresses *cap*, never *share*; there is no tenant weight, no deficit or virtual-time accounting, so a tenant that has consumed ten hours of GPU time is treated exactly like a tenant that arrived one second ago. Starvation is invisible because no metric records per-tenant queue-wait percentiles.

**G6 — The workspace lane key is a hash collision waiting to happen.**

`workspace_lane_key` mixes two UUIDs with `t ^ w.rotate_left(17) ^ 0x0840_0316`, with the comment "unique enough for semaphore map keys in-process". Two distinct `(tenant, workspace)` pairs can collide and silently share a concurrency lane. `Uuid::new_v5` over a namespace costs nothing here.

**G7 — The handoff map can leak a permit.**

`handoffs: Arc<Mutex<HashMap<String, FairnessPermit>>>` stages a permit by `track_id` between park wake and claim. If the woken worker never claims — process death, claim lost to a sibling replica, wake lost — the permit is retained until something calls `take_handoff` for that exact identifier. There is no TTL and no reaper, so the lane silently shrinks.

**G8 — The state enum is too small for the product.**

`TaskStatus` has five variants. The product needs, and the interface already pretends to have, at least: *held* (fairness parked), *cancelling* (intent recorded, work draining), *retry scheduled*, and *dead letter*. Today `Pending` means "never started", "parked", and "waiting for a retry" simultaneously, so no query can distinguish a starved tenant from a backlog.

**G9 — Transitions are enforced by convention, not by a machine.**

Guards live inside individual methods: `mark_success` returns `false` when the task is `Cancelled`, and one unit test asserts `Cancelled ↛ Indexed`. There is no single transition table, no exhaustive matrix, and no database-side constraint. Any new call site can write an illegal status.

**G10 — `TaskStorage` is a fat interface (violates ISP and SRP).**

One trait carries persistence, querying, claiming, leasing, fairness holds, metrics, pruning, partition maintenance, *and* PDF-specific lookups. `find_active_pdf_processing_task` and `find_active_pdf_ingest_task` are almost identical nested paging loops — a literal DRY violation inside the port itself, and they encode document-format knowledge in a generic storage abstraction.

**G11 — Task relationships are implicit.**

The convert-then-ingest split is real (`PdfProcessing` then `Insert`), but the link is discovered by searching for a `pdf_id` inside JSON payloads, and cancellation walks it with a `for _ in 0..8` bounded loop in `apply_cancel_pdf_pipeline_tasks`. There is no `parent_task_id`, no dependency edge, no job aggregate, so no query can answer "show me this document's pipeline" without payload archaeology.

**G12 — Reprocess is not a first-class operation.**

The user-visible verb exists (`reprocess_admission.rs`, `reprocess_stage_reset.rs`, `interrupted_restart.rs`), yet `TaskType` has no `Reprocess` variant, so reprocessing is indistinguishable from a first ingest in metrics, fairness, and audit.

**G13 — Retry scheduling has no time dimension.**

Eligibility is "pending, or processing with an expired lease". There is no `available_at`, so no exponential backoff and no jitter; a failing provider is retried at claim speed. There is no dead-letter destination either, so exhausted tasks sit in `Failed` mixed with genuine user-visible failures.

**G14 — The queue abstraction advertises delivery it must not provide.**

`trait TaskQueue` exposes `send`, `receive`, `try_receive`, `size`. Since Postgres is the delivery truth, this trait should model a wake signal only; its current shape invites exactly the mistake the documentation warns about ("Never process from a channel payload without claim"). `UnboundedChannelTaskQueue` additionally offers unbounded memory growth.

**G15 — Cross-replica observability undercounts.**

`cancel_intent_count`, `tenant_park_waiters`, and byte admission are per-process counters surfaced through `/pipeline/queue-metrics`. With several replicas, each endpoint reports a shard of reality, and there is no starvation or fairness-error metric at all.

---

## Designing the target system

### Three-level model

```
JOB  (what the user asked for)            e.g. "ingest report.pdf", "delete 40 docs"
 │      id, tenant_id, workspace_id, operation, idempotency_key, state
 │
 ├── TASK (a unit of scheduled work)      e.g. convert, ingest, purge_vectors
 │     │  id, job_id, parent_task_id, operation, fairness_class,
 │     │  state, available_at, cancel_requested_at, fence_epoch
 │     │
 │     └── ATTEMPT (one execution)         one row per try, immutable
 │            id, task_id, worker_id, lease_token, started_at, outcome
 │
 └── EVENT (append-only audit + stream source)
        job_id, task_id, seq, kind, payload, at
```

Why three levels: a *job* is the only thing the user can name, a *task* is the only thing a scheduler can rank, and an *attempt* is the only thing a lease can own. Collapsing them, as the branch does today, is what forces payload archaeology (G11) and destroys retry history (G13).

### Target task state machine

```
                              ┌───────────┐
              enqueue ───────► │  QUEUED   │ ◄────────── retry (available_at = t+backoff)
                              └─────┬─────┘
                    fairness at cap │      │ claim (SKIP LOCKED, quota rank)
                                    ▼      ▼
                              ┌───────────┐        ┌───────────┐
                              │   HELD    │───────►│  LEASED   │
                              └───────────┘ wake   └─────┬─────┘
                              hold_until TTL              │ start attempt
                                                          ▼
                                                    ┌───────────┐
                        cancel_requested_at set ───►│  RUNNING  │
                                 (any state)        └──┬────┬───┘
                                                       │    │
                              ┌────────────────────────┘    │ ok
                              ▼                             ▼
                        ┌────────────┐              ┌─────────────┐
                        │ CANCELLING │              │  SUCCEEDED  │
                        └──────┬─────┘              └─────────────┘
                    drain ack  │  fence_epoch bumped
                               ▼
                        ┌────────────┐
                        │ CANCELLED  │
                        └────────────┘

                        ┌────────────┐  attempts left   ┌───────────┐
                        │   FAILED   │─────────────────►│  QUEUED   │
                        └──────┬─────┘                  └───────────┘
                               │ attempts exhausted / non-retryable
                               ▼
                        ┌──────────────┐
                        │ DEAD_LETTER  │
                        └──────────────┘

Terminal set = { SUCCEEDED, CANCELLED, DEAD_LETTER }
Invariant T1: no transition out of a terminal state, ever.
Invariant T2: CANCELLING is reachable from QUEUED, HELD, LEASED, RUNNING only.
Invariant T3: entering CANCELLED requires fence_epoch(document) > epoch held by any attempt.
```

### Deletion as an explicit cancel-first saga

```
 DELETE /documents/{id}
        │
        ▼
 ┌────────────────────┐   1. create Job(operation=delete) + Task(purge)
 │  REQUESTED         │      one durable row, idempotency_key = (doc,epoch)
 └─────────┬──────────┘
           ▼
 ┌────────────────────┐   2. set cancel_requested_at on every task whose
 │  CANCELLING_DEPS   │      document_id column (not JSON!) matches
 └─────────┬──────────┘      pg_notify wakes every replica
           ▼
 ┌────────────────────┐   3. wait until deps are terminal OR their leases
 │  DRAINED           │      have expired — bounded wait, never unbounded
 └─────────┬──────────┘
           ▼
 ┌────────────────────┐   4. documents.fence_epoch += 1
 │  FENCED            │      every writer must present the epoch it read;
 └─────────┬──────────┘      stale writers now fail closed
           ▼
 ┌────────────────────┐   5. vectors → graph → kv → relational
 │  PURGING           │      each step idempotent, each step recorded
 └─────────┬──────────┘
           ▼
 ┌────────────────────┐   6. verify counts are zero; else DEAD_LETTER with
 │  VERIFIED / DONE   │      a quarantine record for the operator
 └────────────────────┘

Compare with today: cancel_and_delete_task cancels a process-local registry,
marks the row cancelled, then DELETES the row — steps 3, 4 and 6 do not exist.
```

### Cross-replica cancellation path

```
POST /tasks/{id}/cancel  (replica A)
       │
       │ 1. UPDATE tasks SET cancel_requested_at = now() WHERE id = $1
       │    AND state NOT IN (terminal)                       ← durable intent
       │
       ├─ 2. pg_notify('task_cancel', id)   ← fast path, best effort
       │        │
       │        └──► replica B LISTEN handler → registry.cancel(id)
       │                                       → CancellationToken.cancel()
       │
       └─ 3. slow path guarantee: refresh_lease returns
              LeaseVerdict::{Renewed, Lost, CancelRequested}
              so the owner learns within one heartbeat (≤60 s)
              even if NOTIFY was lost or the replica just booted
```

This keeps the existing `CancellationRegistry` as what it always was — a fast local cache — while moving the truth into the row, and it upgrades `refresh_lease`'s `bool` into the carrier that closes G1 without a new subsystem.

### Fairness as a durable ledger

```
CLAIM RANKING (evaluated inside one SKIP LOCKED statement)

  rank by:  1. lane quota respected?      active(tenant,lane) < quota(tenant,lane)
            2. deficit round robin        vruntime(tenant) ascending
                                          vruntime += cost / weight
            3. workspace interleave       round robin across workspaces
            4. arrival                    created_at ascending

  where cost is not 1 but the resource actually consumed:
     convert  → pages × vision_cost
     ingest   → bytes and tokens
     delete   → estimated rows touched
```

Slot counting answers "how many", never "how much". A tenant running two 900-page vision conversions consumes vastly more scarce capacity than a tenant running two one-page inserts, yet today they are identical to the scheduler. The ledger lives in Postgres so it is correct across replicas, and the per-process semaphore stays as a local admission cache.

---

## Sequencing the improvement plan

| Phase | Goal | Concrete changes | Closes |
| --- | --- | --- | --- |
| **P0** Make cancel true | Cancel works with more than one replica | Add `cancel_requested_at`; `apply_task_row_cancel` writes it before touching the registry; `refresh_lease` returns a verdict enum; add `LISTEN/NOTIFY` bridge | G1, A2 |
| **P0** Stop resurrection | No writes after terminal | Add `documents.fence_epoch`; every persist path presents its epoch; reject stale writers; keep `cancel_retract` as compensation only | G2, A3 |
| **P0** Fix delete ordering | Deletion never destroys its own guard | Replace `delete_task` in `cancel_and_delete_task` with mark-and-supersede; add the drained barrier before purge | G3, A5 |
| **P1** Make delete cheap | Delete is O(matching tasks) | Promote `document_id`, `pdf_id`, `parent_task_id` to generated columns with indexes; rewrite `task_references_document` as SQL; use the keyset cursor already in `Pagination` | G4 |
| **P1** Grow the machine | States match reality | Extend `TaskStatus` with `Held`, `Cancelling`, `DeadLetter`; add `available_at`; introduce one `transition()` function plus a database `CHECK`, and generate the matrix test from it | G8, G9, G13 |
| **P1** Name the operations | Reprocess is first class | Add `TaskType::Reprocess`; keep `fairness_class()` as the single mapping | G12 |
| **P2** Fairness ledger | Share, not just cap | Durable per-tenant lane counters, `tenant_weights`, cost-weighted `vruntime` in the claim statement; replace the XOR lane key with `Uuid::new_v5`; add TTL and a reaper to the handoff map | G5, G6, G7, A4 |
| **P2** Job graph | Pipelines are queryable | Introduce `jobs` and `parent_task_id`; delete the `for _ in 0..8` chain walk; cancel by job, not by payload search | G11 |
| **P2** Split the port | Traits obey ISP | `TaskRepository`, `TaskClaimer`, `LeaseKeeper`, `QueueMetricsReader`, `TaskAdmin`; move PDF finders to a `PdfTaskQueries` adapter; collapse the two duplicate finders into one generic helper | G10 |
| **P3** Honest telemetry | Fairness is measurable | Per-tenant queue-wait percentiles, starvation counter, cross-replica aggregation, `cancel_to_stop_seconds` histogram | G15 |
| **P3** Interface truth | Stored state drives badges | `ui_phase` derived from `cancel_requested_at` and `state`, not from a process-local set; wake modes narrowed to a `WakeSignal` trait | G6 in interface, G14, A6 |

### Acceptance invariants for the whole programme

| ID | Invariant | How it is proven |
| --- | --- | --- |
| INV-1 | A cancel accepted by any replica stops work on every replica within one heartbeat | integration test with two workers, NOTIFY suppressed |
| INV-2 | No side effect is written for a task after it becomes terminal | fence epoch test: pause a writer, cancel, resume, assert rejection |
| INV-3 | Deletion never completes while a dependent ingest still holds a valid lease | saga test with a slow ingest |
| INV-4 | Terminal states are absorbing | generated transition matrix test plus database constraint |
| INV-5 | A single tenant cannot exceed its weighted share for more than one claim cycle | soak test, two tenants, 10:1 backlog |
| INV-6 | Every user-visible status maps to exactly one stored `(state, cancel_requested_at)` pair | contract test against the OpenAPI snapshot |
| INV-7 | Delete latency is independent of workspace task count | benchmark at 10², 10⁴, 10⁶ rows |

---

## Applying DRY and SOLID deliberately

| Principle | Current honest score | Target rule for this system |
| --- | --- | --- |
| Single responsibility | Mixed — `cancel_facade` is a clean composer, but `TaskStorage` does eight jobs | One trait per reason to change; the cancel facade stays the only composer |
| Open–closed | Weak — adding an operation means touching enums, SQL fragments, and mappers | Operation descriptors registered in one table drive fairness class, timeout, and interface label |
| Liskov | At risk — `memory.rs` default `find_active_pdf_*` scans while `postgres.rs` queries; behaviour differs in complexity and race windows | Port contract tests run against every implementation |
| Interface segregation | Violated (G10) | Split into five narrow ports |
| Dependency inversion | Good — API services depend on `SharedTaskStorage` and `CancellationRegistry`, not on Postgres | Keep; add a `Clock` and `Notifier` port so time and wakeups are testable |
| DRY | Mostly honoured for cancel, violated for lookups (`find_active_pdf_processing_task` versus `find_active_pdf_ingest_task`), lifecycle SQL fragments, and status derivation | One generic active-task query; one transition table; one status vocabulary shared by Rust, SQL, OpenAPI, and the interface |

---

## Cross-reference index

| Gap | Primary lens | Supporting lenses |
| --- | --- | --- |
| G1 cancel not durable | System Engineer | Rust Expert, Full Stack |
| G2 no fence | Database Expert | AI Engineer, Postgres Expert |
| G3 delete destroys guard | Database Expert | Product Owner, System Engineer |
| G4 delete pre-scan | Postgres Expert | Database Expert |
| G5 fairness per process | Postgres Expert | System Engineer, Product Owner |
| G6 lane key collision | Rust Expert | — |
| G7 handoff leak | Rust Expert | System Engineer |
| G8 state enum too small | Product Owner | UX and UI Expert, Database Expert |
| G9 no transition machine | Rust Expert | Database Expert |
| G10 fat trait | Rust Expert | — |
| G11 implicit relations | Database Expert | Full Stack, UX and UI Expert |
| G12 reprocess not typed | Product Owner | Full Stack |
| G13 no backoff or dead letter | System Engineer | Product Owner |
| G14 queue trait misleading | Rust Expert | System Engineer |
| G15 telemetry shards | System Engineer | Product Owner |

[Lens 1 — Product Owner: Operations, Invariants, Roadmap](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%201%20%E2%80%94%20Product%20Owner%20Operations,%20Invariants,%20Roa%2009217020d4354c7b8c31f264dfa6ced1.md)

[Lens 2 — Full Stack: API Contract, Idempotency, Event Stream](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%202%20%E2%80%94%20Full%20Stack%20API%20Contract,%20Idempotency,%20Eve%20f6777bdb6996404099b2040c693b4268.md)

[Lens 3 — Database Expert: Job / Task / Attempt Model, Fencing, Deletion Saga](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%203%20%E2%80%94%20Database%20Expert%20Job%20Task%20Attempt%20Model,%20F%201dcfb9507f214f7db5594a298bb45eba.md)

[Lens 4 — Postgres Expert: Claim SQL, Fair Queueing, Indexes, Vacuum](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%204%20%E2%80%94%20Postgres%20Expert%20Claim%20SQL,%20Fair%20Queueing,%20d9a87d09e6064dd09754641f560352f6.md)

[Lens 5 — System Engineer: Multi-Replica Correctness, Leases, Capacity, SLOs](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%205%20%E2%80%94%20System%20Engineer%20Multi-Replica%20Correctness%20e19deb163905459ba14ff599443382c7.md)

[Lens 6 — Rust Expert: Trait Decomposition, Typestate, DRY and SOLID Refactors](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%206%20%E2%80%94%20Rust%20Expert%20Trait%20Decomposition,%20Typestat%20952387a7830d49cf97df06f86b16b4b2.md)

[Lens 7 — AI Engineer: Cooperative Cancellation, Checkpoints, Cost-Weighted Fairness](SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(/Lens%207%20%E2%80%94%20AI%20Engineer%20Cooperative%20Cancellation,%20Che%20fc6769207d9041ad90fe8f531f98cd5d.md)