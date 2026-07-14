# SPEC-050 — Database Expert Lens

## Cascade Delete Correctness

### Delete Order (must be preserved)

```
1. Cancel in-flight task
   WHY: Stops processor from writing new KV/graph rows after we start deleting.

2. Delete vector embeddings (pgvector table)
   WHY: Chunk IDs are the FK. Delete embeddings before KV chunks to avoid FK violations
   if referential integrity is ever added.

3. Graph cascade (AGE + KG tables)
   WHY: Entity/edge rows reference source_ids. Delete graph after vectors but before
   KV because the graph cascade may need to read chunk metadata.

4. Delete KV rows (document_kv table)
   Keys: {id}-chunk-*, {id}-content, {id}-metadata, workspace_doc_index key
   WHY: KV is the source of truth for list/detail endpoints. This must be last
   so that list queries don't return partially-deleted documents.

5. Delete relational rows (documents, pdf_documents tables)
   WHY: These are the read-model tables. Delete last so read queries still work
   during graph/KV cleanup (consistent reads during delete).

6. Delete content hash key (duplicate detection)
   WHY: Allow re-upload of same content after deletion.
```

### Transaction Boundaries

```
┌─────────────────────────────────────────────────────────────────────┐
│ Step 1: Cancel task       — async, best-effort, no transaction      │
│ Step 2: Delete vectors    — single DELETE WHERE id IN (...)         │
│         ↑ NOT in a transaction — pgvector deletes are idempotent    │
│ Step 3: Graph cascade     — series of AGE Cypher + SQL DML          │
│         ↑ Each entity update is an upsert — idempotent              │
│ Step 4: Delete KV rows    — single batch DELETE                     │
│         ↑ In a single DB statement where possible                   │
│ Step 5: Delete relational — DELETE FROM documents WHERE id = $1     │
│         ↑ In a transaction with KV delete for consistency            │
└─────────────────────────────────────────────────────────────────────┘
```

### Index Coverage

| Query in delete path                                        | Required index                                             | Status  |
| ----------------------------------------------------------- | ---------------------------------------------------------- | ------- |
| `keys_with_prefix("{id}-chunk-")`                           | `kv_storage(key text)` prefix scan                         | EXISTS  |
| `keys_with_suffix("-metadata")`                             | `kv_storage(key text)` — needs GIN trigram or suffix index | PARTIAL |
| `SELECT * FROM documents WHERE id = $1`                     | `documents(id)` PK                                         | EXISTS  |
| `SELECT * FROM entity_sources WHERE document_id = $1`       | `entity_sources(document_id)`                              | CHECK   |
| `SELECT * FROM relationship_sources WHERE document_id = $1` | `relationship_sources(document_id)`                        | CHECK   |
| Vector delete `WHERE id = ANY($1)`                          | `vector_storage(id)`                                       | EXISTS  |

### Bulk Delete N+1 Risk

Current bulk delete iterates documents one by one:
```rust
for (metadata_key, metadata) in scoped_entries {
    // one cascade per document — this is O(N) sequential DB round trips
}
```

Improvement (future): batch vector deletes for all docs in workspace in a single statement:
```sql
DELETE FROM vector_storage
WHERE id = ANY(
  SELECT key FROM document_kv WHERE key LIKE '%-chunk-%'
  AND workspace_id = $1
)
```

For now, sequential is acceptable. Add a `tracing::info!(progress = N/total)` per document for visibility.

### Partial Failure Handling

If graph cascade fails, KV must still be deleted. The existing code already implements this:
```rust
// non-fatal graph cascade — proceed with KV cleanup
let cascade_stats = match cascade_remove_document_sources(...).await {
    Ok(stats) => stats,
    Err(e) => {
        tracing::error!("Graph cascade failed (non-fatal)");
        CascadeStats::default()  // ← continue with zeros
    }
};
```

But `partial_failure` is not returned to the client. Add to response:
```rust
pub struct DeleteDocumentResponse {
    // ... existing fields ...
    pub embeddings_deleted: usize,    // NEW
    pub partial_failure: bool,        // NEW
    pub partial_failure_reason: Option<String>,  // NEW
}
```

### Content Hash Cleanup (OADA-90)

When a document has a `content_hash` in its metadata, the duplicate-detection key must also be deleted:
```
key: "content_hash:{sha256_hex}"
```
This is already implemented. Verify it's in the broadcast data.
