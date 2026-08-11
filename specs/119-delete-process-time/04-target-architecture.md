# 04 — Target Architecture

## Serving path

```ascii
  pg_find_edges_by_source_prefixes
       │
       ├─ modern:  (agtype_to_json)::jsonb -> 'source_ids' @> probe
       │              └── idx_edge_source_ids_gin          (unchanged)
       │
       └─ singular: agtype_to_json(properties)->>'source_chunk_id' IN probes
                    OR agtype_to_json(properties)->>'source_document_id' IN probes
                         │
                         ├─ idx_edge_source_chunk_id       (NEW btree)
                         └─ idx_edge_source_document_id    (NEW btree)
                              │
                              ▼
                         BitmapOr / Nested Loop Index Scan
                         within 2s discovery budget
```

## Index DDL (ensure_indexes SSOT)

```sql
CREATE INDEX IF NOT EXISTS idx_edge_source_chunk_id
  ON {graph}."EDGE"
  ((ag_catalog.agtype_to_json(properties)->>'source_chunk_id'));

CREATE INDEX IF NOT EXISTS idx_edge_source_document_id
  ON {graph}."EDGE"
  ((ag_catalog.agtype_to_json(properties)->>'source_document_id'));
```

Applied by `PostgresAGEGraphStorage::ensure_indexes` (single-flight). Recorded in fleet via marker migration `145_spec119_edge_singular_citation_indexes.sql`.

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| SRP | `ensure_indexes` owns DDL; `scan_ops` owns query shape; cascade owns product orchestration |
| OCP | Add indexes without changing cascade API |
| DRY | Singular extract matches existing `idx_edge_source_id` style (no second cast dialect) |
| DIP | Callers stay on `GraphScanOps` trait |

## Explicit non-targets

```ascii
  ❌ CREATE INDEX on {graph}."_ag_label_edge"   (parent; not queried)
  ❌ Raise SOURCE_DISCOVERY_STATEMENT_TIMEOUT as primary fix
  ❌ Drop Symptom F probe
```
