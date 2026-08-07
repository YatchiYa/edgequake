# LENS — Database Expert (SPEC-112)

## Process-per-connection

PostgreSQL gives each client backend a process. Idle backends still:

- Occupy a `max_connections` slot
- Consume baseline memory
- Add scheduler overhead at high counts

Raising `max_connections` to 400 without reducing app pools **amplifies** that cost (LAW-112-6).

## Idle vs idle in transaction

```text
  idle                  → slot held, no snapshot (usually)
  idle in transaction   → slot + snapshot; blocks VACUUM; leak smell
  active                → working or waiting on locks
```

PPD CSV: EdgeQuake rows were **idle / ClientRead**, last SQL `SET search_path TO public` — classic **pooled idle**, not abandoned transactions.

## Attribution

Without `application_name`, `GROUP BY application_name` collapses EdgeQuake into `(empty)` — ops cannot split query vs ingest saturation. LAW-112-4 is a database operability requirement.

## Budget inequality

```text
  Σ app_pool_max × instances(+overlap) + tools + reserved  ≤  max_connections
```

EdgeQuake alone defaults to **34** per process (`PgPoolBundle`). Two replicas with overlap ≈ **68** before QL.

## PgBouncer

| Mode | Pros | Cons for EdgeQuake |
|------|------|--------------------|
| transaction | High multiplexing | Session features (`LISTEN`, some prepared reuse, AGE path quirks) fragile |
| session | Safer for session GUCs | Weaker multiplexing |

Recommend only after validating LISTEN/AGE needs; until then, **shrink app pools** on shared PG.

## Server GUCs (ops)

| GUC | Role |
|-----|------|
| `max_connections` | Hard ceiling — change requires restart |
| `superuser_reserved_connections` | Break-glass slots |
| `idle_in_transaction_session_timeout` | Safety net (Wave C also sets per-session) |
| `tcp_keepalives_*` | Detect dead clients after hard kill |

## Diagnostic query pack

See [07-ops-runbook.md](../07-ops-runbook.md). Prefer captures **during** `too many clients already`, not after recovery.
