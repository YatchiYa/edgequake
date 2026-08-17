# 07 — Automatic Migration Engine & Upgrade Path

> The machine that executes Waves 1–3 (and later 5) data movement: efficient enough to finish inside an operational window, safe enough to run against live traffic, observable enough that **progress is a number, not a guess** (LD-08). This is the answer to "perfect migration with progressive user information."

## Three classes of change — never one execution context

| Class | Examples | Duration | Context | Boot behavior |
| --- | --- | --- | --- | --- |
| Schema change | create relation, add nullable column, `NOT VALID` constraint, empty index | ms–s | one TX, maintenance connection, bounded `lock_timeout` | **Blocking** — instance refuses to serve until applied |
| Data movement | chunk backfill, key-family moves, vector consolidation, constraint validation, CIC builds | min–days | resumable job, batched commits, maintenance pool, throttled | **Never blocking** — boot verifies, registers, resumes |
| Verification | coverage, checksums, recall comparison, residue checks | s–min | read-only job phase + on-demand | read-only probe, recorded not enforced |

Boot performs read-only schema verification and job resumption. It never applies a backfill, never builds an index, never runs DDL derived from a request. An instance with pending data movement **starts serving immediately**, reports the pending job, and lets the engine proceed at its admission budget.

## Migrations as data: the step descriptor

A script has no cursor, no estimate, no resume point — so each long-running migration is a **descriptor**: stable `step_id`, `schema_generation`, `step_sha384` digest over its statements (reusing the helper behind `edgequake_reconcile_state`, migration 102), keyset-cursor definition, idempotent batch statement, verification query, work-estimate source, reversibility class, admission profile. The engine is generic; descriptors are the migrations (DRY: W1 backfill, W2 family moves, W3 consolidation are all just descriptors).

```sql
CREATE TABLE edgequake_migration_job (
    job_id            uuid PRIMARY KEY DEFAULT uuidv7(),
    step_id           text NOT NULL,
    step_sha384       text NOT NULL,          -- descriptor changed => digest differs => no silent resume
    schema_generation integer NOT NULL,
    state             text NOT NULL,
    reversibility     text NOT NULL,
    cursor_position   jsonb,
    estimated_total   bigint,
    processed_count   bigint NOT NULL DEFAULT 0,
    failed_count      bigint NOT NULL DEFAULT 0,
    batch_size        integer NOT NULL,
    lease_owner       text,
    lease_expires_at  timestamptz,
    heartbeat_at      timestamptz,
    throttle_reason   text,
    started_at        timestamptz,
    completed_at      timestamptz,
    last_error        jsonb,
    UNIQUE (step_id, schema_generation),      -- idempotent creation under concurrent boot
    CHECK (state IN ('pending','preflight','running','paused','verifying','completed','failed','rolled_back')),
    CHECK (reversibility IN ('reversible','reversible_until_drop','irreversible'))
);

CREATE TABLE edgequake_migration_batch (
    job_id        uuid NOT NULL REFERENCES edgequake_migration_job(job_id) ON DELETE CASCADE,
    batch_seq     bigint NOT NULL,
    cursor_from   jsonb NOT NULL,
    cursor_to     jsonb NOT NULL,
    row_count     integer NOT NULL,
    duration_ms   integer NOT NULL,
    wal_bytes     bigint,
    committed_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (job_id, batch_seq)
);
CREATE INDEX ON edgequake_migration_batch (job_id, committed_at DESC);
```

## Job state machine (ASCII)

```ascii
                 ┌─────────┐  lease acquired   ┌───────────┐  all gates pass   ┌─────────┐
                 │ pending │──────────────────▶│ preflight │──────────────────▶│ running │◀──┐
                 └─────────┘                   └─────┬─────┘                   └────┬────┘   │
                                                    │ gate refused                   │       │
                                                    ▼                                │ throttle/operator
                 ┌─────────┐  operator resume   ┌───┴────┐                          ▼       │
                 │ failed  │◀───────────────────│        │                      ┌─────────┐  │
                 └────┬────┘  (cursor retained) │        │                      │ paused  │──┘
                      │                         └────────┘                      └─────────┘ condition
                      │ error budget exhausted       │ cursor exhausted                        cleared
                      ▼                              ▼
                 (retain cursor,                ┌───────────┐  verification satisfied  ┌───────────┐
                  record error)                 │ verifying │─────────────────────────▶│ completed │
                                                └─────┬─────┘                          └─────┬─────┘
                                                      │ refuted                              │ rollback invoked
                                                      ▼                                      ▼
                                                  ┌────────┐                           ┌────────────┐
                                                  │ failed │                           │rolled_back │
                                                  └────────┘                           └────────────┘
```

## Executing efficiently (LAW-D7, LAW-D8)

- **Keyset cursors only**, on an indexed ordered key (`(document_id, chunk_index)`). Offset pagination is prohibited — it turns a linear migration quadratic.
- **Idempotent batches**: `INSERT ... ON CONFLICT DO NOTHING`; batch ledger written **in the same transaction** as the work → a crash loses ≤1 batch, resume produces zero duplicate effect (EC-01).
- **Adaptive batch size**: target TX duration 500 ms–2 s; additive increase under target; immediate halving on duration/lock/error breach; hard range 50–5,000 rows.
- **Honest estimates**: `pg_class.reltuples` + periodic sampling; exact `COUNT(*)` once at preflight only below a declared size; estimate quality reported alongside the percentage.
- **Index strategy by size**: < ~1M rows → indexes maintained during load; ≥ ~1M → load into minimally indexed relation, then `CREATE INDEX CONCURRENTLY`, one build per database, reusing existing invalid-index detection + bounded `maintenance_work_mem`/`lock_timeout` (`vector/ddl.rs:32-160`).
- **Constraints** added `NOT VALID` in schema phase, validated as a job phase → short exclusive-lock windows. PG18 note: `NOT NULL` can later be promoted from a validated `CHECK` without a full table scan ([release-18](https://www.postgresql.org/docs/18/release-18.html)).
- **`ANALYZE`** on migrated relation + join partners before completion — every downstream latency comparison depends on fresh statistics.
- All job work on the **maintenance pool**; never on retrieval or request connections.

## Staying safe: preflight gates & runtime pause conditions

| Gate | Check | On failure |
| --- | --- | --- |
| Schema generation | applied generation = descriptor target | refuse; report version mismatch |
| Descriptor integrity | `step_sha384` matches job's recorded digest | refuse resume; require new job id |
| Extension floor | pgvector ≥ 0.8.2 (0.8.5 preferred); AGE 1.7+ for RLS/bulk (1.8.0-rc0 on PG18) | refuse dependent steps |
| Capacity headroom | free storage > estimated relation+TOAST+index+WAL growth × margin | refuse; report shortfall in bytes |
| Recovery point | verified backup/restore point id recorded on the job | refuse any step above `reversible` |
| Exclusivity | no other live lease for the step (lease + heartbeat + fencing token via `FOR UPDATE SKIP LOCKED`) | yield; report "running elsewhere" |
| Reversibility | descriptor classified ≤ `reversible_until_drop` | refuse automatic run; require recorded operator confirmation |
| Retrieval health | retrieval p95 within SLO | pause, `throttle_reason='retrieval_latency'`, resume on recovery |
| Replica lag | lag < ceiling | pause, `throttle_reason='replica_lag'` |
| Vacuum health | oldest-xact age + dead-tuple growth within limits | pause, `throttle_reason='vacuum_pressure'` |
| Error budget | consecutive batch failures < limit | fail job, retain cursor, record error payload |

Cross-cutting rules: **fail-closed** (unknown state/missing gate/unreadable ledger stops the job); **fencing token** prevents a stale lease holder from writing; **irreversible operations are excluded from automatic mode entirely** — surfaced as ready-to-run actions requiring operator confirmation (LD-07, execution rule 3); **pause is always safe** because the cursor lives in the ledger, not process memory.

## Progressive information: what the user sees, where, and when

Every job reports a fixed field set: `state`, `processed_count`, `estimated_total`, `estimate_quality`, `completion_pct` (monotonic, ledger-derived — identical across restarts), `rows_per_sec` (EWMA over recent batch window), `eta` (error < 20% through final decile), `cursor_position`, `elapsed`, `throttle_reason`, `consecutive_failures`, `last_error`.

```ascii
 ONE LEDGER (edgequake_migration_job + _batch) ──▶ FOUR SURFACES, SAME NUMBERS
 ┌──────────────────────────┐
 │ CLI: edgequake migrate   │  $ edgequake migrate status
 │ status                   │  STEP            STATE   %     ROWS/S   ETA     THROTTLE
 └──────────────────────────┘  w1-chunk-backfill running 42.3%  1,820    01:12:44  none
 ┌──────────────────────────┐  w2-family-wsdoc  pending  0.0%      —       —       —
 │ API: GET /admin/         │
 │ migration-jobs[/{id}]    │  JSON detail incl. cursor, estimate_quality, last_error
 └──────────────────────────┘
 ┌──────────────────────────┐  psql> SELECT * FROM edgequake.migration_progress;
 │ SQL view: edgequake.     │  (join of job + recent batch window; read-only)
 │ migration_progress       │
 └──────────────────────────┘
 ┌──────────────────────────┐  metrics: migration_completion_pct{step} gauge,
 │ Logs + metrics           │  migration_rows_processed_total counter,
 └──────────────────────────┘  migration_batch_duration_ms histogram,
                               migration_throttle{reason} gauge; 1 low-cardinality log line/batch
```

Alerting is defined on **stalls and throttles**, not duration — a slow migration politely yielding to retrieval traffic is the system working correctly. Operator control is exactly four verbs, safe at any point: **pause · resume · adjust admission budget · cancel** (cancel = stop after current batch, cursor retained — a permanent pause, not a revert).

**The fifth surface — derived guidance.** The four surfaces above report *raw* state (numbers the operator must still interpret). The intelligent CLI console (`edgequake migrate console`) is the **advisory** layer on top of the same ledger: it derives the cutover posture from the schema and emits *explicit* next instructions and *gated* guardrails — "backfill 42% → wait", "verify clean → flip `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational`", "drained + verified → safe to apply migration 125". It reuses this ledger and the migration-125 guard verbatim (never a parallel copy), and it refuses unsafe transitions (e.g. a `kv`/`dual` flag against a dropped store) instead of honoring them. See [15-migration-console-cli.md](15-migration-console-cli.md).

## Configuration

`EDGEQUAKE_MIGRATION_MODE = off | verify | automatic`
- `off` — no job work (schema changes still apply; descriptors register as pending).
- `verify` — read-only verification; reports pending jobs; changes nothing. **Default for the first release** carrying any new descriptor.
- `automatic` — executes reversible steps under the gates above.

Admission budget knobs (defaults published to the session like `EDGEQUAKE_MIGRATION_LARGE_GRAPH_THRESHOLD` today): target TX duration, batch-size range, retrieval-latency ceiling, replica-lag ceiling, concurrent index-build limit.

## Upgrade path: v0.22.0 → HEAD (KV retirement train)

> **Live path (2026):** Waves A–D shipped as **one unreleased HEAD train** (migrations **106–125**), not as separate v0.22.1 / v0.23 / v0.24 tags. Engine default remains `EDGEQUAKE_MIGRATION_MODE=verify`. Binding ops runbook: [docs/operations/spec091-upgrade-from-v0.22.0.md](../../docs/operations/spec091-upgrade-from-v0.22.0.md). Assessment: [16-post-cutover-assessment.md](16-post-cutover-assessment.md).

```ascii
 v0.22.0 (≤105, KV SSOT)     HEAD binary (write-stop)        schema apply
 ┌──────────────────┐ backup ┌──────────────────────┐        ┌────────────────────────────┐
 │ GHCR 0.22.0 DB   │───────▶│ roll ALL replicas    │───────▶│ migrate dry-run (no writes)│
 │ multi-tenant data│ restore│ relational flags on  │ review │ pending 106–125 + guard    │
 └──────────────────┘ point  └──────────────────────┘        └─────────────┬──────────────┘
                                                                           │ GREEN?
                                                                           ▼
                                                              migrate --confirm-drop
                                                              (106–124 expand+backfill
                                                               + 125 verified purge/drop)
                                                                           │
                                                                           ▼
                                                              HEAD API (boot migrate off)
                                                              soak: list / isolation / wipe
```

1. **Pre-upgrade** — verify pgvector ≥ 0.8.2 (pin 0.8.5+; CVE floor), AGE compatible with the image; take a restore point (`pg_dump -Fc` or volume snapshot). Rollback after 125 = **restore only**.
2. **Roll write-stop replicas** — every API process must treat KV `42P01` as source-gone **before/with** the drop (R-27). Keep `EDGEQUAKE_CHUNK_TEXT_AUTHORITY` + `EDGEQUAKE_KV_FAMILY_*=relational`. LD-15: boot never applies schema (exit-78 refusal on pending) — the CLI is the only schema writer ([17-boot-migration-gating.md](17-boot-migration-gating.md)).
3. **Preview** — `edgequake migrate dry-run` (exit 0 even when drop-readiness is RED). Optionally `migrate console` / `guard`.
4. **Apply** — `edgequake migrate` refuses 125 without confirmation; `edgequake migrate --confirm-drop` applies 106–125. Family SQL backfills (117–122) run inside the apply; engine `automatic` is only needed for large residual chunk-text backfill jobs on long-lived DBs.
5. **Verify** — HEAD API up; multi-tenant list/isolation/wipe/assets; optional `EDGEQUAKE_SERVING_FENCE=on` after query proof. Synthetic proof: `make spec091-upgrade-soak`.
6. **Later (not this train)** — W3 typed embeddings / W3 source-relation retirement remain separate irreversible programs; until then rollback for non-drop steps is flag/binary revert.

## Acceptance criteria (automatic mode)

| Property | Criterion |
| --- | --- |
| Boot independence | readiness time unchanged whether 0 or N data migrations pending |
| Bounded transactions | batch TX p95 < 2 s, p99 < 5 s across the full run |
| Crash resumption | ungraceful termination loses ≤ 1 batch; zero duplicated/skipped rows on resume |
| Exclusivity | 10 simultaneous instances ⇒ exactly 1 running job per step |
| Traffic protection | retrieval p95 degradation ≤ 10%; job pauses before SLO breach |
| Progress honesty | `completion_pct` monotonic non-decreasing; ETA error < 20% through final decile |
| Verification coverage | every completed job carries a recorded verification result; completion impossible without one |
| Irreversibility control | zero irreversible operations without recorded operator confirmation |
| Replica safety | lag under ceiling for the entire run at the largest ladder rung |
| Scale | chunk backfill completes at 1M and 10M rows with all criteria holding |
