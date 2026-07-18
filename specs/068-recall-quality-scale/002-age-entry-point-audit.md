# SPEC-068 — AGE entry-point index audit

**Stack:** PostgreSQL 18 + Apache AGE **1.8.0** (pin `PG18/v1.8.0-rc0`)  
**Guidance applied:** index entry points; prefer O(entry+hops); keep native SQL expand (Azure AGE perf + AGE 1.8 VLE/index-scan notes).

## Entry-point indexes (created in `ensure_indexes` / `ensure_eq_id_columns`)

| Index | Table | Purpose | Hot path |
|-------|-------|---------|----------|
| `idx_node_id` | `"Node"(id)` | AGE graphid PK lookup | Cypher / id joins |
| `idx_node_eq_node_id` | `"Node"(eq_node_id)` UNIQUE | App `node_id` arbiter (SPEC-062) | Native upsert / lookup |
| `idx_node_tenant_id` / `idx_node_workspace_id` | expression on properties | Tenant/workspace filter | Scoped scans |
| `idx_node_source_id_expr` | expression | Document source entry | Ingest / retract |
| `idx_edge_start_id` / `idx_edge_end_id` | `"EDGE"(start_id\|end_id)` | Traversal endpoints | Expand |
| `idx_edge_start_id_text` / `idx_edge_end_id_text` | cast text | Text join with `::text` | Degrees / search |
| `idx_edge` eq_source/target + UNIQUE | denorm text | SPEC-062 native edge ops | Degrees without `agtype_to_json` |

Source: [`graph_lifecycle.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/graph/helpers/graph_lifecycle.rs) `ensure_indexes` / `ensure_eq_id_columns`.

## Audit result (code review)

- **Pass:** Hot expand/degrees use `eq_*` btree columns (SPEC-062) — O(entry set), not per-row `agtype_to_json` on the write path.
- **Pass:** AGE parent-table indexes intentionally removed (0 rows / write amp) — child `"Node"` / `"EDGE"` only.
- **Pass:** No Cypher rewrite of native expand required for SPEC-068.
- **Action:** Remasure G1 after AGE 1.8 image rebuild; archive under `e2e/artifacts/`.

## Non-goals

- Raising community Louvain / full-graph scan threshold (stays 50k)
- DiskANN
