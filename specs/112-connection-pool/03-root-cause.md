# 03 — Root cause (code is law)

> Partner narrative + CSV + HEAD. Hypotheses ranked by evidence.  
> Honesty: [`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md).

## Symptom chain (observed)

```text
  Shared PG approaching connection ceiling
            │
            ▼
  EdgeQuake process holds many idle backends (pool)
            │
            ▼
  QL / other services get connection refused / cannot reconnect
            │
            ▼
  Stop EdgeQuake → TCP/backends released → QL reconnects
            │
            ▼
  Ops raises max_connections → 400  (band-aid)
```

## H1 — Shared-DB pool oversubscription (PRIMARY)

### What the code does

Production boot (`state/postgres.rs`):

```rust
let pool_bundle = edgequake_storage::PgPoolBundle::connect(&database_url).await?;
```

`PgPoolBundle::connect` opens **four** pools. Defaults (`pool_bundle.rs`):

| Role | Default max | min |
|------|------------:|----:|
| query | 16 | 1 |
| ingest | 12 | 1 |
| queue | 4 | 1 |
| admin | 2 | 1 |
| **sum** | **34** | **4 always warm** |

`connect_role` always sets `min_connections(1)` → at least **4** backends even under zero traffic. Under load, up to **34** per process.

There is **no** startup check that `total_max_connections() × instances ≤ SHOW max_connections − reserve`.

### CSV support

- 10 idle EdgeQuake backends from one `client_addr` — consistent with a partially warm / mid-load pool, not with “no pooling.”
- Co-tenants (QL, DBeaver, monitor) share the same instance.

### Verdict

**CONFIRMED as design risk on shared PG.** Whether PPD was at the hard ceiling at peak is **not** proven by this CSV (only 23 clients). Partner recovery-by-stop strongly supports slot contention.

---

## H2 — Missing `application_name` (CONFIRMED gap)

### What the code does

`with_session_hygiene` (`connection.rs`):

```rust
.after_connect(|conn, _| {
    Box::pin(async move {
        sqlx::query("SET search_path TO public").execute(conn).await.map(|_| ())
    })
})
```

No `SET application_name`. Bundle `connect_role` reuses this helper without role labeling.

### CSV support

All 10 EdgeQuake rows: `application_name=""`, last query `SET search_path TO public`.

### Verdict

**CONFIRMED product gap.** Blocks ops attribution and role-level incident response (LAW-112-4).

---

## H3 — No `pool.close()` on graceful shutdown (CONFIRMED gap)

### What the code does

`server.rs` `run`: SIGTERM → CancellationToken → axum `with_graceful_shutdown` + drain budget (SPEC-083 X-31).

Grep of API `src/`: **no** `pool.close()` / bundle close on that path. Connections live until process teardown / TCP timeout.

### CSV support

Indirect: stopping the service freed slots (partner). Hard kill works eventually; graceful close would free slots immediately and predictably.

### Verdict

**CONFIRMED lifecycle gap** (LAW-112-5). Not the sole cause of idle-while-running contention, but worsens deploys and “zombie after stop” windows.

---

## H4 — Bundle lacks explicit idle / max lifetime (CONFIRMED soft gap)

### What the code does

`PostgresPool::initialize` sets `idle_timeout(Some(config.idle_timeout))` (default 600s) but **not** `max_lifetime`.

`PgPoolBundle::connect_role` sets only `max_connections`, `min_connections(1)`, `acquire_timeout` — **no** explicit `idle_timeout` / `max_lifetime` → sqlx defaults (idle ~10m, max_lifetime ~30m).

### Verdict

**CONFIRMED inconsistency.** Not a smoking gun for the incident, but violates LAW-112-7 (one SSOT) and can leave aged backends around longer than operators expect.

---

## H5 — Classic connection leak (`idle in transaction`) (NOT SUPPORTED by CSV)

CSV: **0** rows with `state = 'idle in transaction'`. All EdgeQuake rows are plain `idle`.

### Verdict

**Not evidenced** in the attached snapshot. Still mitigate with `idle_in_transaction_session_timeout` (Wave C) as a safety net.

---

## H6 — “Need max_connections=400” (REJECTED as product conclusion)

Raising the ceiling may have been a necessary **ops emergency**. It is not evidence that EdgeQuake correctly budgets slots. Product response is LAW-112-6: fix budget/identity/reap/close first.

---

## ASCII — defect surfaces on HEAD

```text
                    ┌─────────────────────────────┐
   HTTP SIGTERM ───►│ server.rs graceful drain    │
                    │   ✗ no pool.close()         │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │ PgPoolBundle (≤34)          │
                    │  query │ ingest │ queue │ admin │
                    │  min=1 each (4 warm)        │
                    │  ✗ no app_name              │
                    │  ✗ no explicit idle/lifetime│
                    │  ✗ no fleet budget check    │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │ PostgreSQL shared instance  │
                    │  EdgeQuake + QL + tools     │
                    │  max_connections (ops=400)  │
                    └─────────────────────────────┘
```

## Summary table

| ID | Hypothesis | Status |
|----|------------|--------|
| H1 | Shared-DB oversubscription via large pools | PRIMARY (design) |
| H2 | Empty `application_name` | CONFIRMED |
| H3 | Missing shutdown `pool.close()` | CONFIRMED |
| H4 | Missing explicit idle/max lifetime on bundle | CONFIRMED |
| H5 | Idle-in-transaction leak | NOT in CSV |
| H6 | 400 max_connections is the fix | REJECTED |
