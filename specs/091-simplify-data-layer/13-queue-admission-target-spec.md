# 13 — Queue & Admission: Target Specification

> Contracts for the queue/admission redesign: the task state machine (code SSOT), the provider-slot ledger (DDL), the admission resolver (capacity SSOT), and the API surface changes. Each traces to a law in [12](12-queue-admission-first-principles.md) and a decision ([LD-11..13](README.md#locked-decisions)). Waves in [14](14-queue-admission-plan.md).

## Task state machine (LAW-Q2 — code SSOT)

### States (persisted vocabulary — `tasks.status`, unchanged strings)

| State | Meaning | Terminal? |
| --- | --- | --- |
| `pending` | Admitted; waiting for claim (covers the user's `UPLOADED`+`QUEUED`: admission and enqueue are one atomic step — see § Admission) | no |
| `processing` | Claimed under a live lease by a worker | no |
| `indexed` | Completed successfully (success terminal) | yes |
| `failed` | Failed permanently (max retries, non-retryable class, timeout breaker) — reprocess is an explicit *new intent*, not a state the machine re-enters silently | yes (reversible by operator event) |
| `cancelled` | Cancel intent honored at a stage boundary | yes |

`uploaded`/`queued` from the UX sketch map to **one** persisted state (`pending`): admission and enqueue happen in the same request/TX, so no observable intermediate state exists. The *queued-vs-working* distinction the user sees is a **projection** (`queue_position` > 0 ⇒ queued, = 0 with lease ⇒ working) — LAW-D4: a count/position is a projection of the state machine, never a separate state that can drift.

### Events (`TaskEvent` — the only legal mutation verbs)

```rust
pub enum TaskEvent {
    Enqueue,          // (none) -> pending            — admission accepted
    Claim,            // pending -> processing; stale-processing -> processing (SQL-guarded)
    Complete,         // processing -> indexed
    Fail,             // pending|processing|failed -> failed — per-attempt failure record
    RetryRequeue,     // failed -> pending            — automatic retry (error fields retained)
    Reprocess,        // failed -> pending            — operator intent only
    Cancel,           // pending|processing|failed -> cancelled
    LeaseLost,        // processing -> pending        — stale reclaim / boot orphan auto-resume
    Release,          // processing -> pending        — voluntary (fairness park, byte budget)
}
```

`failed` is "failed, disposition pending": retry bookkeeping (`retry_count`, circuit breaker) lives on the row, and the retry/terminal decision is taken by the worker after classification. The machine keeps `Failed → Pending` explicit (two distinct intents: `RetryRequeue` vs `Reprocess`) so metrics can tell them apart.

### Transition table (exhaustive — anything not listed is a `TransitionError`)

```ascii
 FROM        │Enqueue│ Claim │Complete│ Fail │RetryRequeue│Reprocess│ Cancel │LeaseLost│Release
 ────────────┼───────┼───────┼────────┼──────┼────────────┼─────────┼────────┼─────────┼───────
 (none)      │pending│   ✗   │   ✗    │  ✗   │     ✗      │    ✗    │   ✗    │    ✗    │   ✗
 pending     │   ✗   │ proc. │   ✗    │failed│     ✗      │    ✗    │ cancel │    ✗    │   ✗
 processing  │   ✗   │ proc.*│indexed │failed│     ✗      │    ✗    │ cancel │ pending │pending
 indexed     │   ✗   │   ✗   │   ✗    │  ✗   │     ✗      │    ✗    │   ✗    │    ✗    │   ✗
 failed      │   ✗   │   ✗   │   ✗    │failed│  pending   │ pending │ cancel │    ✗    │   ✗
 cancelled   │   ✗   │   ✗   │   ✗    │  ✗   │     ✗      │    ✗    │   ✗    │    ✗    │   ✗

 * Claim on `processing` is legal ONLY when lease_expires_at < now() (stale reclaim arm,
   postgres.rs claim arms) — the SQL guard carries the staleness proof, callers cannot skip it.
   `Fail` from `pending`/`failed` covers fail-before-start and per-attempt re-failure
   bookkeeping; `Cancel` from `failed` covers cancelling a failed task's pending retry
   (apply_task_row_cancel cancels any non-Indexed row).
```

### Module contract — `edgequake/crates/edgequake-tasks/src/state_machine.rs` (QW0)

- `transition(from: TaskStatus, event: TaskEvent) -> Result<TaskStatus, TransitionError>` — pure function, the one table above.
- `guard_sql(event: TaskEvent) -> &'static str` — the SQL `WHERE` fragment that *enforces the same guard in the database* (e.g. `Claim` ⇒ `status='pending' OR (status='processing' AND lease_expires_at < now())`). DRY: the Rust table and the SQL guard are generated from one definition; a unit test asserts they cannot drift.
- All current mutation sites route through it: `Task::mark_*` (`types/task.rs:191-302`), `claim_next`/`release_claim` (`postgres.rs:653-853`), retry (`worker.rs:775-829`), boot recovery (`orphan_task_recovery.rs:155-217`).
- **Zero behavior change in QW0** — the table formalizes exactly what the code does today, including the ambiguous `Failed` (kept terminal; `Reprocess` is the only exit).

## Provider-slot ledger (LAW-Q3, LD-11 — migration 110)

```sql
-- edgequake/migrations/110_spec091_provider_budget.sql
CREATE SCHEMA IF NOT EXISTS edgequake;

CREATE TABLE edgequake.provider_slot (
    provider_key     text        NOT NULL,             -- 'ollama:gemma3', 'openai:gpt-5-nano', ...
    slot_id          integer     NOT NULL,             -- 0..budget-1, seeded per budget change
    lease_owner      text,                             -- worker instance id
    lease_token      uuid,                             -- fencing token
    lease_expires_at timestamptz,
    task_track_id    text,                             -- attribution (observability)
    workspace_id     uuid,
    acquired_at      timestamptz,
    PRIMARY KEY (provider_key, slot_id)
);
-- Stale-slot reclaim: same discipline as task leases
CREATE INDEX idx_provider_slot_stale
    ON edgequake.provider_slot (provider_key, lease_expires_at)
    WHERE lease_owner IS NOT NULL;

CREATE TABLE edgequake.provider_budget (                  -- LAW-Q1 SSOT, seeded by resolver
    provider_key   text PRIMARY KEY,
    budget         integer NOT NULL CHECK (budget BETWEEN 0 AND 64),
    source         text NOT NULL,                         -- 'measured' | 'env' | 'profile'
    updated_at     timestamptz NOT NULL DEFAULT now()
);

CREATE VIEW edgequake.provider_inflight AS
SELECT provider_key, count(*) FILTER (WHERE lease_owner IS NOT NULL) AS inflight,
       (SELECT budget FROM edgequake.provider_budget b WHERE b.provider_key = s.provider_key) AS budget
FROM edgequake.provider_slot s GROUP BY provider_key;
```

Acquisition (`try_acquire_slot`): one statement — `UPDATE … FROM (SELECT … WHERE lease_owner IS NULL OR lease_expires_at < now() ORDER BY slot_id FOR UPDATE SKIP LOCKED LIMIT 1) … RETURNING slot_id, lease_token`. Release is CAS on `(lease_owner, lease_token)`. A reaper (same cadence as the task-lease discipline) expires stale slots; chaos-proven (EC-22). **`budget = 0` short-circuits the port** (cloud-only deployments pay no round trip; LD-11 does not force DB traffic where no local provider exists).

### Port — `edgequake/crates/edgequake-tasks/src/provider_budget.rs` (QW1)

```rust
#[async_trait]
pub trait ProviderBudget: Send + Sync {
    async fn try_acquire(&self, provider: &str, task: &str, ws: Option<Uuid>)
        -> Result<Option<ProviderSlotLease>, StorageError>;   // None = saturated (caller parks)
    async fn release(&self, lease: ProviderSlotLease) -> Result<(), StorageError>;  // CAS
    async fn reap_expired(&self) -> Result<u64, StorageError>;
}
```

`ProviderSlotLease` is RAII + heartbeat (60 s, mirroring `worker.rs:661-693`); drop without release ⇒ TTL expiry (default 120 s, `EDGEQUAKE_TASK_LEASE_TTL_SECS` reused — one TTL law for all leases). The existing `local_inference_gate.rs` acquire path is re-pointed at this port; its env (`EDGEQUAKE_LOCAL_MAX_INFLIGHT`) becomes a *budget override* feeding `provider_budget.source='env'`, not a semaphore size.

## Admission resolver (LAW-Q1 — capacity SSOT, QW2)

`edgequake/crates/edgequake-pipeline/src/pipeline/admission_resolver.rs`:

```rust
pub struct ProviderProfile {           // one per configured provider
    pub provider_key: String,          // 'ollama' | 'lmstudio' | 'openai' | 'mock' | ...
    pub budget: u16,                   // B — the ONLY hand-set number (env/profile/measured)
}
pub struct AdmissionPlan {             // EVERYTHING below is f(B) — no second opinion
    pub worker_threads: u16,           // clamp(2B, 4, 32)
    pub extraction_concurrency: u16,   // clamp(B, 1, 8)
    pub embed_max_async: u16,          // clamp(B/2, 1, 8)
    pub merge_max_async: u16,          // clamp(B/2, 1, 8)
    pub vision_jobs: u16,              // clamp(B/2, 1, 4)
    pub queue_soft_bound: u32,         // ceil(λ̂ × target_wait) — measured drain rate
    pub tenant_lane_weight: u32,       // 1 (equal DRR shares, LAW-Q5)
}
pub fn resolve(profile: &ProviderProfile, drain_rate_per_min: f64) -> AdmissionPlan;
```

The five existing resolvers (`pipeline/config.rs:177-306`, `local_inference_gate.rs:15-18`, `core/resource/budget.rs:73,149`, `tasks/admission.rs:19-25`) become **readers of the plan**, keeping their env vars as overrides that feed `ProviderProfile.budget` (backward compatible, one release of deprecation warnings). `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY` remains the explicit opt-out, recorded in `provider_budget.source`.

## API surface changes (LAW-Q4, LD-12 — QW2)

Upload responses (text insert, multipart, PDF — all three paths converge on `admit_document_for_processing` / `create_pdf_processing_task`):

```json
{ "status": "queued", "track_id": "pdf-…", "document_id": "…",
  "queue_position": 7, "eta_seconds": 252, "eta_basis": "ewma_drain_rate_10m" }
```

- `queue_position` = count of older pending tasks in the same workspace-share order (projection, LAW-D4 — not a stored field).
- `eta_seconds` = `queue_position / λ̂ × 60`, λ̂ = EWMA of task completions (τ = 10 min, seeded from the first 10 completions after boot; clamped `[0, 4h]`, labeled `eta_basis` so the UI can be honest about uncertainty — R-15).
- HTTP status stays **202** for all admitted uploads (no 429 for valid uploads — LD-12); the wake channel is consumed by a dispatcher task with a timeout, never by the HTTP handler (fixes the silent-hang at `queue.rs:84-94`).
- `QueueMetrics.rate_limited` is deleted (it was hardcoded `false`) and replaced by real fields: `queue_position_p95`, `drain_rate_per_min`, `admission_soft_beyond` gauge. Pressure labels (`task_queue_pressure.rs`) stay observational.
- `GET /api/v1/tasks/{track_id}` gains `queue_position`/`eta_seconds` while `pending` (same projection, one helper — DRY).

## Fair-share lanes (LAW-Q5 — QW3)

`TenantConcurrencyLimiter` keeps its dual classes (`Ingest`, `Lifecycle`) and the park machinery; the **per-tenant cap becomes a weight**. Effective share for tenant *i* among *n* active ingest tenants = `⌊B × wᵢ / Σw⌋` with minimum 1 when the tenant has claimed work. Single active tenant ⇒ full `B` (regression guard, R-16). The nested per-workspace cap of 1 is retained as an *ordering* device within a tenant (it prevents one workspace from starving its siblings), not as a capacity number.

## Env / flag inventory (new — all others unchanged)

| Variable | Default | Meaning |
| --- | --- | --- |
| `EDGEQUAKE_PROVIDER_BUDGET` | unset ⇒ profile default (local 2 / cloud 32) | cluster-wide provider in-flight budget B (seeds `provider_budget`) |
| `EDGEQUAKE_QUEUE_TARGET_WAIT_SECS` | 600 | target wait used for the queue soft bound |
| `EDGEQUAKE_ADMISSION_ETA_CLAMP_SECS` | `0,14400` | ETA clamp range |
| `EDGEQUAKE_FAIR_SHARE` | `on` | `on` = weighted DRR lanes; `off` = legacy caps (one-release escape hatch, R-16) |
