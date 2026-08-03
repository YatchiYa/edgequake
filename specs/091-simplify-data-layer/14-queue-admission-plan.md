# 14 — Queue & Admission: Implementation Plan (Waves QW0–QW3)

> Same discipline as [06](06-implementation-plan.md): ordered by dependency and reversibility; every wave states **entry → mechanism → exit gate → rollback**; an exit gate failing twice returns the wave to its previous state. These waves are independent of W1–W5 (different subsystem) but inherit the same execution rules (flags, dual evidence, ≤1 irreversible op per release). Proposed release mapping: QW0+QW1 → v0.23.x alongside W1 · QW2 → v0.24.x · QW3 → v0.25.x.

## Sequencing invariants (queue-specific)

1. **The state machine precedes any new transition** — you cannot add events to a machine that does not exist (LAW-Q2).
2. **The budget ledger precedes the resolver** — the resolver derives *from* a budget that must have a cluster-visible home first (LAW-Q3 before LAW-Q1 enforcement).
3. **Admission labels precede fair-share** — users must see honest queue state before the sharing policy changes underneath them (LAW-Q4 before LAW-Q5).
4. **No behavioral change without a conformance test landing in the same wave** (ECs are acceptance criteria).

```ascii
 ┌─────────┐   ┌──────────────┐   ┌──────────────────┐   ┌─────────────────┐
 │ QW0     │──▶│ QW1          │──▶│ QW2              │──▶│ QW3             │
 │ state   │   │ provider-slot│   │ admission        │   │ fair-share +    │
 │ machine │   │ ledger + port│   │ resolver + queued│   │ lifecycle proof │
 │ (pure   │   │ (gate re-    │   │ state + ETA      │   │ + e2e/chaos     │
 │ refactor│   │  pointed)    │   │                  │   │                 │
 └─────────┘   └──────────────┘   └──────────────────┘   └─────────────────┘
```

## QW0 — State machine SSOT (pure refactor, zero behavior change)

**Entry:** none.
**Mechanism:**
1. Add `edgequake-tasks/src/state_machine.rs` per [13 § state machine](13-queue-admission-target-spec.md): `TaskEvent`, `transition()`, `guard_sql()`, `TransitionError`.
2. Route all mutation sites through it: `types/task.rs:191-302` (`mark_success/mark_failed/mark_failed_with_details/mark_cancelled`), `postgres.rs:653-853` (`claim_next`/`release_claim` SQL guards rewritten to use `guard_sql`), `worker.rs:775-829` (retry path uses `FailRetryable` instead of field mutation), `edgequake-api/src/services/orphan_task_recovery.rs:155-217` (boot recovery uses `LeaseLost`/`FailPermanent`).
3. `contract_spec091_state_machine_transitions`: exhaustive valid/invalid matrix — every (state × event) cell asserted; plus a drift test asserting each SQL guard matches the Rust table.
**Flag:** none (pure refactor; behavior identical by construction + by the existing test suite staying green).
**Exit gate:** contract suite green; full `edgequake-tasks` + `edgequake-api` test suites green with zero changes to their assertions; `grep` proves no remaining `SET status` / `status =` outside `state_machine.rs` + the two SQL claim/release sites that embed `guard_sql`.
**Rollback:** revert — no data or flag state involved. Free.

## QW1 — Provider-slot ledger + budget port

**Entry:** QW0 exit.
**Mechanism:**
1. Migration `110_spec091_provider_budget.sql` ([13 § ledger](13-queue-admission-target-spec.md)) + checksum-lock append; slot seeding function keyed by `provider_budget`.
2. `edgequake-tasks/src/provider_budget.rs`: `ProviderBudget` port + `PostgresProviderBudget` adapter (`try_acquire`/`release`/`reap_expired`) + `NoopProviderBudget` (budget 0 / tests). RAII lease with heartbeat mirroring the task-lease discipline.
3. Re-point `edgequake-api/src/local_inference_gate.rs:81-120` to acquire through the port (keep the module name/API so `safety_limits.rs` callers are untouched — DIP at the boundary that already exists); `EDGEQUAKE_LOCAL_MAX_INFLIGHT` becomes a budget override.
4. Reaper task spawned at boot (admin pool, beside the migration engine wiring in `state/postgres.rs`).
**Flag:** `EDGEQUAKE_PROVIDER_BUDGET=0` disables the ledger (cloud-only); non-zero enables. Default preserves today's effective behavior (local 2).
**Exit gate:** `contract_spec091_provider_budget_*` green (acquire/release/CAS/reap, concurrent claimants); `chaos_spec091_queue_worker_crash_lease_reclaim` proves a killed worker's slot is reclaimed within TTL+ε; saturation test proves cluster-visible inflight never exceeds B under 2 simulated instances.
**Rollback:** set budget 0 / revert the gate re-point; the ledger tables are inert. Free.

## QW2 — Admission resolver + explicit queued state

**Entry:** QW1 exit (budget has a home the resolver can read).
**Mechanism:**
1. `edgequake-pipeline/src/pipeline/admission_resolver.rs` per [13 § resolver](13-queue-admission-target-spec.md); the five legacy resolvers read the plan (env vars become overrides of `ProviderProfile.budget` with deprecation warnings).
2. Enqueue admission: dispatcher task consumes the wake channel with timeout (HTTP handler never blocks on `queue.rs:84-94`); `admit_document_for_processing` + PDF enqueue compute `queue_position`/ETA projection and return it in the 202 body; `GET /api/v1/tasks/{track_id}` exposes the same projection while pending (one shared helper).
3. Metrics: `QueueMetrics.rate_limited` removed; real gauges (`edgequake_provider_inflight`, `edgequake_queue_position_p95`, `edgequake_queue_drain_rate_per_min`, `edgequake_task_transitions_total{event}`) in `edgequake-observability/src/metrics.rs`.
**Flag:** `EDGEQUAKE_QUEUE_TARGET_WAIT_SECS` (bound derivation); ETA always on (additive response fields — additive JSON is backward compatible).
**Exit gate:** `e2e_spec091_queue_explicit_queued_state` green (saturate budget → upload returns 202 + position + ETA → drains in order); p95 enqueue latency unchanged vs pre-QW2 baseline; zero clippy/test regressions.
**Rollback:** revert the response fields + dispatcher (additive, no data migration). Free.

## QW3 — Fair-share lanes + lifecycle precedence + edge-case proof

**Entry:** QW2 exit.
**Mechanism:**
1. `tenant_limiter.rs`: caps → weights (DRR over active tenants, [13 § fair-share](13-queue-admission-target-spec.md)); park machinery untouched; `EDGEQUAKE_FAIR_SHARE=off` escape hatch.
2. Lifecycle precedence formalized: every stage boundary (claim, retry requeue, fairness park, pipeline stage) evaluates cancel/delete intents through the state machine; delete ⇒ cancel-intent coupling asserted as a machine-level rule (LAW-Q7).
3. Land the full e2e/chaos edge-case suite ([11 § queue](11-e2e-test-matrix.md)): EC-17..EC-24.
**Flag:** `EDGEQUAKE_FAIR_SHARE = on | off`.
**Exit gate:** all EC-17..24 tests green; `e2e_spec091_queue_provider_budget_never_exceeded` proves inflight ≤ B ∧ no starvation across 2 tenants; single-tenant throughput within 5% of pre-QW3 measurement (R-16 guard).
**Rollback:** `EDGEQUAKE_FAIR_SHARE=off`. Free.

## Execution rules (inherited + queue-specific)

1. Every behavioral change behind a flag with an escape hatch (LD-07 analog).
2. The state machine is the only place a transition is *defined*; tests are the only place a transition is *asserted*; SQL is the only place a transition is *enforced under concurrency* — three surfaces, one table (LAW-Q2).
3. Additive API fields only; no response field is removed in these waves (`rate_limited` removal ships with a deprecation note, not a breaking release).
4. An exit gate failing twice returns the wave to its previous state.

## Definition of Done (queue & admission)

Status aligned with [11 § Exists today](11-e2e-test-matrix.md#exists-today-run-these) (not the aspirational row names above).

- [x] State-machine / claim-guard discipline in tree; lib + queue suites exercise transitions (named `contract_spec091_state_machine_*` binaries remain aspirational — behavior covered via tasks contracts + e2e)
- [x] Provider budget enforced cluster-wide (`contract_spec091_provider_budget`); crash-reclaim covered in `e2e_spec091_queue_chaos`
- [x] Explicit queued admission path in tree (`e2e_spec091_queue_admission`)
- [x] Tenant fair-share / park marker (`contract_spec091_fairness_park_marker` + queue e2e)
- [x] EC-17..EC-24 mechanisms landed; CI runs the **existing** queue admission/chaos binaries (not every planned suite name in the tables above)
- [ ] Full aspirational matrix row-for-row (`e2e_spec091_queue_eta_honest`, stage-parameterized delete suites, …) — still open where no binary exists
