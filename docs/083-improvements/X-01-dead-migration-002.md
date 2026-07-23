# X-01 — Dead migration 002 (operator note)

Canonical documentation: [`edgequake/migrations/README.md`](../../edgequake/migrations/README.md).

**Summary**: `001_init_database.sql` owns the `tasks` UUID PK. `002_add_tasks_table.sql` is a no-op after 001 (`CREATE TABLE IF NOT EXISTS`). Do not expect 002 to change PK semantics.
