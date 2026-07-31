# RM4 — Hot-path EXPLAIN artifacts (SPEC-091)

**Date:** 2026-07-31  
**Scope:** Plan shapes for the ten highest-traffic refs. Full `EXPLAIN (ANALYZE, BUFFERS)` on production-scale data remains soak-only (100k+ vectors); CI proves contracts via `e2e_spec091_recall_parity` / fleet recall fixtures.

## Method

Expected index usage under PG16 and PG18 (unified SQL; capability-gated iterative_scan / uuidv7 only):

| Ref ID | Expected plan shape | Index / gate |
| --- | --- | --- |
| `DATA-PGVEC-VECTORS-ANN-QUERY-001` | HNSW Index Scan on `chunk_embeddings` | typed HNSW ef_construction=128 (mig 129) |
| `DATA-PGVEC-VECTORS-ANN-QUERY-FILTERED-002` | HNSW + `hnsw.iterative_scan=relaxed_order` | pgvector ≥0.8; workspace filter |
| `DATA-PG-VECTORS-TEXT-SEARCH-FILTERED-003` | Bitmap Index Scan on `idx_chunks_content_tsv` | mig 136 GIN |
| `DATA-LIST-DOCUMENTS-WORKSPACE-001` | Index Scan Backward on `(workspace_id, created_at DESC)` | mig 128 |
| `DATA-FENCE-JOIN-READY-001` | Nested Loop / Hash Join `chunks` ⋈ `chunk_serving_state` | PK on chunk_id; fence default on |
| `DATA-AGE-GRAPH-GET-NEIGHBORS-042` | Index Scan on `"EDGE"(start_id)` / `(end_id)` | ensure_indexes |
| `DATA-AGE-GRAPH-SEARCH-NODES-041` | Bitmap/GIN on Node properties / source_chunk_ids | RM3 GIN |
| `DATA-AGE-GRAPH-UPSERT-NODES-BATCH-046` | Insert + UNIQUE node_id | mig 074/083 |
| `DATA-OUTBOX-DRAIN-CLAIM-001` | Update … FOR UPDATE SKIP LOCKED on partial unprocessed | mig 134 |
| `DATA-WIPE-CHUNKS-WORKSPACE-001` | Delete using documents workspace predicate | RM1 set-based |

## CI fixture vs soak

| Ladder | Gate | Status |
| --- | --- | --- |
| Small CI fixture | `e2e_spec091_recall_parity`, `e2e_spec091_fleet_recall_parity` in `make spec091-gates` | Wired |
| 100k+ filtered recall@k | Soak / nightly only | **Deferred** with date 2026-07-31 (RM-AC-12 honesty) |

## Capture commands (operator)

```bash
# Example — filtered ANN (session GUCs)
SET hnsw.iterative_scan = 'relaxed_order';
EXPLAIN (ANALYZE, BUFFERS)
SELECT chunk_id FROM chunk_embeddings
WHERE workspace_id = $1
ORDER BY embedding <=> $2 LIMIT 10;

# Example — chunk FTS
EXPLAIN (ANALYZE, BUFFERS)
SELECT c.id FROM chunks c
JOIN documents d ON d.id = c.document_id
WHERE c.content_tsv @@ plainto_tsquery('english', 'token')
  AND d.workspace_id = $1
LIMIT 10;
```

Record outputs next to this file as `rm4-explain-<ref>-pg16.txt` / `-pg18.txt` when measured.
