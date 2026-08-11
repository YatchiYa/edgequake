# 03 — Code As-Is

## Call chain

```ascii
  Document merge (pipeline)
       │
       ▼
  merger/mod.rs  upsert_vectors_chunked
       │
       ▼
  FleetEmbeddingIndex::mirror_legacy_batch
       │  load EntityNameIndex per workspace (unordered SELECT)
       │  resolve name → FK UUID
       │  build FleetEmbeddingRow { legacy_vector_id: Some(id) }
       ▼
  upsert_batch(family, …)
       │
       ▼
  INSERT … ON CONFLICT (model_id, fk) DO UPDATE
         SET legacy_vector_id = COALESCE(existing, EXCLUDED)
       │
       ├─ PK hit → stamp-once OK
       └─ legacy unique hit, different FK → 23505 → StorageError
              │
              ▼
         record_error → GraphMerge → compensate_merge_failure
```

## Failing expression (entity family)

```sql
INSERT INTO entity_embeddings
  (model_id, entity_id, workspace_id, embedding, dimensions, legacy_vector_id)
SELECT $1, e, w, v::halfvec, d, NULLIF(lid, '')
FROM unnest(...) AS t(e, w, v, d, lid)
ON CONFLICT (model_id, entity_id) DO UPDATE
  SET legacy_vector_id = COALESCE(
        entity_embeddings.legacy_vector_id, EXCLUDED.legacy_vector_id);
-- ↑ does NOT arbiter idx_entity_embeddings_legacy_vector_id
```

Identical pattern for `relationship_embeddings` and `report_embeddings`.

## Active uniqueness

| Object | Constraint |
|--------|------------|
| `entities` | `UNIQUE (tenant_id, workspace_id, name)` + live ON CONFLICT |
| `relationships` | `UNIQUE (tenant_id, workspace_id, source_id, target_id, relation_type)` |
| `entity_embeddings` | PK `(model_id, entity_id)`; partial UNIQUE `(workspace_id, legacy_vector_id)` |
| `relationship_embeddings` | PK `(model_id, relationship_id)`; same partial UNIQUE |
| `report_embeddings` | PK `(model_id, report_id)`; same partial UNIQUE |

## Wrong fixes

| Temptation | Why wrong |
|------------|-----------|
| Second `ON CONFLICT … DO UPDATE` in one INSERT | Postgres forbids multiple conflict targets |
| Drop legacy unique index | Breaks provenance invariant (LAW-120-2); regresses 143/144 |
| Only catch 23505 and retry whole batch blindly | Can loop; loses stamp-once clarity |
| Raise ingest concurrency as “solved” | Hides race; increases silent alias debt |
| Treat issue as “entity UNIQUE missing” | Exact-name path already race-safe (LAW-120-4) |
