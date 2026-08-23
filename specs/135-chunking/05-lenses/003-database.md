# Lens 003 — Database

## Stake

Migration 066 already added `chunks.page_start` / `page_end` and
`idx_chunks_page_span`. Live PDF ingest never binds them. SPEC-033 citation
and overlay queries that `SELECT` columns are lying.

## As-is

```ascii
  public.chunks
    page_start INT NULL
    page_end   INT NULL
    metadata   JSONB  ← writer puts page_* here
    token_count INT

  domain Chunk (SPEC-091)
    NO page_start / page_end fields
    INSERT cannot bind columns that do not exist on the struct
```

## Target

1. Extend `edgequake_storage::traits::domain::Chunk` with
   `page_start: Option<i32>`, `page_end: Option<i32>`.
2. `build_relational_chunks` copies `ChunkResult.page_*` onto those fields
   **and** keeps JSON metadata (lineage KV compat).
3. Postgres `insert_batch` binds the two columns.
4. Invariant: when the chunk was produced from a page-marked segment,
   both columns are `NOT NULL` and `page_end >= page_start`.
5. No backfill of historical rows in v1 (LAW-135-11 future ingestions).
   Old rows stay NULL until Rebuild KG.

## Queries the gate uses

```sql
-- E2E-135-01
SELECT page_start, page_end, count(*)
FROM chunks
WHERE document_id = $1
GROUP BY 1, 2
ORDER BY 1, 2;

-- Sanity: no NULL pages on page-marked ingest
SELECT count(*) FROM chunks
WHERE document_id = $1 AND page_start IS NULL;
-- gold: 0
```

## What not to do

- Do not drop JSON `metadata.page_*` in v1 (KV lineage still reads it).
- Do not require a new migration if 066 columns exist — **bind** them.
- Do not rewrite `idx_chunks_page_span`; it becomes useful once columns fill.

## Cross-refs

- LAW-135-9: [../01-first-principles.md](../01-first-principles.md)
- Hole: [../03-code-as-is.md](../03-code-as-is.md)
- E2E: [../08-test-protocol.md](../08-test-protocol.md)
