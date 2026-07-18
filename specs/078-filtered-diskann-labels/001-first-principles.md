# SPEC-078 — First principles (Filtered-DiskANN labels)

## Why labels in the DiskANN graph

Post-filter ANN underfills when selectivity is low: the index returns approximate neighbors, then `WHERE workspace_id = …` discards most of them. Wave-2 fixes this for HNSW with **partial indexes**. Dedicated DiskANN @150k avoids filters by isolating one workspace per table.

**Shared-table DiskANN** needs a different fix: put the filter **into the graph walk**. Microsoft Filtered-DiskANN (WWW’23) and pgvectorscale label filtering do that with `smallint[]` labels included in the index.

Official pgvectorscale 0.9.0 pattern:

```sql
-- Column (smallint range only)
ALTER TABLE items ADD COLUMN labels smallint[];

-- Index includes labels
CREATE INDEX ON items
  USING diskann (embedding vector_cosine_ops, labels);

-- Query: overlap pushes filter into traversal
SELECT id FROM items
WHERE labels && ARRAY[$ws_label]::smallint[]
ORDER BY embedding <=> $q
LIMIT 20;
```

Arbitrary `WHERE workspace_id = …` on an embedding-only DiskANN index falls back to **post-filtering** (honesty baseline in this bake-off).

## UUID → dense smallint

pgvectorscale labels are `smallint` (−32768…32767). Workspace UUIDs / string IDs cannot be indexed as labels directly.

| Rule | Meaning |
|------|---------|
| Dense assign | Map each workspace string → unique `i16` (1…) |
| Bound | Fail closed at 32767 distinct workspaces in the map |
| Lifecycle | Remap requires rebuild of DiskANN+labels index |
| Tenant | Optional second label later; smoke uses workspace-only |

Helper: `WorkspaceLabelMap` (harness / future opt-in — not boot default).

## EdgeQuake placement

| Layer | Role |
|-------|------|
| Wave-2 halfvec + partial HNSW | **Supported default** @100k |
| Dedicated DiskANN + list/rescore | **Supported opt-in** @150k |
| Filtered-DiskANN labels (this pack) | Opt-in **study** for shared-table DiskANN without post-filter cliff |

Filter law still applies: bake-off measures **workspace-filtered** recall@20. Unfiltered demos are not a promote path.

## Env (harness / future opt-in)

| Env | Default | Meaning |
|-----|---------|---------|
| `EDGEQUAKE_FILTERED_DISKANN_LABELS` | off | Study/harness enable tip (not wired into product query path) |

## Honesty

- Smoke N archives directionality; **does not** raise floors.
- Do not silent-flip existing DBs to `labels smallint[]` or DiskANN+labels indexes.
- Wave-2 and dedicated DiskANN floors remain as in [`docs/product-limits.md`](../../docs/product-limits.md).
