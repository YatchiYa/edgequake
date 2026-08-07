# 04 — Fix plan (DRY / SOLID)

> **Status:** **Implemented** on HEAD (Waves A–E).  
> **Laws:** [01-first-principles.md](01-first-principles.md) · **Tests:** [05-e2e-test-matrix.md](05-e2e-test-matrix.md) · **Gates:** [measurements/e2e112-gates.txt](measurements/e2e112-gates.txt)

## Wave overview

```text
  Wave A (P0)  Identity + lifecycle
       │
       ▼
  Wave B (P0)  Shared-DB budget
       │
       ▼
  Wave C (P1)  Session safety nets
       │
       ▼
  Wave D (P1)  Observability / ops UX
       │
       ▼
  Wave E      Gates (ship with A–D; do not defer)
```

---

## Wave A — Identity + lifecycle (P0)

### A1 — Role-aware `application_name` (LAW-112-4, DRY)

**Where:** extend [`with_session_hygiene`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/connection.rs) to accept role label, **or** thin wrapper `with_session_hygiene_for_role(role)` used only from `pool_bundle::connect_role`.

**Do:**

```sql
SET application_name TO 'edgequake:query'   -- etc.
SET search_path TO public
```

in `after_connect` (single round-trip preferred: one `query` with both SETs).

**Do not:** invent a second hygiene path for `PostgresPool::initialize` without sharing the helper (DRY). Single-pool path may use `edgequake:default` or env `EDGEQUAKE_DB_POOL_ROLE`.

### A2 — Explicit `idle_timeout` + `max_lifetime` on bundle (LAW-112-7)

**Where:** `pool_bundle.rs` `connect_role`.

**Do:** set explicit durations (align with `PostgresConfig` defaults: idle 600s; max_lifetime sqlx-like 1800s unless env overrides). Document env if added (`EDGEQUAKE_DB_POOL_IDLE_TIMEOUT_SECS`, `EDGEQUAKE_DB_POOL_MAX_LIFETIME_SECS`).

### A3 — `PoolShutdown` after HTTP drain (LAW-112-5, SOLID-S/I)

**Where:** `PgPoolBundle` gains `pub async fn close(&self)` closing query/ingest/queue/admin; `server.rs` (or AppState drop path invoked from `run`) calls it after drain.

**DIP:** API depends on `close()` method on the bundle, not four duplicated `pool.close().await` sites.

**Edge:** SIGKILL still skips this — document in ops runbook.

---

## Wave B — Shared-DB budget (P0)

### B1 — Startup budget check (LAW-112-3)

```text
  need = total_max_connections() × instance_count
  limit = SHOW max_connections − reserve
  if need > limit → warn (default) or fail (EDGEQUAKE_DB_POOL_BUDGET_MODE=fail)
```

- `EDGEQUAKE_DB_POOL_INSTANCE_COUNT` default `1`
- Reserve: at least `superuser_reserved_connections` + documented headroom (e.g. 10) for tools/autovacuum

Log at boot: `query_max`, `ingest_max`, `queue_max`, `admin_max`, `total`, `instance_count`, `pg_max_connections`.

### B2 — Shared-DB sizing guidance (docs + optional lower defaults)

Ops runbook publishes recommended PPD sizes for co-tenant PG (example starting point — tune with partner):

| Role | Solo default | Shared-DB suggestion |
|------|-------------:|---------------------:|
| query | 16 | 8 |
| ingest | 12 | 6 |
| queue | 4 | 2 |
| admin | 2 | 1 |
| **sum** | **34** | **17** |

Do **not** change solo defaults without measurement; prefer env in PPD compose/helm.

### B3 — Reject “set max_connections=400” as product guidance (LAW-112-6)

Runbook: raise only after budget math; prefer PgBouncer for many services.

---

## Wave C — Session safety nets (P1)

### C1 — `idle_in_transaction_session_timeout`

Set in the same `after_connect` SSOT (e.g. `120s`). Protects against abandoned transactions holding snapshots (even though CSV had none).

### C2 — Optional `idle_session_timeout` (PG 14+)

Only if partner PG version supports it; gate by probe or document as server-side GUC.

### C3 — Preserve LAW-P4 release reset

Keep `RESET ALL` + `search_path` pin in `after_release`. Document EC-06 interaction with prepared statements ([06-edge-cases.md](06-edge-cases.md)).

---

## Wave D — Observability / ops UX (P1)

### D1 — Metrics: configured max per role

Extend `record_db_pool_stats_for_role` (or sibling) to include `max` from bundle fields already on `PgPoolBundle`.

### D2 — Health / ready payload

Expose per-role `{max, size, idle}` and optional `budget_ok` boolean. Operator action string when budget or util critical (reuse store-contention voice).

### D3 — Docs

Update `docs/operations/configuration.md` / performance-tuning with SPEC-112 formula and env table from [02-cross-ref-matrix.md](02-cross-ref-matrix.md).

---

## Wave E — Tests (mandatory with code)

See [05-e2e-test-matrix.md](05-e2e-test-matrix.md). No wave merges without its gates green.

---

## SOLID / DRY checklist for implementers

| Check | Pass criterion |
|-------|----------------|
| One hygiene SSOT | All production pools set name + search_path via one helper |
| One close API | `PgPoolBundle::close` only call site from server shutdown |
| One size resolve | Still `EDGEQUAKE_DB_POOL_SIZE_*` + clamp 1–128 |
| No second pool stack | No new global pool crate / no bypass of bundle in `state/postgres.rs` |
| Fail closed optional | Budget `fail` mode is explicit env, default `warn` for upgrade safety |

## Non-goals for first code train

- Mandatory PgBouncer in EdgeQuake Docker
- Changing partner QL pools
- Rewriting AGE session model for transaction-mode poolers
