# SPEC-040 — PostgreSQL AGE / pgvector Expert Lens

**Lens:** PostgreSQL 16 + Apache AGE + pgvector  
**Focus:** Index inheritance, planner statistics, migration safety

---

## AGE storage model (code is law)

Apache AGE in EdgeQuake uses **PostgreSQL table inheritance**:

```
_ag_label_vertex (parent, ~0 rows)
    └── "Node" (child — ALL vertex data)

_ag_label_edge (parent, ~0 rows)
    └── "EDGE" (child — ALL edge data)
```

Evidence: `migrations/070_consolidate_age_indexes.sql:17-21`, `graph_lifecycle.rs:218-222`.

**Critical rule:** Indexes on parent tables are **never scanned** for label queries. Migration 014 created workspace indexes on `_ag_label_vertex` — useless for planner on `"Node"`.

---

## Issue #262 — Query anatomy

### Failing pattern (from issue #262 + code)

```sql
WITH filtered_nodes AS MATERIALIZED (
  SELECT v.id::text AS id_text, v.properties
  FROM eq_eq_default_graph."_ag_label_vertex" v
  WHERE ag_catalog.agtype_to_json(v.properties)->>'workspace_id' = $1
),
edge_counts AS (
  SELECT e.start_id::text AS start_id_text, COUNT(*) AS out_degree
  FROM eq_eq_default_graph."_ag_label_edge" e
  INNER JOIN filtered_nodes fn ON e.start_id::text = fn.id_text
  GROUP BY e.start_id_text
)
SELECT ... FROM filtered_nodes fn
LEFT JOIN edge_counts ec ON fn.id_text = ec.start_id_text
```

Source: `query_ops.rs:511-526` (same structure for popular labels at 186-203).

### Why Nested Loop happens

1. **Underestimated cardinality** on `workspace_id` expression → planner thinks `filtered_nodes` ≈ 1 row.
2. **Missing index** on `"Node"` for `(agtype_to_json(properties)->>'workspace_id')`.
3. **Cast join** `start_id::text` without expression index on `"EDGE"` — M072 adds `idx_*_edge_start_id_text`.

### Correct plan target

```
Hash Left Join
  -> Seq/Index Scan on filtered_nodes (Index Scan using idx_node_workspace_id)
  -> Hash
       -> HashAggregate on edge_counts (Index Scan using idx_*_edge_start_id_text)
```

---

## Index SSOT today

| Index | Table | Created by | Used by |
| ----- | ----- | ---------- | ------- |
| `idx_node_workspace_id` | `"Node"` | `graph_lifecycle.rs:170-177` | workspace COUNT, filters |
| `idx_node_tenant_id` | `"Node"` | `graph_lifecycle.rs:160-167` | tenant filters |
| `idx_node_prop_node_id_btree` | `"Node"` | graph_lifecycle | batch node fetch |
| `idx_edge_start_id` | `"EDGE"` | graph_lifecycle | raw graphid |
| `idx_*_edge_start_id_text` | `"EDGE"` | M072 | BFS, popular nodes |
| ~~idx on `_ag_label_vertex`~~ | parent | M014 (dropped M070) | **never** |

---

## Recommended migration M078 (battle-tested)

**Purpose:** Idempotent child-table index + stats repair for all graphs (fixes legacy installs reporting #262).

```sql
-- migrations/078_age_child_workspace_stats.sql (proposed)
DO $$
DECLARE v_graph text;
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN RETURN; END IF;

  FOR v_graph IN SELECT name FROM ag_catalog.ag_graph LOOP
    -- Skip if Node label not materialized yet (SPEC-039)
    IF to_regclass(format('%I."Node"', v_graph)) IS NULL THEN CONTINUE; END IF;

    EXECUTE format(
      'CREATE INDEX IF NOT EXISTS idx_node_workspace_id ON %I."Node"
       ((ag_catalog.agtype_to_json(properties)->>>''workspace_id''))', v_graph);
    EXECUTE format(
      'CREATE INDEX IF NOT EXISTS idx_node_tenant_id ON %I."Node"
       ((ag_catalog.agtype_to_json(properties)->>>''tenant_id''))', v_graph);
    EXECUTE format(
      'CREATE INDEX IF NOT EXISTS idx_edge_start_id_text ON %I."EDGE"
       ((start_id::text))', v_graph);

    EXECUTE format('ANALYZE %I."Node"', v_graph);
    EXECUTE format('ANALYZE %I."EDGE"', v_graph);
  END LOOP;
END $$;
```

**Edge cases:**

| Case | Handling |
| ---- | -------- |
| Graph empty (no EDGE yet) | Skip EDGE index; Node index still valid |
| Large graph (>100k nodes) | Use `support/078/concurrent.sql` pattern from M038 |
| AGE not installed | No-op (CI memory mode) |
| Multiple graphs (multi-tenant) | Loop all `ag_graph` names |

---

## pgvector interaction (#262 secondary)

Workspace stats **does not** scan all vectors — chunk counts use per-doc prefix (`stats.rs:257-264`). Slowness is **graph-bound**, not HNSW.

Post-M071 HNSW rebuild at startup can add deploy latency — unrelated to #262 timeout during stats poll.

---

## Verification runbook (operator)

```bash
# 1. Confirm child indexes exist and are used
psql "$DATABASE_URL" -c "
SELECT schemaname, tablename, indexname, pg_size_pretty(pg_relation_size(indexrelid))
FROM pg_stat_user_indexes
WHERE schemaname LIKE 'eq_%' AND indexrelname LIKE '%workspace%';"

# 2. EXPLAIN workspace popular nodes (replace schema + workspace UUID)
psql "$DATABASE_URL" -c "
EXPLAIN (ANALYZE, BUFFERS)
WITH filtered_nodes AS MATERIALIZED (
  SELECT v.id::text AS id_text, v.properties
  FROM eq_eq_default_graph.\"_ag_label_vertex\" v
  WHERE ag_catalog.agtype_to_json(v.properties)->>'workspace_id' = 'YOUR_WS_UUID'
)
SELECT COUNT(*) FROM filtered_nodes;"

# 3. Index scan count monotonic increase under load
SELECT indexrelname, idx_scan FROM pg_stat_user_indexes
WHERE schemaname = 'eq_eq_default_graph' ORDER BY idx_scan DESC;
```

**Pass criteria:** `idx_node_workspace_id` idx_scan > 0; EXPLAIN shows Index Scan or Bitmap Index Scan; execution <500ms for 27k filtered nodes.

---

## Issue #259 — PostgreSQL angle

FK `messages_conversation_id_fkey` is correct relational design. Performance degradation from #262 causes **connection pool hold time** increase:

- Long graph queries occupy pool connections
- Conversation writes block → cascading timeouts

Fix #262 first to reduce pool pressure; add conversation re-validation separately.

---

## Issue #253 — PostgreSQL angle

Duplicate hashes live in **KV table** (`eq_*_kv`), not AGE. Orphan detection requires LEFT JOIN logic:

```sql
-- Conceptual: hash key exists, metadata key absent
SELECT h.key FROM kv h
LEFT JOIN kv m ON m.key = replace(h.key, '-hash-', '-metadata-') -- simplified
WHERE h.key LIKE '%-hash-%' AND m.key IS NULL;
```

Implement in Rust via existing `KVStorage` traits — no new SQL table.

---

## DRY with existing migrations

| Do | Don't |
| -- | ----- |
| Extend `graph_lifecycle.rs` index list | Duplicate index DDL in 014, 046, docker/init.sql |
| Reuse `eq_drop_graph_index_if_exists` from M070 | Manual DROP in ops runbooks |
| Call ANALYZE after bulk ingest job | Rely on autovacuum alone on expression indexes |

---

## Rollback

M078 is additive (CREATE INDEX IF NOT EXISTS + ANALYZE). Rollback = DROP INDEX if needed; **no data loss**.
