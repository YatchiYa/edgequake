# SPEC-098 — Entity spine + EDGE arbiter (operator)

Typed fleet embeddings require relational spine rows. Migrations **139** (entities)
and **140** (EDGE arbiter + relationships) are markers; support scripts reconcile
on PG **16 / 17 / 18** (portable SQL).

## Hot path

1. Saturated SOURCE_IDS KEEP still upserts the relational spine (AGE description
   mutation skipped). Reprocess should not fail with `typed fleet mirror resolved 0/N`.
2. Native AGE edge upserts use a **single** arbiter
   `idx_edge_eq_source_target_rel (eq_source_id, eq_target_id, eq_rel_type)`.
   Reprocess of multi-chunk extracts must not fail with
   `ON CONFLICT DO UPDATE command cannot affect row a second time`.
3. When `EDGEQUAKE_NATIVE_GRAPH_WRITES=0` (debug), Cypher MERGE also keys edges
   on `(source_id, target_id, relation_type)` — same multigraph semantics.

## Manual re-run

```bash
# Entity spine (AGE vertices → entities)
psql "$DATABASE_URL" -f edgequake/migrations/support/139/apply.sql

# EDGE arbiter hygiene + AGE edges → relationships
psql "$DATABASE_URL" -f edgequake/migrations/support/140/apply.sql
```

Progress:

```sql
SELECT value FROM server_config WHERE key = 'spec098_spine_ensure_progress';
SELECT value FROM server_config WHERE key = 'spec098_edge_arbiter_progress';
```

## EDGE index checklist

```sql
-- Replace 'edgequake' with your graph name (server_config.age_graph_name).
SELECT indexname
FROM pg_indexes
WHERE schemaname = 'edgequake' AND tablename = 'EDGE'
ORDER BY 1;
```

| Index | Required |
|-------|----------|
| `idx_edge_eq_source_target_rel` | **Yes** (3-col multigraph arbiter) |
| `idx_edge_eq_source_target` | **No** — must be dropped |
| `idx_edge_source_target_unique` | **No** — must be dropped |

Runtime bootstrap (`ensure_eq_id_columns` / `reconcile_legacy_graph_arbiters`)
drops legacy UNIQUEs even when the schema is already “ready”.

## 040 vs 139 vs 140

| Migration | Progress key | Role |
|-----------|--------------|------|
| 040 | (see support/040) | Historical CQRS entity dual-write backfill |
| 139 | `spec098_spine_ensure_progress` | Ensure bare `entities` for typed fleet FK |
| 140 | `spec098_edge_arbiter_progress` | Single EDGE arbiter + `relationships` spine |
