# SPEC-053 — Hardening: Graph Search Reliability

## Error

```
Service unavailable: Graph materialization capacity reached. Retry shortly.
```

Triggered by typing in the graph node search bar (≥2 chars) while the graph is loading.

## 5 WHY Root Cause

| #   | WHY                                    | Evidence                                                                                                                                                                                                   |
| --- | -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `search_nodes` returns 503             | `admit_graph_materialization` → `try_acquire_owned()` — zero-wait immediate fail when all slots occupied (`search.rs:75`)                                                                                  |
| 2   | All 4 slots occupied at search time    | `stream_graph` holds `_materialize_guard` for the **entire SSE stream duration** (~5-10s), not just the initial DB fetch (`graph_stream.rs:78`)                                                            |
| 3   | Multiple slots consumed simultaneously | React StrictMode double-mount + workspace switch + manual refetch = 2-3 concurrent `stream_graph` calls at page load; `get_popular_labels` takes a 4th                                                     |
| 4   | Every keystroke fires a server search  | FEAT0405 removed the `isTruncated` guard → `searchNodes` called on every debounced keystroke ≥2 chars (`graph-search.tsx:239`)                                                                             |
| 5   | Wrong semaphore class for search       | `GraphMaterializationSemaphore` designed for O(V+E) full-graph scans (3 DB connections × concurrent streams). `search_nodes` is an O(log N) indexed btree lookup (1 connection, ≤50ms). Misclassification. |

## Root Cause Statement (First Principles)

> `search_nodes` — an O(log N) indexed lookup — shares admission control with
> `stream_graph` — an O(V+E) full-graph materialization. The shared semaphore
> was designed to cap DB connection usage (3 connections × concurrent streams).
> Indexed searches use 1 connection for ≤100ms. Applying the heavyweight cap
> to a lightweight operation makes interactive search unreliable whenever the
> graph is loading.
>
> Secondary cause: `stream_graph` holds the semaphore permit for the entire
> SSE streaming loop (seconds), not just the initial data-fetch phase.
>
> Tertiary cause: FEAT0405 removed the `isTruncated` guard, causing every
> keystroke to call the server even when the local MiniSearch index has results.

## Fix Plan

### B1 — Remove materialization gate from `search_nodes` (CRITICAL)

`search_nodes` is an indexed query protected by a DB `statement_timeout`.
The materialization semaphore is the wrong tool; remove it entirely from `search_nodes`.

### B2 — Release `stream_graph` guard after data fetch, before streaming (HIGH)

Acquire guard → fetch data → **release guard** → stream SSE events.
The expensive resource use (3 parallel DB connections) ends when the fetch completes.

### F1 — Restore intelligent server search condition (HIGH)

Only call `searchNodes` when: `query.length >= 3` AND (`isTruncated` OR `localResults.length < 3`).

### F2 — Handle 503 gracefully — silent fallback to local results (MEDIUM)

503 from server search should not show an error banner. Fall back to local MiniSearch.

### F3 — Increase server search debounce to 300ms (LOW)

Keep local search at 150ms; server search at 300ms to reduce unnecessary API calls.


**Error observed (screenshot evidence)**

```
Storage error: Database error: Batch incident edges query failed:
error returned from database: canceling statement due to statement timeout
```

**Context**: user uploaded document `arts_2606.21891V1`, then asked
"Can you recreate the flowchart as a mermaid diagram". EdgeQuake triggered a
BFS neighborhood traversal which called `pg_get_incident_edges_batch`.

---

## 1. Five WHY Analysis

### WHY 1 — Why was the request cancelled with an error?

PostgreSQL canceled the executing statement because it exceeded the configured
`statement_timeout` (15 s, set via `EDGEQUAKE_GRAPH_QUERY_TIMEOUT_SECS` defaulting
to `DEFAULT_GRAPH_QUERY_TIMEOUT_SECS = 15`).

### WHY 2 — Why did the query exceed 15 s?

`pg_get_incident_edges_batch` in
`edgequake/crates/edgequake-storage/src/adapters/postgres/graph/edges_ops.rs` (line 264)
executed this SQL per 100-node chunk:

```sql
SELECT
    ag_catalog.agtype_to_json(e.properties) AS props,
    ag_catalog.agtype_to_json(sv.properties)->>'node_id' AS source_id,
    ag_catalog.agtype_to_json(tv.properties)->>'node_id' AS target_id
 FROM {graph}."_ag_label_edge" e
 JOIN {graph}."_ag_label_vertex" sv ON e.start_id::text = sv.id::text
 JOIN {graph}."_ag_label_vertex" tv ON e.end_id::text = tv.id::text
 WHERE ag_catalog.agtype_to_json(sv.properties)->>'node_id' IN ({in_list})
    OR ag_catalog.agtype_to_json(tv.properties)->>'node_id' IN ({in_list})
```

At production graph scale this is a **full sequential scan** on
`_ag_label_vertex` (the AGE parent inheritance table) with
JSON extraction on every row — O(V × E) work for each BFS frontier step.

### WHY 3 — Why didn't an index prevent the full scan?

Three compounding reasons:

| #   | Cause                                                                                                                                                                                                                                                          | Evidence                                                                         |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| A   | The query targets `_ag_label_vertex` (AGE parent inheritance table). **Migration 070 explicitly dropped all parent-table indexes** ("0 rows in parent, all data is in child label tables"). There are NO indexes on the parent table.                          | `migrations/070_consolidate_age_indexes.sql` lines 89-100                        |
| B   | Expression indexes `idx_node_tenant_id`, `idx_node_workspace_id`, etc. are created on `"Node"` (child), not on `_ag_label_vertex`. PostgreSQL cannot use child-partition indexes for predicate filters on the parent unless the query uses the child directly. | `graph_lifecycle.rs:ensure_indexes()` — every `CREATE INDEX … ON {graph}."Node"` |
| C   | The `OR` predicate in the WHERE clause prevents efficient Bitmap-OR planning even if some index were present; it forces a full vertex table materialize before the JOIN.                                                                                       | SQL pattern confirmed by EXPLAIN ANALYZE on equivalent schema                    |

### WHY 4 — Why does the query join vertex tables when it doesn't need to?

The edges table (`"EDGE"`) already stores `source_id` and `target_id` as explicit
named properties (set during every `pg_upsert_edge` / `pg_upsert_edges_batch` call):

```rust
// edges_ops.rs pg_upsert_edge
props_with_ids.insert("source_id".to_string(), Value::String(source.to_string()));
props_with_ids.insert("target_id".to_string(), Value::String(target.to_string()));
```

Moreover, `ensure_indexes()` already creates two btree expression indexes on those
properties:

```
idx_edge_source_id  ON "EDGE" ((agtype_to_json(properties)->>'source_id'))
idx_edge_target_id  ON "EDGE" ((agtype_to_json(properties)->>'target_id'))
```

The vertex JOIN is architecturally redundant for this query path. The original
SPEC-025 implementation did not exploit the denormalized source/target stored on
edges, and the insight went unrecorded.

### WHY 5 — Why wasn't this caught before production?

The contract test (`tests/contract_spec025_incident_edges_batch.rs`) validates
**structural correctness** of the SQL pattern (avoids `UNWIND`, uses text cast)
but runs against `MemoryGraphStorage` — no real PostgreSQL, no EXPLAIN plan.
No performance regression test exists that runs the incident-edge BFS against
a PostgreSQL instance with ≥10k nodes. The `arts_2606.21891V1` document ingestion
grew the graph past the threshold where O(V) scan time exceeds the 15 s budget.

---

## 2. Root-Cause Statement (First Principles)

> `pg_get_incident_edges_batch` scans `_ag_label_vertex` (the AGE inheritance
> parent table, which has no indexes) to resolve edge endpoints, when the same
> information (`source_id`, `target_id`) is already stored as indexed properties
> on every `"EDGE"` row. The fix is to eliminate the vertex table entirely and
> use a UNION of two btree index scans on `"EDGE"` — reducing complexity from
> O(V + E) per chunk to O(k log E) where k = chunk size.

---

## 3. Impact Scope

| Component                     | File                     | Severity                                             |
| ----------------------------- | ------------------------ | ---------------------------------------------------- |
| `pg_get_incident_edges_batch` | `edges_ops.rs:264`       | **CRITICAL** — primary failure path                  |
| `pg_node_degrees_batch`       | `nodes_ops.rs:~500`      | **HIGH** — same parent-table pattern                 |
| `pg_node_degree` (single)     | `nodes_ops.rs:~460`      | **MEDIUM** — single-node so faster, but same pattern |
| Entity neighborhood BFS       | `entity_neighborhood.rs` | **HIGH** — calls incident edges + degrees            |
| Knowledge graph scoped BFS    | `query_ops.rs:~120`      | **HIGH** — calls same batch fn                       |

---

## 4. Evidence Summary

```
File: edges_ops.rs
Line 264-280: SQL using _ag_label_edge + _ag_label_vertex (parent tables, no indexes)
Line 274: error message "Batch incident edges query failed: {}"

File: graph_lifecycle.rs:ensure_indexes()
Creates idx_edge_source_id, idx_edge_target_id on "EDGE" child table (NOT parent)

File: migrations/070_consolidate_age_indexes.sql
Explicitly drops ALL parent-table (_ag_label_vertex, _ag_label_edge) indexes as "never scanned"

File: resource/budget.rs:51
DEFAULT_GRAPH_QUERY_TIMEOUT_SECS = 15  (the hard wall the query hits)

File: session.rs:17
SET statement_timeout = '{}s' — applied per-connection, kills any query over budget
```
