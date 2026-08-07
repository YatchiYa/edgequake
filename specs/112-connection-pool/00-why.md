# 00 — Why SPEC-112

## Trigger

Partner message (PPD, 2026-08-07):

> Connection pool problem in PPD. Many **idle** connections on EdgeQuake; stopping the service let other services reconnect to the DB. Please check whether EdgeQuake pools are managed correctly. QL will investigate their side too. Strange: `max_connections` was set to **400**. Attached: `pg_stat_activity` snapshot at the time of the problem.

Snapshot artifact: [`measurements/pg_stat_activity.csv`](measurements/pg_stat_activity.csv) — analyzed in [`00-incident-data.md`](00-incident-data.md).

## User impact

| Layer | Impact if ignored |
|-------|-------------------|
| Ops | Shared PostgreSQL becomes a single point of failure across EdgeQuake + QL + admin tools |
| Reliability | Recovery playbook becomes “restart / stop EdgeQuake” — unacceptable for multi-service fleets |
| Capacity | Raising `max_connections` to 400 hides oversubscription and increases PG memory / scheduler cost |
| Diagnosis | Empty `application_name` makes it impossible to attribute backends to query vs ingest vs queue vs admin |
| Deploy | Rolling updates can transiently double held slots (old + new pods) and tip a shared DB over the limit |

## Why this pack (not a one-line “tune the pool”)

1. **Code is law** — production already uses a four-role `PgPoolBundle` (SPEC-090). The defect is governance of that design on a **shared** database, not “missing pooling.”
2. **Idle ≠ free** — every idle sqlx checkout still occupies a PostgreSQL backend process and a `max_connections` slot.
3. **Partner trust** — QL and EdgeQuake share one PG; EdgeQuake must be a good co-tenant with measurable budget.
4. **Honesty** — the attached CSV is not peak saturation evidence; we must not invent a leak of 400 connections from a 10-row EdgeQuake slice.

## Non-goals

- Hot-patching partner PostgreSQL GUCs as the product “fix.”
- Redesigning AGE / LISTEN / migrate onto a different connection model in this pack.
- Claiming EdgeQuake alone caused the full incident without peak-window evidence.
- Implementing Waves A–E in this documentation deliverable.

## Success condition

- Partners can diagnose and size pools using [`07-ops-runbook.md`](07-ops-runbook.md) today.
- Engineering has a DRY/SOLID wave plan ([`04-fix-plan.md`](04-fix-plan.md)) with e2e gates ([`05-e2e-test-matrix.md`](05-e2e-test-matrix.md)).
- Every LAW-112 maps to code symbols and tests to write ([`02-cross-ref-matrix.md`](02-cross-ref-matrix.md)).
- Brutal honesty states what the CSV proves and what it does not ([`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md)).
