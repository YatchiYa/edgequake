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
   - Use `eq_merge_graph_properties` on DO UPDATE (SPEC-058)  
6. **Preflight** notices `server_version_num` majors 16/17/18 for operators.  
7. **Runtime** identity ensure on saturated KEEP closes the hot path without waiting for backfill; arbiter reconcile runs every boot even when schema is “ready”.

## Operator

```bash
# Entity spine (139)
psql "$DATABASE_URL" -f edgequake/migrations/support/139/apply.sql

# Edge arbiter + relationship spine (140)
psql "$DATABASE_URL" -f edgequake/migrations/support/140/apply.sql

# Verify EDGE indexes (replace graph name)
psql "$DATABASE_URL" -c \
  "SELECT indexname FROM pg_indexes WHERE schemaname = 'default' AND tablename = 'EDGE' ORDER BY 1;"
```

## Non-goals

- Dropping legacy `eq_*_vectors` (SPEC-091 / 131).  
- Changing RLS policies (SPEC-096 FORCE remains).  
- Upstream AGE Cypher index pushdown.
