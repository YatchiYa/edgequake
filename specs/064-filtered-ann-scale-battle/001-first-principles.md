# SPEC-064 — First principles

## Binding constraint (from SPEC-063 L1 + SPEC-064 WAVE0)

| Signal | Value | Reading |
|--------|-------|---------|
| Cold single @100k/1536 | p95 ~1470ms | Q1-d miss — **query wall** |
| Warm single | p95 ~50–70ms | Same plan, residency win |
| EXPLAIN shape | btree `(tenant,ws)` → **exact** distance on ~20k rows | **Not** HNSW iterative walk at 20% filter |
| HNSW create @100k | ~7–80s | Cold ingest OK — not the product cliff |

## Cost model

```
cost ≈ embedding_bytes × filtered_heap_rows × (1 + I/O_miss_penalty)
```

- **DIM bytes:** `vector` = 4×D; `halfvec` = 2×D → residency win at fixed RAM.
- **Filter shape @ ~20%:** planner prefers **Bitmap Heap + Sort** over global HNSW; cliff = cold exact scan of the workspace slice.
- **Partial HNSW `WHERE workspace_id = $ws`:** usable when SQL implies the predicate (column-only filter); keeps hot-WS ANN option when selectivity worsens.

## Deferred

DiskANN / pgvectorscale, Mix RRF redesign, blind global `ef_search` bump without recall, workspace GB quotas.
