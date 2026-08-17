# Lens 5 — System Engineer: Multi-Replica Correctness, Leases, Capacity, SLOs

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). Normative for runtime behaviour across processes: delivery modes, lease timing, drain bounds, capacity, telemetry, and runbooks. Storage shape is in Lens 3 and Lens 4.
> 

## Reading the deployment topology honestly

The branch already refuses the unsafe configuration: boot fails when `EDGEQUAKE_REPLICAS>1` while `EDGEQUAKE_TASK_DELIVERY=local`. That check is a good instinct, and it also documents the real limitation — correctness depends on which mode is selected, because two subsystems remain per process.

```
            ┌──────────────────┐        ┌──────────────────┐
            │    REPLICA A       │        │    REPLICA B       │
            │                    │        │                    │
cancel ────►│ CancellationRegistry│        │ CancellationRegistry│
            │   cancel_intents   │   ✘─────►   (never told)     │
            │ TenantLimiter      │        │ TenantLimiter      │
            │   semaphores: 2    │   ✘─────►   semaphores: 2    │
            │ ByteBudget 512MiB  │   ✘─────► ByteBudget 512MiB  │
            └────────┬─────────┘        └────────┬─────────┘
                     └──────────┐  ┌─────────┘
                                ▼  ▼
                          ┌─────────────────┐
                          │   POSTGRES      │  ◄ the only shared truth
                          │   claim + lease │
                          └─────────────────┘

Effective per-tenant concurrency = MAX_TASKS_PER_TENANT × replica count.
Effective memory ceiling         = 512 MiB × replica count.
Effective cancel reach           = one replica.
```

The consequence to state plainly: with three replicas, a cap configured as "two ingest tasks per tenant" actually admits six, and a cancel has a one-in-three chance of reaching the process doing the work. Both are accounting problems solved by moving the ledger into the database (Lens 4), not by tuning the semaphores.

## Timing the lease, the heartbeat, and the drain

```
t=0     claim                lease_expires_at = t + TTL (120 s default)
t=60    heartbeat            → lease_expires_at = t + 120
t=120   heartbeat            → …

worker dies at t=130
t=180   lease expires        another replica may claim; orphan recovery sweeps

CONSTRAINTS
  heartbeat_interval  <  TTL / 2          60 < 60 … currently exactly at the edge
  TTL                 >  p99 stage pause  a vision call must not outlive its lease
  drain_bound          = TTL              worst case wait for a dead owner

RECOMMENDATION
  TTL 120 s, heartbeat 30 s (currently 60 s), giving four chances to renew
  before expiry instead of two. MIN_TASK_LEASE_TTL_SECS = 30 stays as the floor.
```

Why this matters beyond hygiene: the deletion saga's drain step (Lens 3) waits for dependents to be terminal *or* for their leases to expire. The lease TTL is therefore the hard upper bound on how long a delete can block. Shortening the heartbeat does not shorten the bound, but it sharply reduces false expiries under load, which is what causes duplicate execution of the same ingest.

### Carrying cancellation on the heartbeat

```rust
// Today: refresh_lease(...) -> bool          (renewed? yes/no)
// Target:
enum LeaseVerdict {
    Renewed,            // keep going
    Lost,               // someone else owns it; abandon without side effects
    CancelRequested,    // durable intent observed; begin cooperative drain
}
```

This single signature change is the cheapest closure of hub gap G1. The heartbeat already runs, already talks to the authoritative row, and already returns a decision the worker respects. Notify is then a latency optimisation, not a dependency: with it, stop happens in seconds; without it, stop happens within one heartbeat.

## Bounding the cancellation path end to end

```
USER            API               POSTGRES            OWNER REPLICA
 │  cancel        │                   │                     │
 │─────────────►│ UPDATE intent     │                     │
 │                │────────────────►│                     │
 │ ◄ 202 cancelling                   │  NOTIFY             │
 │                │                   │───────────────────►│ token.cancel()
 │                │                   │                     │ ≤ 1 s
 │                │                   │                     │ abort at next await
 │                │                   │ ◄ state=cancelled   │ retract indexes
 │ ◄ event cancelled                  │                     │ release permits

BUDGET
  notify path   : ≤ 1 s signal + ≤ 4 s to reach an await point   → p99 ≤ 5 s
  heartbeat path: ≤ 30 s to next heartbeat + 4 s                 → p99 ≤ 35 s
  dead owner    : ≤ lease TTL 120 s                              → hard bound
```

The "≤ 4 s to reach an await point" term is not free — it is a requirement on the model pipeline, and Lens 7 specifies where the checks must sit to make it true.

## Sizing the system

```
Little's law:  L = λ × W

L = concurrent tasks needed
λ = arrival rate            e.g. 120 documents / hour = 0.033 /s
W = mean service time       e.g. 90 s for convert + ingest

L = 0.033 × 90 ≈ 3 concurrent tasks for steady state.
Provision for burst: L_peak = burst_factor × L, burst_factor 3–5 → 9–15 slots.

Slots come from two limits that must be consistent:
   WORKER_THREADS               per-replica execution slots
   MAX_TASKS_PER_TENANT         per-tenant ingest lane (local default 2)
   MAX_LIFECYCLE_TASKS_PER_TENANT  lifecycle lane (default 4)

Rule: sum of per-tenant caps for active tenants should exceed WORKER_THREADS,
or workers idle while work waits. Today, with the local defaults of 4 threads
and 2 ingest tasks per tenant, a single-tenant deployment can only ever use
half its capacity for ingestion — correct for a laptop, wrong for a cluster.
```

Byte admission is the second dimension. `DEFAULT_MAX_IN_FLIGHT_BYTES = 512 MiB` with `DEFAULT_TASK_BYTE_COST = 4 MiB` means roughly 128 unestimated tasks before refusal, but a handful of large documents can exhaust it. Because the budget is per process, the cluster ceiling is `512 MiB × replicas`; the container memory limit must be set against the per-replica figure, not the aggregate.

## Failing well

```
FAILURE                        DETECTION              RESPONSE
────────────────────────────────────────────────────────────────────
Worker process killed          lease expiry           reclaim; attempt marked
                                                      abandoned, task re-queued
Replica partitioned from DB    claim failures         stop claiming, drain, /ready 503
Provider timeout               typed timeout          retry with backoff; breaker
                                                      after 3 consecutive, unless
                                                      progress was made
DB pool saturation             db_pool_utilization    shed new work at 0.90,
                                                      warn at 0.75 (already wired)
Compensation failure           quarantine key         alert at 1, critical at 5
                                                      (already wired)
NOTIFY channel lost            none needed            periodic claim tick covers it
Handoff permit leaked          lane active_count > 0
                               with no running tasks  reaper releases after TTL
Clock skew between replicas    lease timestamps       use now() from the DB only,
                                                      never a local clock
```

The last row is a real hazard introduced by durable leases: any lease arithmetic performed with a process clock becomes wrong under skew. Every timestamp comparison in the claim and drain paths must be evaluated server-side in Postgres.

### Draining on shutdown

```
SIGTERM
  │
  ├─ stop accepting new claims               (shutdown.rs signal)
  ├─ /ready → 503 so the load balancer stops routing
  ├─ keep heartbeating running attempts      ◄ crucial: do not let leases lapse
  ├─ wait min(grace_period, longest running task)
  ├─ for anything still running: release_claim so it is re-queued cleanly
  │     rather than waiting for a 120 s lease expiry
  └─ exit

grace_period should exceed the p95 task duration, or every deploy manufactures
orphaned attempts that another replica must reclaim after the TTL.
```

## Instrumenting for truth rather than comfort

| Signal | Today | Change |
| --- | --- | --- |
| `cancel_intent_count` | per process, undercounts | derive from `COUNT(*) WHERE state='cancelling'` |
| `tenant_park_waiters{,_ingest,_lifecycle}` | per process | keep as a local gauge, add a database-derived `held` count |
| `pending_count` / `processing_count` | global | add per-tenant breakdown, needed for fairness |
| queue wait | `avg_wait_time_seconds`, `max_wait_time_seconds` | add p50/p95/p99 **per tenant**; averages hide starvation |
| — | missing | `cancel_to_stop_seconds` histogram, the SLO for INV-1 |
| — | missing | `fairness_error_ratio`: consumed share divided by entitled share |
| — | missing | `starved_tasks`: waiting longer than 20× the median |
| — | missing | `fence_rejected_writes_total`, proves INV-2 is active |
| `store_contention.*` | good | keep unchanged |

### Service level objectives

| Objective | Target | Error budget window |
| --- | --- | --- |
| Cancel honoured | p99 ≤ 5 s with notify, ≤ 35 s without | 30 days, 0.1 % |
| Delete completes | p99 ≤ 30 s for a single document | 30 days, 1 % |
| Queue wait for a tenant within quota | p95 ≤ 60 s | 7 days, 5 % |
| Duplicate execution | zero observed | any occurrence is an incident |
| Data resurrection after delete | zero observed | any occurrence is a severity-one incident |

## Proving it with chaos

| Experiment | Expected outcome | Invariant |
| --- | --- | --- |
| `kill -9` a worker mid-ingest | task re-queued after TTL, no duplicate vectors | INV-2 |
| Block NOTIFY between replicas | cancel still honoured within one heartbeat | INV-1 |
| Partition a replica from Postgres | replica stops claiming, reports not ready, no split-brain writes | INV-2 |
| Delete during an active convert | purge waits, then fence rejects the late writer | INV-3 |
| One tenant enqueues 10 000 tasks | quiet tenant starts within one claim cycle | INV-5 |
| Restart with `EDGEQUAKE_STARTUP_AUTO_RESUME=1` mid-flight | orphans reclaimed once, not twice | INV-4 |
| Clock skew of 5 minutes on one replica | no premature expiry, because time comes from the database | — |

## Running it day to day

```
SYMPTOM: users report cancel does nothing
  1. SELECT count(*) FROM tasks WHERE state='cancelling'
       AND cancel_requested_at < now() - interval '2 minutes';
  2. If non-zero: check the LISTEN connection health on every replica,
     then confirm heartbeats are advancing in attempts.
  3. If heartbeats are stale, the owner is gone: expect reclaim at the TTL.

SYMPTOM: one tenant appears starved
  1. Compare fairness_error_ratio per tenant.
  2. Check held counts: a large held population with idle workers means the
     quota is too low, not that the scheduler is unfair.
  3. Verify no leaked handoff permits: lane active_count without running attempts.

SYMPTOM: delete is slow
  1. EXPLAIN the dependent-task lookup; it must use tasks_document_active_idx.
  2. Check the drain wait: a dead owner costs up to one lease TTL by design.
```

## Where to read next

The claim and ledger statements are in Lens 4. Row semantics for `cancelling` and the drain barrier are in Lens 3. The Rust shapes for `LeaseVerdict` and the wake port are in Lens 6. Await-point placement that makes the cancel budget real is in Lens 7. The user-facing meaning of these budgets is in Lens 8, and the objectives roll up to the metrics table in Lens 1.