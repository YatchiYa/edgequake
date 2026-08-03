# LENS — Database (SPEC-098)

## Question

How do we make typed fleet FK resolution **and** AGE edge upserts reliable across PostgreSQL **16, 17, and 18** without version-forked DDL?

## Answers

1. **Spine SSOT** remains `entities` / `relationships` with  
   `UNIQUE NULLS NOT DISTINCT (tenant_id, workspace_id, name)` (migration 001).  
2. **Fleet FKs** (`entity_embeddings.entity_id` → `entities.id`) from migration 130 — unchanged.  
3. **Migration 139** is a marker + `support/139/apply.sql` reconcile (patterned on 040):  
   - Paginated identity ensure from AGE vertices missing relational rows  
   - `INSERT … ON CONFLICT DO UPDATE` (portable on PG16+)  
   - No PG18-only features unless gated by `PostgresCapabilities`  
4. **Migration 140** is a marker + `support/140/apply.sql`:  
   - Single EDGE arbiter `idx_edge_eq_source_target_rel`  
   - Drop legacy `idx_edge_eq_source_target` / `idx_edge_source_target_unique`  
   - Refresh `_eq_sync_edge_ids` (prefer column, then props)  
   - Optional AGE→`relationships` spine ensure  
5. **Native upserts** must:  
   - Dedupe in Rust to arbiter key  
   - `DISTINCT ON` in SQL (defense in depth)  
   - Use `eq_merge_graph_properties` on DO UPDATE for **ingest** (SPEC-058)  
   - Use `properties = EXCLUDED.properties` (**Replace**) for cascade shared prune (LAW-098-12) — union merge undoes subtractive `source_ids`
   - Provenance collector must ignore edge topology `source_id`/`target_id`; cascade/delete keys `(src,tgt,rel)` (LAW-098-13)
6. **Preflight** notices `server_version_num` majors 16/17/18 for operators.  
7. **Runtime** identity ensure on saturated KEEP closes the hot path without waiting for backfill; arbiter reconcile runs every boot even when schema is “ready”.  
8. **Migration 141** extends `documents_valid_status` with lifecycle statuses `deleting` / `delete_failed` so delete admit can dual-write SQL (LAW-098-9). Portable drop/re-add CHECK (NOT VALID + VALIDATE when large tables matter).  
9. **Shell writers must pass through lifecycle statuses** (LAW-098-11). Mapping `delete_failed`→`failed` undoes dual-write and mislabels Retry Failed. If admit logs `spec098_sql_deleting_mirror_failed`, re-run `support/141/apply.sql`.

## Operator

```bash
# Entity spine (139)
psql "$DATABASE_URL" -f edgequake/migrations/support/139/apply.sql

# Edge arbiter + relationship spine (140)
psql "$DATABASE_URL" -f edgequake/migrations/support/140/apply.sql

# Document lifecycle statuses (141)
psql "$DATABASE_URL" -f edgequake/migrations/support/141/apply.sql

# Verify EDGE indexes (replace graph name)
psql "$DATABASE_URL" -c \
  "SELECT indexname FROM pg_indexes WHERE schemaname = 'default' AND tablename = 'EDGE' ORDER BY 1;"

# Mid-delete honesty
psql "$DATABASE_URL" -c \
  "SELECT id, status FROM documents WHERE status IN ('deleting','delete_failed');"
```

## Non-goals

- Dropping legacy `eq_*_vectors` (SPEC-091 / 131).  
- Changing RLS policies (SPEC-096 FORCE remains).  
- Upstream AGE Cypher index pushdown.  
- Soft-delete / undo; full CDC outbox for KV↔SQL.
