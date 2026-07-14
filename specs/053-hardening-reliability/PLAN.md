# SPEC-053 — Hardening Reliability: Battle-Tested Implementation Plan

## Principles: DRY + SOLID Applied

| Principle | Application                                                                                         |
| --------- | --------------------------------------------------------------------------------------------------- |
| **SRP**   | Each SQL helper owns one well-defined query shape; no multi-concern monster queries                 |
| **OCP**   | Indexes added via migration (closed to modification, open to extension)                             |
| **LSP**   | MemoryGraphStorage and PostgresAGEGraphStorage satisfy the same `get_incident_edges_batch` contract |
| **ISP**   | Contract tests assert minimal interface invariants — don't couple to internal SQL shape             |
| **DIP**   | API layer depends on `GraphStorage` trait, not on SQL internals                                     |
| **DRY**   | Single `edges_from_props_row` helper; single SQL builder for the UNION pattern                      |

---

## Task Breakdown

### T-1: Rewrite `pg_get_incident_edges_batch` (CRITICAL fix)

**File**: `edgequake/crates/edgequake-storage/src/adapters/postgres/graph/edges_ops.rs`

**Why**: Replaces O(V+E) parent-table full scan with O(k log E) btree UNION scan.

**New SQL pattern** (UNION forces two separate index scans):
```sql
SELECT ag_catalog.agtype_to_json(e.properties) AS props
FROM {graph}."EDGE" e
WHERE ag_catalog.agtype_to_json(e.properties)->>'source_id' IN ({in_list})
UNION
SELECT ag_catalog.agtype_to_json(e.properties) AS props
FROM {graph}."EDGE" e
WHERE ag_catalog.agtype_to_json(e.properties)->>'target_id' IN ({in_list})
```

**Key invariants**:
- Query `"EDGE"` child table (not `_ag_label_edge` parent) → indexes apply
- UNION (not UNION ALL) deduplicates edges where source AND target are both in frontier
- source/target extracted from `props` JSON (already stored on every edge)
- Chunk size raised from 100 → 200 (safe: O(log E) per chunk, not O(V))
- Rename `edges_from_sql_rows` to `edges_from_props_rows` (DRY: single extraction helper)

### T-2: Rewrite `pg_node_degrees_batch` (HIGH fix)

**File**: `edgequake/crates/edgequake-storage/src/adapters/postgres/graph/nodes_ops.rs`

**Why**: Same parent-table pattern — vertex lookup to get graphids, then edge JOIN for counts.
Replace with direct `"EDGE"` property aggregation using the same indexed columns.

**New SQL pattern**:
```sql
WITH out_deg AS (
    SELECT agtype_to_json(e.properties)->>'source_id' AS node_id, COUNT(*)::bigint AS cnt
    FROM {graph}."EDGE" e
    WHERE agtype_to_json(e.properties)->>'source_id' IN ({in_list})
    GROUP BY node_id
),
in_deg AS (
    SELECT agtype_to_json(e.properties)->>'target_id' AS node_id, COUNT(*)::bigint AS cnt
    FROM {graph}."EDGE" e
    WHERE agtype_to_json(e.properties)->>'target_id' IN ({in_list})
    GROUP BY node_id
)
SELECT n, COALESCE(o.cnt,0)+COALESCE(i.cnt,0) AS degree
FROM unnest(ARRAY[{in_list_vals}]) n
LEFT JOIN out_deg o ON o.node_id = n
LEFT JOIN in_deg   i ON i.node_id = n
```

### T-3: Update contract test assertions (CORRECTNESS)

**File**: `edgequake/crates/edgequake-storage/tests/contract_spec025_incident_edges_batch.rs`

Update string-match assertions:
- Remove: `"start_id::text = sv.id::text"` (old JOIN pattern)
- Add: `"EDGE"` direct table access
- Add: `"source_id"` property-based filtering
- Add: NOT `"_ag_label_vertex"` — verify vertex table eliminated

### T-4: Add migration 086 (SAFETY — existing deployments)

**File**: `edgequake/migrations/086_edge_bfs_index_reconcile.sql`

Purpose: Ensure `idx_edge_source_id` and `idx_edge_target_id` exist on all
existing graph schemas. Migration 070 kept these indexes but older databases
bootstrapped before M070 may be missing them.

This migration is a no-op (IF NOT EXISTS) on up-to-date databases.

### T-5: E2E spec for timeout regression (BATTLE-TESTED)

**File**: `edgequake_webui/e2e/spec053-reliability.spec.ts`

Tests:
1. Entity neighborhood API returns 200 (not 503 timeout) when graph is populated
2. Response time stays under 5s for depth=2 traversal
3. Verify `@graph-timeout` error is not shown in UI after query

---

## Performance Budget (Post-Fix)

| Query                                           | Before                                    | After (target)      |
| ----------------------------------------------- | ----------------------------------------- | ------------------- |
| `pg_get_incident_edges_batch` 100-node chunk    | ~200ms (seqscan) → timeout on large graph | **<10ms** (indexed) |
| `pg_node_degrees_batch` 100 nodes               | ~150ms (seqscan)                          | **<15ms** (indexed) |
| Entity neighborhood depth=2, frontier ~50 nodes | >15s (timeout)                            | **<500ms**          |

---

## Testing Strategy

### Unit / Integration Tests
- Existing: `contract_spec025_incident_edges_batch` (string assertion, updated)
- New: `contract_spec053_incident_edges_indexed` — asserts no `_ag_label_vertex` in SQL

### Contract Test (MemoryGraphStorage)
- Existing `contract_spec025_incident_edges_batch_matches_per_node_union` must still pass
- Semantics unchanged — only the PostgreSQL SQL rewritten

### E2E (Playwright)
- `spec053-reliability.spec.ts` — submits a query via the chat UI and verifies no timeout error

---

## Risk Assessment

| Risk                                                                    | Mitigation                                                                                             |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Edge property `source_id` / `target_id` may be absent on very old edges | `filter_map` in `edges_from_props_rows` silently skips malformed rows (existing behavior)              |
| UNION deduplication changes row count semantics                         | Contract test `batch.len() == 3` already validates deduplication                                       |
| `"EDGE"` child table may not exist until first edge written             | `if node_ids.is_empty() { return Ok(vec![]) }` early exit protects; empty EDGE table returns empty set |
| Migration 086 runs on fresh databases with no graphs                    | `IF NOT EXISTS (SELECT 1 FROM ag_catalog.ag_graph …)` guard                                            |
