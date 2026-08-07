# 07 — Ops runbook (SPEC-112)

> Partners can act **now** without waiting for Waves A–E.  
> Evidence: [`00-incident-data.md`](00-incident-data.md) · Laws: [`01-first-principles.md`](01-first-principles.md)

## Immediate triage (shared PG connection pressure)

### 1. Confirm the boundary

```sql
SHOW max_connections;
SHOW superuser_reserved_connections;

SELECT count(*) AS client_backends
FROM pg_stat_activity
WHERE backend_type = 'client backend';
```

If `client_backends` ≈ `max_connections − reserved` → server limit. If much lower → look at intermediate pooler / app-side pool exhaustion.

### 2. Attribute by user / app / client

```sql
SELECT usename,
       COALESCE(NULLIF(application_name, ''), '(empty)') AS application_name,
       client_addr,
       state,
       count(*)
FROM pg_stat_activity
WHERE backend_type = 'client backend'
GROUP BY 1, 2, 3, 4
ORDER BY count(*) DESC;
```

**Today (v0.24.3+):** EdgeQuake rows show `edgequake:query|ingest|queue|admin`. Pre-0.24.3 pins often showed `(empty)` + last query `SET search_path TO public`.

### 3. Classify state

```sql
SELECT state, count(*)
FROM pg_stat_activity
WHERE backend_type = 'client backend'
GROUP BY 1;
```

| Dominant state | Meaning | First action |
|----------------|---------|--------------|
| `idle` | Held pool capacity | Reduce app pool sizes; stop nonessential replicas; do not “just” raise max_connections |
| `idle in transaction` | Leak / abandoned txn | `pg_terminate_backend` for old ones; set timeout |
| `active` | Slow queries / locks | `pg_locks` / `pg_stat_activity` query_start |

### 4. Safe relief (emergency)

1. Scale down EdgeQuake replicas **or** stop non-serving instances (migrate jobs, stray pods).
2. Prefer SIGTERM (graceful) over SIGKILL.
3. As last resort, terminate clearly idle EdgeQuake backends only with partner approval:

```sql
-- REVIEW FIRST — do not blind-fire in prod
SELECT pid, usename, application_name, client_addr, state, state_change, left(query, 80)
FROM pg_stat_activity
WHERE usename = 'edgequake'
  AND state = 'idle'
  AND state_change < now() - interval '30 minutes';
```

4. Raising `max_connections` requires restart and more memory — **band-aid only** (LAW-112-6).

---

## Sizing formula (LAW-112-3)

```text
EdgeQuake_need ≈ replicas × (QUERY + INGEST + QUEUE + ADMIN)
                 × deploy_overlap_factor   -- use 2 during rolling updates

Fleet_need ≈ EdgeQuake_need
           + QL_pools
           + other_services
           + admin_tools_headroom (≥5–10)
           + superuser_reserved

Require: Fleet_need ≤ max_connections
```

### claim_next / WORKER_THREADS (incident pattern)

Task workers use the **queue** pool only. Boot floors:

```text
queue_max = max(EDGEQUAKE_DB_POOL_SIZE_QUEUE, resolved_worker_count)
```

`resolved_worker_count` comes from `resolve_worker_pool_limits()` (honors `WORKER_THREADS` and local caps).

If logs show:

```text
pool timed out while waiting for an open connection  (claim_next)
unexpected response from SSLRequest: 0x00
```

1. Check Postgres health / recent container restarts (SSL 0x00 = dead/reset socket mid-handshake).
2. Confirm `/health` → `storage.db_pools` queue `max >=` worker count.
3. Restart backend after Postgres is healthy so pools re-form.

Do **not** raise server `max_connections` as the first fix.
### Defaults on HEAD (per EdgeQuake process)

| Env | Default |
|-----|--------:|
| `EDGEQUAKE_DB_POOL_SIZE_QUERY` | 16 |
| `EDGEQUAKE_DB_POOL_SIZE_INGEST` | 12 |
| `EDGEQUAKE_DB_POOL_SIZE_QUEUE` | 4 |
| `EDGEQUAKE_DB_POOL_SIZE_ADMIN` | 2 |
| **Sum** | **34** |

### Suggested shared-DB starting point (PPD co-tenant)

```bash
export EDGEQUAKE_DB_POOL_SIZE_QUERY=8
export EDGEQUAKE_DB_POOL_SIZE_INGEST=6
export EDGEQUAKE_DB_POOL_SIZE_QUEUE=2
export EDGEQUAKE_DB_POOL_SIZE_ADMIN=1
# sum = 17 per process
```

Example: 2 replicas, overlap 2 → plan for `2 × 17 × 2 = 68` EdgeQuake slots during rollout, plus QL.

---

## Monitoring (today)

| Signal | Where |
|--------|-------|
| Per-role pool size / idle | Prometheus via `/metrics` (`record_db_pool_stats_for_role`) |
| Ready blocked on store contention | `/ready` pool util (SPEC store contention) |
| Server-side | `pg_stat_activity` queries above; alert at **70%** of `max_connections` |

After Wave D: configured max per role in metrics/health.

---

## PgBouncer (optional, recommended for many services)

- Prefer **transaction** mode for typical CRUD; validate EdgeQuake `LISTEN` / AGE session needs first.
- Set `server_reset_query = DISCARD ALL` (aligns with LAW-P4 intent).
- Size Postgres `max_connections` modestly; let PgBouncer multiplex.

---

## Partner split of ownership

| Owner | Action |
|-------|--------|
| EdgeQuake | Pool budget, `application_name`, shutdown close, docs (this pack + Waves) |
| QL | Own pool sizes, `LISTEN` idle policy, `application_name` on QL backends |
| Platform | Shared `max_connections`, reserved slots, optional PgBouncer, alerting |

Stopping EdgeQuake to unblock QL is an **incident escape hatch**, not an operating model.
