# Lens 003 — Database Expert

## Serving table

Live discovery queries `FROM {graph}."EDGE" e` (AGE child). Parent `_ag_label_edge` may retain historical M036 indexes; they are **not** the fix surface.

## Index design

| Name | Expression | Type | Query shape |
|------|------------|------|-------------|
| `idx_edge_source_chunk_id` | `(agtype_to_json(properties)->>'source_chunk_id')` | btree | `=`, `IN` |
| `idx_edge_source_document_id` | `(agtype_to_json(properties)->>'source_document_id')` | btree | `=`, `IN` |

OR of two keys → expect **BitmapOr** of two bitmap index scans (or nested loop index probes), not Seq Scan.

## Expression matching law

`agtype_to_json` returns **json**. Indexing `json->>'k'` is not the same expression as `(json)::jsonb->>'k'`. Proven:

```text
idx_edge_source_id used without ::jsonb
Seq Scan with ::jsonb on the same property
```

## Observability

```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT 1
FROM {graph}."EDGE" e
WHERE ag_catalog.agtype_to_json(e.properties)->>'source_chunk_id'
      = 'known-chunk-id';
-- Expect: Index Scan / Bitmap Index Scan on idx_edge_source_chunk_id
```

## Migration strategy

Marker-only sqlx version (like M137). Real DDL via `ensure_indexes` so new graphs and existing fleets converge without hand-applied CONCURRENTLY ops (operators may still CONCURRENTLY in maintenance windows for very large EDGE tables if startup CREATE is too heavy — document operational note).
