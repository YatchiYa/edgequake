# 07 — Similar Issues Audit

| Site | Pre-106 | Post-106 |
|------|---------|----------|
| `pg_get_edges_for_nodes_batch` | **Raw graphid JOIN** (#356) | `::text` |
| `pg_get_nodes_with_degrees_batch` | Fixed #214 | unchanged |
| `pg_node_degree` / degrees batch | `::text` | unchanged |
| `query_ops/search.rs` popular nodes | `::text` | unchanged |
| `scan_ops.rs` BFS | `start_id::text = sv.id::text` | unchanged |
| `pg_get_edges_for_node_set` | property `source_id`/`target_id` ANY | unchanged (safe) |
| Native node/edge upsert | text `eq_*` / assignment | unchanged |
| GH #214 | Closed partial | Complements this fix |
| GH #161 AGE upgrade | Unrelated | — |
| SPEC-104 `graphid_ops` soft-skip | Related class | Still ops concern |

Audit command (expect zero hits):

```bash
rg 'src\.vid = e\.start_id|tgt\.vid = e\.end_id' edgequake/crates/edgequake-storage
```
