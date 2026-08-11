# 04 — Target Architecture

## Decision

**Dual-identity bridge** (LAW-118-1..3):

1. Keep composite pipeline `doc_id` for graph + citations.
2. Resolve trailing injection UUID for relational FK writers.
3. Store bridge fields in chunk metadata.

Skip-only is rejected for product default `relational` authority.

## Shared resolver (SOLID / DRY)

New module: `edgequake-pipeline/src/persistence/document_id_resolve.rs`

```rust
/// Resolve a pipeline document_id string into a relational DocumentId.
/// - bare UUID → DocumentId
/// - `injection::{ws}::{uuid}` → DocumentId(trailing uuid)
/// - else → StorageError::InvalidData
pub fn resolve_relational_document_id(raw: &str) -> Result<DocumentId, StorageError>;

/// True when `raw` used the injection:: bridge (for metadata tagging).
pub fn is_injection_composite_document_id(raw: &str) -> bool;
```

### Call sites (only)

| Site | Behavior |
|------|----------|
| `relational_chunk_writer::parse_document_id` | Use resolver; on injection bridge set `metadata.legacy_document_id` |
| `typed_embedding_writer::persist_typed_chunk_embeddings` | Use resolver instead of bare `Uuid::parse_str`; unknown non-UUID may remain soft `Ok(0)` |

### Explicitly unchanged

| Site | Reason |
|------|--------|
| `injection_doc_id` / `tag_injection_sources` | Citation + graph contract |
| `is_injection_source` | Prefix exclusion |
| `document_stage_mirror` soft-skip | Unrelated non-UUID stage docs |

## Metadata bridge

When mapping an injection composite:

```json
{
  "legacy_chunk_key": "injection::{ws}::{id}-chunk-0",
  "legacy_document_id": "injection::{ws}::{id}",
  "start_line": 1,
  "end_line": 1
}
```

## Delete / cascade

```ascii
  DELETE /injections/{injection_id}
       │
       ├─ typed_injection_delete(injection_uuid)
       │       └─ documents DELETE → chunks ON DELETE CASCADE ✅
       └─ cleanup_document_graph_data(composite doc_id) ✅
```

No schema migration required.

## Authority matrix (target)

| Authority | Injection chunks in `public.chunks` | Task result |
|-----------|-------------------------------------|-------------|
| `relational` | Yes (mapped UUID) | completed |
| `dual` | Yes | completed |
| `kv` | Optional / legacy | completed (regression) |

## Anti-patterns

```ascii
  ❌ Change injection_doc_id to bare UUID
       → breaks is_injection_source / citation exclusion

  ❌ Skip persist_relational_chunks for injection::
       → empty typed SSOT under product default

  ❌ Duplicate injection:: parsing in API + pipeline
       → DRY violation; drift risk

  ❌ Soft-skip unknown garbage in chunk writer
       → hides corrupt ingest
```
