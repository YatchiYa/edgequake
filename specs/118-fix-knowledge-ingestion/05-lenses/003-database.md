# Lens 003 — Database Expert

## Schema facts

```ascii
  public.documents (id UUID PK)
         ▲
         │ FK document_id
  public.chunks (id UUID PK, document_id UUID NOT NULL, ...)
         ▲
         │ FK chunk_id
  public.chunk_embeddings (...)
```

Injection Wave B6 already inserts `documents.id = injection_id` with `metadata.source_type = 'injection'`.

## Mapping implication

| Pipeline id | Relational `document_id` |
|-------------|--------------------------|
| bare UUID | same UUID |
| `injection::{ws}::{inj}` | `inj` UUID |
| other non-UUID | reject (chunk writer) |

`PostgresChunkRepository::insert_batch` → `ensure_document_parents` (`ON CONFLICT DO NOTHING`) remains safe if upsert raced.

## Cascade / delete

```ascii
  DELETE documents WHERE id = injection_uuid
       └─► chunks CASCADE
            └─► chunk_embeddings CASCADE
  Graph cleanup still keys on composite doc_id (KV/graph), not FK
```

## No migration

SPEC-118 v1 requires **no** new SQL migration. Optional future: backfill historical injection chunks if any were written under wrong keys (none expected — writer hard-failed).

## Observability SQL

```sql
-- After successful injection
SELECT id, status, metadata->>'source_document_id'
FROM documents WHERE id = :injection_id;

SELECT count(*) FROM chunks WHERE document_id = :injection_id;
```
