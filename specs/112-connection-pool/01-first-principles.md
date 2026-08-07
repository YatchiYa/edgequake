# 01 — First principles (LAW-112)

> **Cross-refs:** [WHY](00-why.md) · [Incident data](00-incident-data.md) · [SPEC-090 laws](../090-performance/00-first-principles.md) · [sqlx PoolOptions](https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html)

## Laws

### LAW-112-1 — Code is law

GitHub / Slack prose is a hypothesis. The SSOT is:

- `edgequake-storage/src/adapters/postgres/pool_bundle.rs` — `PgPoolBundle`
- `edgequake-storage/src/adapters/postgres/connection.rs` — `with_session_hygiene`, `PostgresPool`
- `edgequake-api/src/state/postgres.rs` — production boot wires the bundle
- `edgequake-api/src/server.rs` — graceful HTTP drain (SPEC-083)

If narrative and code disagree, **code wins** until patched.

### LAW-112-2 — Idle is held capacity

```text
  sqlx idle queue ──TCP──► PostgreSQL backend (process)
                              │
                              ▼
                     consumes 1 × max_connections slot
                     + backend memory / scheduler time
```

An `idle` / `ClientRead` backend is not “free.” On a shared database it starves co-tenants exactly as an `active` backend does for **slot accounting**.

### LAW-112-3 — Fleet slot budget

```text
  Σ (instances × per_process_pool_max)
  + migrate/CLI/worker pools
  + admin tools (DBeaver, psql)
  + autovacuum / replication / reserved
  ≤  max_connections − reserve
```

Default EdgeQuake serving process: `per_process_pool_max = 34` (16+12+4+2) unless env overrides.

Corollary: **N replicas × 34** must fit the shared budget. Rolling deploy ≈ temporary **2N**.

### LAW-112-4 — Identity on every backend

Every EdgeQuake backend **must** set:

```text
application_name = edgequake:<role>
  role ∈ {query, ingest, queue, admin}
```

Without this, `pg_stat_activity` cannot attribute load (PPD CSV: all empty). Identity is an ops feature, not cosmetics.

### LAW-112-5 — Shutdown closes pools

SPEC-083 drains HTTP. SPEC-112 extends the lifecycle:

```text
  SIGTERM → cancel accept → drain in-flight
         → PgPoolBundle.close() all roles
         → process exit
```

Without `pool.close()`, backends rely on TCP death detection — slower and friendlier to “stop the service to free slots” as a recovery hack.

### LAW-112-6 — Do not treat max_connections inflation as the product fix

Raising PostgreSQL `max_connections` (e.g. to 400) increases process/memory overhead and papers over oversubscription. Product fix order:

1. Attribute (`application_name`)
2. Budget + size for shared DB
3. Reap (`idle_timeout` / `max_lifetime`)
4. Close on shutdown
5. Only then consider server ceiling / PgBouncer

### LAW-112-7 — One connect SSOT for protective GUCs

Session safety nets belong in the same helper that already pins `search_path` (extend `with_session_hygiene` or a role-aware wrapper):

| Knob | Purpose |
|------|---------|
| `application_name` | Attribution (LAW-112-4) |
| `idle_in_transaction_session_timeout` | Kill abandoned txns holding snapshots |
| sqlx `idle_timeout` / `max_lifetime` | Reap idle / aged pooled conns |
| LAW-P4 `RESET ALL` + `search_path` on release | No GUC leak across checkouts |

Do not scatter SET strings across adapters.

### LAW-112-8 — Observability is part of the pool contract

Configured max, live size, and idle per role must be visible in metrics/health (extend existing gauges in `handlers/metrics.rs` / store-contention readiness). Operators cannot manage what they cannot see.

---

## SOLID / DRY application

| Principle | Application |
|-----------|-------------|
| **S** | Connect hygiene owns session identity/GUCs; shutdown helper owns close; sizing helper owns budget math |
| **O** | New roles extend `PoolRole` + env keys; do not fork a fifth ad-hoc `PgPoolOptions::new()` in API |
| **L** | `for_role` / `primary()` remain interchangeable for callers already on the bundle |
| **I** | Narrow `PoolShutdown` / budget check APIs — do not force migrate CLI through full AppState |
| **D** | Handlers depend on pool stats traits/helpers, not raw sqlx internals duplicated |
| **DRY** | One `with_session_hygiene` (role-aware); one close-all; one size resolve (`EDGEQUAKE_DB_POOL_SIZE_*`) |

## Relationship to SPEC-090 LAW-P4

LAW-P4: DDL/maintenance GUCs never leak onto pooled connections.  
LAW-112 adds: **fleet-level** connection identity, lifetime, and shared-DB budget on top of that hygiene — same connect/release hooks, broader contract.
