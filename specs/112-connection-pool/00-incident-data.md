# 00 — Incident data (`pg_stat_activity`)

> Source: partner CSV copied to [`measurements/pg_stat_activity.csv`](measurements/pg_stat_activity.csv).  
> Captured: **2026-08-07 ~11:02–11:10 +0200** (backend_start / query_start windows).

## Aggregate counts

| Slice | Count |
|-------|------:|
| Total rows | 31 |
| Client backends | 23 |
| Background (autovacuum, walwriter, …) | 8 |
| `datname=edgequake` | **10** |
| `datname=ppd_quantalogic_db` | 10 |
| `datname=postgres` | 3 |
| Client state `idle` | 22 |
| Client state `active` | 1 (DBeaver count of `pg_stat_activity`) |

## EdgeQuake slice (all 10)

| Field | Value |
|-------|-------|
| `usename` | `edgequake` |
| `client_addr` | `10.79.1.84` (single host) |
| `state` | `idle` (all) |
| `wait_event` | `ClientRead` (all) |
| `application_name` | **empty** (all) |
| Last `query` | `SET search_path TO public` (all) |
| `backend_start` | ~11:02:03–11:03:12 |

```text
edgequake@10.79.1.84
┌────────────────────────────────────────────────────────────┐
│ 10 × idle ClientRead                                       │
│ application_name = ""                                      │
│ query = SET search_path TO public                          │
│ ← matches with_session_hygiene after_connect / after_release│
│ ← cannot tell query vs ingest vs queue vs admin            │
└────────────────────────────────────────────────────────────┘
```

## Co-tenants visible in the same snapshot

| Actor | Evidence |
|-------|----------|
| QL (`quantadbu`) | idle backends on `ppd_quantalogic_db`; includes `LISTEN ql_tasks_changes` |
| Monitor (`quanta_monitor`) | `SELECT * FROM pg_stat_database_conflicts` |
| DBeaver | multiple metadata / SQLEditor / data-transfer sessions |
| psql | admin grouping query on `pg_stat_activity` |

## What this snapshot is

1. Proof EdgeQuake holds **idle** backends whose last SQL is the pool hygiene `search_path` pin.
2. Proof backends are **unattributed** (`application_name` empty) — LAW-112-4 gap.
3. Proof the database is **shared** (EdgeQuake + QL + tools on one instance).
4. Consistent with partner narrative: stopping EdgeQuake frees slots for others.

## What this snapshot is not

1. **Not** a peak `max_connections` exhaustion capture (23 clients ≪ 400).
2. **Not** proof of a per-connection leak into hundreds of zombies.
3. **Not** role-split proof (no `application_name` → cannot map to `PgPoolBundle` roles).
4. **Not** proof that 400 was required — only that ops raised the ceiling.

Full honesty write-up: [`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md).

## Recommended follow-up captures (ops)

```sql
SHOW max_connections;
SHOW superuser_reserved_connections;

SELECT usename, application_name, client_addr, state, count(*)
FROM pg_stat_activity
WHERE backend_type = 'client backend'
GROUP BY 1,2,3,4
ORDER BY count(*) DESC;

SELECT count(*) AS backends,
       count(*) FILTER (WHERE state = 'idle') AS idle,
       count(*) FILTER (WHERE state = 'idle in transaction') AS idle_in_xact,
       count(*) FILTER (WHERE state = 'active') AS active
FROM pg_stat_activity
WHERE backend_type = 'client backend';
```

Capture again **during** refusal errors (`FATAL: too many clients already`), not only after partial recovery.
