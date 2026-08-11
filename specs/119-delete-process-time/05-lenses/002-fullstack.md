# Lens 002 — Full Stack Developer

## Touch points

```ascii
  storage: scan_ops.rs (singular SQL expression)
  storage: graph_lifecycle.rs (ensure_indexes entries)
  migrations: 145_spec119_*.sql (marker)
  tests: contract EXPLAIN + wall e2e
  api (optional): friendlier timeout mapping on deletion path
```

## Implementation checklist

1. Change singular filter to btree-matching `->>'…'` (no `::jsonb`).
2. Add `idx_edge_source_chunk_id` + `idx_edge_source_document_id` next to existing edge btrees.
3. Marker migration documents runtime apply (pattern M137).
4. Contract asserts indexes exist after `ensure_indexes`.
5. EXPLAIN asserts Index Cond / Bitmap / Index Scan — not Seq Scan for singular equality.
6. Keep modern GIN path’s `::jsonb -> 'source_ids'` untouched.

## Risks

| Risk | Mitigation |
|------|------------|
| Index CREATE locks large EDGE | `CREATE INDEX IF NOT EXISTS` at ensure_indexes with DDL timeout 0 (existing pattern) |
| Cast sneak-back | Unit/contract on SQL fragment; EXPLAIN gate |
| Parent-table temptation | Explicit assert plan does not use `_ag_label_edge` |
