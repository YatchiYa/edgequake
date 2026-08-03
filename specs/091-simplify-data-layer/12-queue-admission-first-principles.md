# 12 — Queue & Admission: First Principles

> Method: same as [02](02-first-principles.md) — reduce the ingestion-scheduling problem to what a provider-bound pipeline *minimally requires*, state those requirements as axioms, derive enforceable laws (`LAW-Q1..Q7`), and anchor each law to (a) the code that today honors or violates it (verified at the current branch, cited path:line) and (b) queueing-theory and systems references. Scope: the task queue, worker admission, and provider concurrency of the **ingestion** path. Locked by [LD-11..LD-13](README.md#locked-decisions).

## WHY (Five WHYs)

1. **Why do bulk uploads against a local model degrade into hours-long stalls with no user signal?** Because 100 uploads are all admitted, then drained at the provider's serial rate while the UI can only show "processing".
2. **Why does nothing push back earlier?** Because admission limits are *tenant* counts (1–2 tasks/tenant), while the true bottleneck — the local model — is guarded only by a process-local semaphore of 2 that no replica knows about and no enqueue path consults.
3. **Why is the provider guard process-local and disconnected from admission?** Because it grew as an OOM/overload patch (`local_inference_gate.rs`), not as a designed scarce-resource budget; the queue, the fairness lanes, and the gate each carry an independent opinion of "how much is too much".
4. **Why do independent opinions exist?** Because there is no state machine and no capacity SSOT: task status is mutated in four places by three mechanisms, and concurrency numbers are resolved in five files with five defaults.
5. **Why no state machine / capacity SSOT?** Because the queue evolved feature-by-feature (cancel, fairness, lease, vision gate) without a first-principles model of *what is scarce* and *what states a unit of work can be in*. This document states that model.

```ascii
 CAUSAL CHAIN (today, verified in code)
 tenant caps (tenant_limiter.rs) ─┐
 per-task extraction sem (extraction.rs:41) ─┼─▶ 5 independent "limit" opinions
 process-local gate =2 (local_inference_gate.rs:18) ─┘        │
                                                               ▼
                                   provider (Ollama) overloaded OR idle-starved
                                                               │
        unbounded DB queue + 100-slot wake channel (queue.rs:84)│
                                                               ▼
                          101st upload hangs silently; ETA impossible;
                          cancel/delete/duplicate handled by 3 ad-hoc mechanisms
```

## Axioms

1. **The scarce resource is provider inference capacity, not tenants.** For local models (Ollama/LM Studio) execution is near-serial; for cloud providers it is rate-limited by account. Every tenant, worker, and pipeline stage competes for the *same* provider.
2. **A unit of work is a state machine; illegal transitions must be unrepresentable.** A task that can be mutated by raw SQL in one place and struct methods in another will eventually occupy an impossible state.
3. **Queue depth is a derived signal, never an independent fact.** The honest bound on a queue is a function of measured drain rate and target latency (Little's Law), not a constant someone chose.
4. **Fairness is fair-sharing of the scarce resource.** Capping tenants does not protect the provider; dividing the provider budget among active tenants does both.
5. **An intent outlives the process that received it.** Cancel and delete are durable intents checked at every stage boundary, not in-memory signals checked when convenient.

## From axioms to enforceable laws

| Law | Statement | Derived from | Anchor | Today |
| --- | --- | --- | --- | --- |
| **LAW-Q1 — Capacity is derived, not guessed** | Every concurrency figure (worker threads, lane weights, extraction/embed/vision fan-out, queue bound) resolves from **one** `ProviderProfile` SSOT, seeded by measurement of the configured provider. | Axiom 1, 3 | Little's Law \(L = \lambda W\); overload control (SRE workbook, ch. 21) | **Violated** — five resolvers, five defaults: `pipeline/config.rs:177-306`, `local_inference_gate.rs:15-18`, `core/resource/budget.rs:73,149`, `tasks/admission.rs:19-25`, Makefile profiles (F-091-18, F-091-20) |
| **LAW-Q2 — Single transition authority** | Every task-status mutation passes through one state-machine module (`TaskEvent` → guard → next state). Raw SQL `SET status=…` outside it is forbidden; the transition table is exhaustively tested. | Axiom 2 | State-machine pattern; SPEC-083 X-29 (defect: missing guards, "FIXED" but partial) | **Violated** — transitions in `types/task.rs:191-302` (methods), `postgres.rs:653-853` (raw SQL), `worker.rs:792-794` (field mutation), `orphan_task_recovery.rs:155-217` (boot SQL); `Failed` is ambiguously retryable *and* terminal (F-091-17) |
| **LAW-Q3 — Provider budget is cluster-global** | In-flight inference capacity is leased from a Postgres slot ledger (`FOR UPDATE SKIP LOCKED` + TTL + fencing), so N replicas cannot multiply provider load by N. Slots are attributable (provider, workspace, task) and reclaimable on crash. | Axiom 1 | Same pattern as task `claim_next` (`postgres.rs:653-791`) and the migration-engine lease ([07](07-migration-engine.md)) | **Violated** — `local_inference_gate.rs` is one process-wide `tokio::sync::Semaphore` (2 permits); extraction semaphore is *per pipeline run* (`extraction.rs:41-43`); 2 replicas ⇒ 4 concurrent Ollama calls, silently (F-091-18) |
| **LAW-Q4 — Bounded, honest queue** | The pending queue has a derived bound: `max_pending = drain_rate × target_wait`. Admission beyond the soft bound still succeeds (LD-12) but is *labeled* — the response carries `queue_position` and an ETA computed from measured drain rate. The wake channel never silently blocks an HTTP handler. | Axiom 3 | Little's Law; bounded queues + load shedding (SRE) | **Violated** — no enqueue admission at all; backpressure is a 100-slot channel whose `send().await` hangs the 101st upload (`queue.rs:84-94`, wired at `state/postgres.rs:414`); `QueueMetrics.rate_limited` is hardcoded `false` (`postgres.rs:948`) (F-091-19) |
| **LAW-Q5 — Tenants fair-share the provider** | Tenant/workspace lanes carry **weights**, not hard caps; the provider budget is divided among *active* tenants (deficit round-robin). A single active tenant receives the whole budget. | Axiom 4 | DRR (Shreedhar & Varghese, 1995); weighted fair queueing | **Violated** — `TenantConcurrencyLimiter` enforces hard caps (ingest 1–2, lifecycle 4, per-workspace nested cap 1: `tenant_limiter.rs:91-166`, `pipeline/config.rs:218-223`), which under-feed a 2-permit provider *and* fail to protect it when caps exceed capacity (F-091-20) |
| **LAW-Q6 — Identity is idempotent end-to-end** | An upload's identity is its content hash within a workspace; a duplicate submitted while the original is queued/processing returns the in-flight identity and enqueues nothing. Single-flight registry + DB unique constraint are two layers of the *same* law. | Axiom 2 (identity) | Existing mechanisms, honored: checksum dedup (`pdf_upload/upload.rs:394-411`), `PdfAdmissionRegistry` (`services/pdf_admission_registry.rs:13-22`), hash dedup (`document_admission.rs:140-179`) | **Mostly honored** — three mechanisms, three code paths; the law is kept by making them one contract with one conformance test (EC-19) |
| **LAW-Q7 — Lifecycle intents are durable and preemptive** | Cancel and delete intents persist (survive restart), are evaluated at *every* stage boundary (claim, retry requeue, fairness park, pipeline stage), and take precedence over admitting new work. Delete ⇒ cancel-intent on the in-flight task, then cascade. | Axiom 5 | Existing registry (`cancellation.rs:57-177`), deletion cascade (`services/document_deletion.rs:392-414`) | **Partially honored** — checks exist at claim/retry/park (`worker.rs:468-489,807-819,1070-1078`) but are convention, not law; no conformance test enumerates the boundary set (EC-17/18) |

## Capacity math (LAW-Q1, made concrete)

```ascii
 PROVIDER PROFILE (resolved ONCE, SSOT — admission_resolver.rs)
 ┌─────────────────────────────────────────────────────────────────────┐
 │  provider = ollama(gemma3)  │  provider = openai(gpt-5-nano)        │
 │  measured in-flight max  B  │  B = account RPM / avg call seconds   │
 │  (e.g. B = 2 for 1 GPU)     │  (e.g. B = 32)                        │
 └───────────────┬─────────────────────────────────────────────────────┘
                 │ derives (single resolver — every number below is f(B))
                 ▼
   worker_threads      = clamp(2B, 4, 32)          workers wait on slots
   extraction fan-out  = clamp(B, 1, 8) per task   today: 2 local / 16 cloud
   embed/merge fan-out = clamp(B/2, 1, 8)          today: 1/2 local
   vision jobs         = clamp(B/2, 1, 4)          today: 1-2 local
   queue soft bound    = λ̂ × target_wait           λ̂ = EWMA drain rate
   tenant lane weight  = 1 (equal shares, DRR)     single tenant ⇒ full B

 Little's Law:  L = λW  ⇒  ETA(position p) = p / λ̂
   where λ̂ = measured completed-tasks/min (EWMA, τ = 10 min)
```

Every number in the right-hand column of the "today" rows is a *separate env var with a separate default* — the LAW-Q1 violation in one glance. After QW2, they are all `f(B)`.

## DRY / SOLID / SSOT mapping

```ascii
 LAW              DRY                              SOLID                       SSOT
 ┌─────────┐ ┌──────────────────────────┐ ┌──────────────────────────┐ ┌──────────────────────────┐
 │ LAW-Q2  │─▶│ one transition table,    │─▶│ SRP: state_machine.rs    │─▶│ transitions defined once,│
 │         │  │ not 4 mutation sites     │  │ owns guards; worker/SQL  │  │ tested exhaustively; SQL │
 │         │  │                          │  │ only *execute* events    │  │ CHECK mirrors the table  │
 ├─────────┤ ├──────────────────────────┤ ├──────────────────────────┤ ├──────────────────────────┤
 │ LAW-Q3  │─▶│ one budget port; gate,   │─▶│ DIP: pipeline depends on │─▶│ provider_slot ledger is  │
 │         │  │ extraction, embed,       │  │ ProviderBudget trait,    │  │ the only in-flight truth │
 │         │  │ vision share it          │  │ never on a semaphore     │  │ cluster-wide             │
 ├─────────┤ ├──────────────────────────┤ ├──────────────────────────┤ ├──────────────────────────┤
 │ LAW-Q1  │─▶│ one resolver replaces    │─▶│ OCP: new provider = new  │─▶│ ProviderProfile: one     │
 │         │  │ five clamp functions     │  │ profile row, zero code   │  │ struct, one resolve site │
 ├─────────┤ ├──────────────────────────┤ ├──────────────────────────┤ ├──────────────────────────┤
 │ LAW-Q5  │─▶│ one limiter (weights),   │─▶│ LSP: fairness classes    │─▶│ lane weights derived     │
 │         │  │ not caps + nested caps   │  │ substitutable under the  │  │ from B, never re-derived │
 │         │  │                          │  │ same conformance suite   │  │ per call site            │
 └─────────┘ └──────────────────────────┘ └──────────────────────────┘ └──────────────────────────┘
```

## What this deliberately does NOT change

- **The wake-channel design** (SPEC-057 P1): the channel stays a wake-up signal; workers re-authorize via `claim_next`. QW2 only stops it from being the *de-facto* backpressure.
- **The lease/heartbeat machinery** (`lease.rs`, `worker.rs:661-693`): the provider-slot ledger reuses the exact pattern rather than inventing a second leasing discipline.
- **The fairness park mechanism** (`worker.rs:1017-1094`): park-not-churn is kept; only the *numbers* change (weights instead of caps).
- **Deletion cascade phases** (`document_deletion.rs`): QW3 formalizes the cancel-intent coupling as a state-machine event; the cascade itself is untouched.
