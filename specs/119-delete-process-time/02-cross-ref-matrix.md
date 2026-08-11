# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Singular probe required for poisoned citations | SPEC-098 Symptom F; `scan_ops.rs` singular SQL |
| Discovery timeout 2s | `SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS`; `LocalTimeoutTx` |
| Modern path uses GIN `source_ids` | SPEC-071; `idx_edge_source_ids_gin` |
| Plural citation GIN exists | SPEC-091 RM3; `idx_edge_source_chunk_ids_gin`; M137 marker |
| Parent edge prop indexes obsolete | M036 (historical) + M070 consolidate; `ensure_indexes` |
| Expression cast defeats index | [#362](https://github.com/raphaelmansuy/edgequake/issues/362); local EXPLAIN on `idx_edge_source_id` |
| Child-table index precedent | [#331](https://github.com/raphaelmansuy/edgequake/issues/331) → `idx_node_source_ids_gin` |
| Production timeout | [#375](https://github.com/raphaelmansuy/edgequake/issues/375) |
| Fix laws | SPEC-119 LAW-119-1..7 |

## Code SSOT (as-is → target)

| Concern | Path |
|---------|------|
| Singular + modern discovery | `edgequake-storage/.../graph/scan_ops.rs` (`pg_find_edges_by_source_prefixes`) |
| Timeout budget | `.../helpers/source_lineage_sql.rs` |
| Index DDL SSOT | `.../helpers/graph_lifecycle.rs` (`ensure_indexes`) |
| Delete cascade | `edgequake-api/.../document_graph_cascade.rs` → `find_document_edges` |
| Delete worker | `edgequake-api/.../document_deletion.rs` |
| Reprocess retract | `edgequake-api/.../retract_document_indexes.rs` + `reprocess.rs` |
| Timeout string detect (graph) | `edgequake-api/.../graph_materialization.rs` (`statement timeout`) |

## Related specs / issues

| Spec / Issue | Relationship |
|--------------|--------------|
| SPEC-098 | Introduced Symptom F singular orphan discovery |
| SPEC-071 | Indexed modern `source_ids` path; wall-budget pattern |
| SPEC-091 RM3 | Citation GIN + marker migration pattern (M137) |
| SPEC-069 | Delete progress / statement_timeout = 0 for DDL |
| GH #375 | This mission |
| GH #331 | Same defect class (missing index) on vertices |
| GH #362 | Same defect class (cast defeats index) |

## DRY rule

Singular citation extract expressions used in **filters and indexes** must share one shape: `ag_catalog.agtype_to_json(properties)->>'…'` (no `::jsonb` on the btree path). Do not invent a second extract helper that reintroduces the cast.
