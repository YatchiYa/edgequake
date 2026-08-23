# 01 — First principles

Inherits [SPEC-120 laws](../120-race-condition-legacy-vector/01-first-principles.md) **LAW-120-1**, **LAW-120-2**, **LAW-120-3**, **LAW-120-6**.

Postgres: `ON CONFLICT` exists only on [INSERT](https://www.postgresql.org/docs/current/sql-insert.html). [UPDATE](https://www.postgresql.org/docs/current/sql-update.html) has no equivalent. Unique violations are SQLSTATE **23505** (`unique_violation`). `NOT EXISTS` is racy under concurrency; uniqueness + absorb-on-23505 is the complete arbiter ([index unique checks](https://www.postgresql.org/docs/current/index-unique-checks.html)).

| Law | Statement |
|-----|-----------|
| **LAW-136-1** | Stamp-once UPDATE is an arbiter: if it would violate `(workspace_id, legacy_vector_id)`, absorb (0 rows stamped), do not `Err` |

Do **not** give the loser the same lid (LAW-120-2). Do not invent a second lid. Do not drop the unique index. Do not soft-Ok `fleet_provenance_stamp`. Alias spine merge remains SPEC-083.
