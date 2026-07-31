# Lens 3 — Database Expert: Job / Task / Attempt Model, Fencing, Deletion Saga

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). Normative for the logical model, state semantics, and the fencing protocol. Physical tuning belongs to Lens 4; process behaviour to Lens 5.
> 

## Diagnosing the model that exists

One table carries one flat row per unit of work, identified by `track_id`, with a JSON `task_data` payload. Everything relational about the domain lives inside that payload.

```
CURRENT (inferred from storage.rs, postgres.rs, document_task_cleanup.rs)

  tasks
  ├─ track_id            TEXT PK        "insert-…", "upload-…"
  ├─ task_type           TEXT
  ├─ status              TEXT           5 values only
  ├─ tenant_id           UUID
  ├─ workspace_id        UUID
  ├─ task_data           JSONB          ◄── document_id lives HERE
  │                                      ◄── pdf_id lives HERE
  │                                      ◄── metadata.document_id too
  ├─ retry_count         INT
  ├─ worker_id           TEXT           lease owner
  ├─ lease_token         UUID
  ├─ lease_expires_at    TIMESTAMPTZ
  ├─ fairness_hold_until TIMESTAMPTZ
  └─ created_at / updated_at

  Consequences:
   * "tasks for document D" is not a query, it is a scan plus Rust filtering
     (task_references_document probes three JSON paths)
   * "the pipeline for this PDF" is a payload search plus a bounded loop
     (apply_cancel_pdf_pipeline_tasks, for _ in 0..8)
   * retry history is a counter, not a record — the previous failure is overwritten
   * cancel intent has no column at all
```

Three normalisation failures, each with a direct operational cost: unindexable identity (delete cost, hub G4), unrepresented relationships (hub G11), and unrepresented intent (hub G1).

## Designing the target model

```
┌─────────────────┐      ┌────────────────────┐      ┌─────────────────┐
│ jobs             │ 1  n │ tasks              │ 1  n │ attempts         │
│─────────────────│──────│────────────────────│──────│─────────────────│
│ id           PK  │      │ id            PK   │      │ id           PK  │
│ tenant_id        │      │ job_id        FK   │      │ task_id      FK  │
│ workspace_id     │      │ parent_task_id FK  │      │ attempt_no       │
│ operation        │      │ operation          │      │ worker_id        │
│ subject_kind     │      │ fairness_class     │      │ lease_token      │
│ subject_id       │      │ state              │      │ lease_expires_at │
│ idempotency_key  │      │ available_at       │      │ started_at       │
│ state            │      │ cancel_requested_at│      │ finished_at      │
│ created_by       │      │ hold_until         │      │ outcome          │
│ created_at       │      │ document_id  GEN   │      │ failure_kind     │
└─────────────────┘      │ pdf_id       GEN   │      │ fence_epoch      │
                         │ fence_epoch        │      └─────────────────┘
                         │ cost_estimate      │
                         │ payload      JSONB │      ┌─────────────────┐
                         └────────────────────┘      │ task_events      │
                                                    │─────────────────│
  documents                                         │ task_id      FK  │
  ├─ id                                             │ seq              │
  ├─ fence_epoch  BIGINT NOT NULL DEFAULT 0  ◄──────│ kind             │
  └─ …            bumped by destructive ops        │ payload          │
                                                    └─────────────────┘
  GEN = generated column extracted from payload, indexed (Lens 4)
```

The existing `track_id` stays as the public identifier so no client breaks; it becomes a unique column on `tasks` rather than the primary key, and the `upload-` / `insert-` prefixes generated today remain valid.

### Why `attempts` is separate

A lease belongs to an execution, not to a task. Today `worker_id`, `lease_token`, and `lease_expires_at` sit on the task row, so each retry overwrites the record of the previous one and each heartbeat updates the same row the scheduler reads. Splitting attempts gives three things at once: an audit trail for `consecutive_timeout_failures` reasoning, a natural home for the fence epoch that an execution captured, and — as Lens 4 explains — relief from heartbeat write amplification on the table the claim query scans.

## Specifying the state semantics

```
state                terminal   claimable   meaning
────────────────────────────────────────────────────────────────
queued               no         yes*        waiting; * only if available_at <= now
held                 no         no          fairness parked until hold_until
leased               no         no          an attempt owns it, not started yet
running              no         no          attempt is executing
cancelling           no         no          cancel_requested_at set, draining
succeeded            YES        no          today's Indexed
failed               no         yes*        retryable; * available_at gate
cancelled            YES        no          drained and fenced
dead_letter          YES        no          attempts exhausted or non-retryable

Mapping from today, additive and non-breaking:
  Pending    → queued  (held when fairness_hold_until is set)
  Processing → leased, then running
  Indexed    → succeeded
  Failed     → failed, or dead_letter once retries are exhausted
  Cancelled  → cancelled
```

### The transition table is the single source of truth

```
            │ queued held leased running cancelling succ failed cancd dead
────────────┼────────────────────────────────────────────────
queued      │   ─     ✓     ✓      ─        ✓        ─     ─      ✓     ─
held        │   ✓     ─     ✓      ─        ✓        ─     ─      ✓     ─
leased      │   ✓     ─     ─      ✓        ✓        ─     ✓      ✓     ─
running     │   ✓     ─     ─      ─        ✓        ✓     ✓      ─     ─
cancelling  │   ─     ─     ─      ─        ─        ─     ─      ✓     ─
succeeded   │   ─     ─     ─      ─        ─        ─     ─      ─     ─   ◄ absorbing
failed      │   ✓     ─     ─      ─        ─        ─     ─      ✓     ✓
cancelled   │   ─     ─     ─      ─        ─        ─     ─      ─     ─   ◄ absorbing
dead_letter │   ✓     ─     ─      ─        ─        ─     ─      ─     ─   ◄ retry only

  queued  → cancelled directly: nothing has run, no drain needed
  running → queued: lease lost or expired; the attempt is abandoned, not the task
  running → cancelled is FORBIDDEN: it must pass through cancelling so that the
            drain acknowledgement and the fence are observable facts
```

This matrix is declared once in Rust (Lens 6 defines the function shape) and mirrored by a database trigger so that a rogue `UPDATE` cannot produce an illegal pair. Today the only enforcement is `mark_success` returning `false` for cancelled tasks plus a single test — convention, not constraint (hub G9).

## Fencing writes with a monotonic epoch

Cooperative cancellation cannot bound a write, because the writer may already be past its last check. A fence can.

```
INGEST ATTEMPT                                  DESTRUCTIVE OPERATION
──────────────                                  ─────────────────────
t0  read documents.fence_epoch = 7
    store 7 on the attempt row
t1  … chunk, embed …
                                                t2  UPDATE documents
                                                      SET fence_epoch = 8
                                                    WHERE id = D
t3  INSERT vectors …
    WHERE (SELECT fence_epoch FROM documents
           WHERE id = D) = 7        ────►  0 rows: write refused, attempt aborts

Every persist path becomes epoch-conditional. The check costs one indexed read
and converts an unbounded race into a deterministic refusal.
```

When to bump: on `document.delete`, `document.delete_batch`, `workspace.wipe`, and on `document.reprocess` that supersedes a prior pipeline. Never on retries — a retry is the same logical write and must keep its epoch, otherwise legitimate work self-destructs (see the risk table in Lens 1).

## Ordering the deletion saga

```
jobs.state for operation = document.delete

┌───────────┐
│ requested │── write the job row; return 202 to the caller
└─────┬─────┘
      ▼
┌─────────────────┐  UPDATE tasks SET cancel_requested_at = now(),
│ cancelling_deps  │    state = 'cancelling'
└─────┬────────────┘  WHERE document_id = $1        ◄ indexed column, not JSON
      │                  AND state NOT IN (terminal)
      │                then pg_notify('task_cancel', …) per row
      ▼
┌───────────┐  wait until, for every dependent task:
│ drained   │    state IN (terminal) OR its attempt lease_expires_at < now()
└─────┬─────┘  bounded by lease TTL, so the wait can never exceed 120 s
      ▼
┌───────────┐  UPDATE documents SET fence_epoch = fence_epoch + 1
│ fenced    │  from here the delete is a commitment: cancel returns 423
└─────┬─────┘
      ▼
┌───────────┐  vectors → graph → kv → relational, each step idempotent,
│ purging   │  each step recorded in task_events so a resume knows where it was
└─────┬─────┘
      ▼
┌───────────┐  count residue in every store; zero → done,
│ verified  │  non-zero → dead_letter + compensation_quarantine record
└─────┬─────┘
      ▼
┌─────────┐
│ done    │
└─────────┘
```

### Stop deleting the task rows

`document_task_cleanup::cancel_and_delete_task` performs `mark_cancelled` and then `storage.delete_task(&task.track_id)`. The row is the guard: `claim_next` refuses to claim a cancelled task, and the lease identifies who is still running it. Removing it removes both protections at the exact moment they are needed, and destroys the audit that a compliance persona requires.

Replacement rule: cancelled tasks are **marked and superseded**, never deleted inline. Add `superseded_by UUID NULL` so a reprocess can point at the pipeline it replaced. Physical removal is the job of `prune_terminal_tasks`, which already exists and already respects partitions, with a retention window per state.

## Guaranteeing tenant isolation in the model

| Rule | Enforcement |
| --- | --- |
| Every task row carries `tenant_id` and `workspace_id` | `NOT NULL`, no nullable escape hatch |
| No cross-tenant read | row-level security policy on `tenant_id`, not only application filters (`tenant_guard.rs`, `tenant_isolation.rs` stay as defence in depth) |
| Identity is never derived from a payload | `parse_explicit_workspace_uuid` becomes a validation step at creation, not a read-time inference |
| Fairness accounting keys | `(tenant_id, workspace_id, fairness_class)` as a real composite, replacing the XOR-mixed lane key (hub G6) |

## Where to read next

Indexes, partitions, generated columns, and claim SQL for this model are in Lens 4. Drain timing and lease TTL interactions are in Lens 5. Rust types and the transition function are in Lens 6. Epoch checks at persist points inside the model pipeline are in Lens 7. The contract that exposes `state` and `cancel_requested_at` is in Lens 2.