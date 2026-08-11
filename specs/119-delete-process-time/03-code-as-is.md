# 03 — Code As-Is

## Call chain

```ascii
  DELETE /api/v1/documents/:id
       → delete_document (202)
       → TaskType::Deletion
       → perform_document_deletion
       → cascade_remove_document_sources*
       → find_document_edges
       → GraphScanOps::find_edges_by_source_prefixes
       → pg_find_edges_by_source_prefixes
              ├─ modern_sql   (GIN source_ids)
              ├─ legacy_sql?  (optional)
              └─ singular_sql (Symptom F)  ← GH-375 timeout

  POST /api/v1/documents/reprocess
       → retract_document_indexes
       → cascade_remove_document_sources
       → (same find_edges_by_source_prefixes)
```

## Singular SQL (broken expression)

File: `edgequake/crates/edgequake-storage/src/adapters/postgres/graph/scan_ops.rs`

```ascii
  WHERE ...
    AND (
      ({props})::jsonb->>'source_chunk_id' IN (probes)
      OR ({props})::jsonb->>'source_document_id' IN (probes)
    )
  LIMIT 5000

  props = ag_catalog.agtype_to_json(e.properties)
```

Error on failure: `"Source-prefix singular edge query failed: …"`.

## Indexes present today (child `"EDGE"`)

| Index | Serves |
|-------|--------|
| `idx_edge_source_id` / `idx_edge_target_id` | Topology endpoint props |
| `idx_edge_source_ids_gin` | Modern `@> source_ids` |
| `idx_edge_source_chunk_ids_gin` | Plural `source_chunk_ids` GIN |
| `idx_edge_workspace_id` / `idx_edge_tenant_id` | Tenant filters |
| **missing** | Singular `source_chunk_id` / `source_document_id` |

## Timeout wrapper

```ascii
  LocalTimeoutTx::begin(conn, SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS=2000)
       → SET LOCAL statement_timeout = '2000ms'
       → modern + legacy + singular in one tx
       → commit / rollback
```

## Proven expression trap (local)

```ascii
  WHERE agtype_to_json(properties)->>'source_id' = 'x'
       → Index Scan using idx_edge_source_id

  WHERE (agtype_to_json(properties))::jsonb->>'source_id' = 'x'
       → Seq Scan   (identical human intent, different expression)
```

Singular probe uses the second shape today → even a future btree would be ignored until the cast is removed.
