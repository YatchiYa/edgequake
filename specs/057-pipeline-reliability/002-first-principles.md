# 002 — First Principles

**Spec:** SPEC-057  
**Law:** Code is law — invariants below are what *must* hold; [003-code-is-law.md](./003-code-is-law.md) records what *does* hold today.

---

## Problem restated

Ingestion is a long-running, multi-store, multi-tenant workflow. Reliability is not “zero errors” — it is **predictable control**: admit, observe, cancel, resume/reprocess, and leave stores consistent under failure and restart.

---

## Invariants (must be true)

| ID | Invariant | Rationale |
| -- | --------- | --------- |
| INV-01 | **Durable intent** — A task that was accepted exists as a durable row until terminal. | Restarts must not invent amnesia. |
| INV-02 | **Delivery ≠ truth** — Queue channels may wake workers; they must not be the only copy of work. | Process death is normal. |
| INV-03 | **Cancel is cooperative and terminal** — After cancel, work must not restart; retries forbidden. | Controllability + cost control. |
| INV-04 | **Cancel intent is durable** — Pending/parked work must observe cancel across restart. | Race: cancel before worker registers token. |
| INV-05 | **One status story** — Task, doc KV, PDF, and UI stage agree on terminal semantics (incl. Cancelled). | Trust. |
| INV-06 | **Fairness parks, never thrash-requeues** — Excess tenant work waits without channel storms. Durable **fairness hold** makes parked work claim-invisible; claim prefers tenants with free lane capacity (FP-1…FP-5 below). | Multi-tenant SLO + max productive admission. |
| INV-07 | **Idempotent persist** — Re-running a stage must not corrupt KV/vector/graph (upsert + source-tracked merge). | At-least-once delivery. |
| INV-08 | **Saga leaves no queryable orphan** — On merge failure, compensate; on compensate failure, alert. | Dual-store consistency. |
| INV-09 | **Permanent failures do not burn retry budget** — Taxonomy SSOT (SPEC-045). | Cost + latency. |
| INV-10 | **Honest progress** — UI stages reflect real pipeline stages; ETA from cost model, not fake %. | UX reliability. |
| INV-11 | **Asymptotic class matches document** — Born-digital PDF must not default to O(pages×LLM) when O(pages) exists. | SPEC-038. |
| INV-12 | **Worker lease bounded** — A single task must not monopolize a worker/tenant slot beyond designed phases. | Fairness + timeout. |

---

## What must be true vs what code guarantees today

```text
  INVARIANT                         TODAY (code is law)
  ───────────────────────────────   ──────────────────────────────────────────
  INV-01 Durable task row           YES — Postgres tasks / SharedTaskStorage
  INV-02 Delivery ≠ sole truth      PARTIAL — ChannelTaskQueue is default wake;
                                    hydrate only if STARTUP_AUTO_RESUME=1
  INV-03 Cancel terminal            YES — TaskStatus::Cancelled, no retry
  INV-04 Durable cancel intent      NO — CancellationRegistry intents in-memory
  INV-05 One status story           NO — PDF enum lacks Cancelled; multi-layer stages
  INV-06 Fairness parks             YES — limiter park + durable fairness_hold_until
                                    (claim excludes holds; prefer under-cap tenants)
  INV-07 Idempotent persist         YES — upsert + merger (best effort)
  INV-08 Saga no orphans            PARTIAL — compensate on merge fail; crash window
  INV-09 Permanent taxonomy         YES — ingestion_reliability.rs (gaps → Unknown)
  INV-10 Honest progress             PARTIAL — stage bridge + SPEC-048; upload fake % history
  INV-11 Asymptotic routing         PARTIAL — EdgeParse path exists; Vision still default-ish
  INV-12 Bounded worker lease       NO — PDF convert + KG inline in one task
```

---

## First-principles decomposition

```text
                    ┌─────────────────────┐
                    │  Accept work        │  INV-01
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
        Durable row      Wake signal      Admission/backpressure
        (Postgres)       (channel/NOTIFY) (fairness + queue pressure)
              │                │                │
              └────────────────┼────────────────┘
                               ▼
                    ┌─────────────────────┐
                    │  Execute phases     │  INV-07, INV-11, INV-12
                    │  with checkpoints   │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
           Cancel           Fail class       Persist saga
           INV-03/04/05     INV-09           INV-08
                               │
                               ▼
                    ┌─────────────────────┐
                    │  Terminal + UX      │  INV-05, INV-10
                    └─────────────────────┘
```

---

## INV-06 — Fairness first principles (FP-1…FP-5)

| Law | Meaning |
| --- | ------- |
| **FP-1 Capacity before claim** | A task that cannot run under its tenant lane must not occupy claim bandwidth. |
| **FP-2 Tenant priority** | Among claimable work, prefer tenants with **free** ingest/lifecycle slots (durable in-flight count + configured max). Not a latency SLA or weighted fair-share %. |
| **FP-3 Dual-lane preserved** | Ingest vs lifecycle remain separate. Workspace-fair pick (SPEC-084 LAW-13) applies under that. |
| **FP-4 Orthogonal byte admission** | Global in-flight byte budget stays **after** the fairness slot; do not merge into the semaphore. |
| **FP-5 One park SSOT** | Park visibility lives in **storage** (`fairness_hold_until`); process-local park set is only duplicate-waiter safety, not the scheduling filter. |

**Priority guarantee by tenant (v1):** when tenant A is at ingest cap (active Processing leases **or** active fairness holds), pending work for tenant B with free capacity is claimed before A’s at-cap Pending (including within the same workspace); A’s held rows are not reclaim-stormed. Hold TTL expiry while still parked re-marks the hold. Park wake stages a lane permit by `track_id` before clear/send; cancel/skip drops that permit.

**Non-goals for INV-06:** weighted fair-share %, per-tenant latency SLOs, priority scores, or folding byte admission into lane semaphores.

Ops SSOT: [docs/ingestion-cancel-and-fairness.md](../../docs/ingestion-cancel-and-fairness.md).

---

## Design principles for the improvement plan

1. **Postgres is the source of truth** for task lifecycle (industry pattern: `FOR UPDATE SKIP LOCKED` claim).  
2. **Channel / NOTIFY is an accelerator**, never the ledger.  
3. **SRP:** `task_cancel` owns task-row+registry; status mapper owns cross-enum projection; persister owns saga.  
4. **OCP:** New delivery backends implement `TaskQueue` / delivery mode — do not fork worker loops.  
5. **DIP:** Workers depend on traits (`TaskProcessor`, `TaskQueue`, `TaskStorage`), not concrete channel types.  
6. **DRY:** One cancel apply path, one failure taxonomy, one large-doc profile driving timeout+UX.

---

## Non-goals (this assessment)

- Replacing Postgres with an external workflow engine (Temporal, etc.)  
- Perfect exactly-once across LLM providers (impossible; aim for idempotent side effects)  
- Changing GraphRAG extraction quality algorithms (separate from reliability mechanics)
- Weighted fair-share / latency SLOs for tenant scheduling (see INV-06 non-goals)

Next: [003-code-is-law.md](./003-code-is-law.md)
