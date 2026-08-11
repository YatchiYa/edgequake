# 03 — Code As-Is

## Failure path (ASCII)

```ascii
  PUT /api/v1/workspaces/{workspace_id}/injection
       │
       │  crud.rs: workspace_id_from_tenant (path often ignored)
       │  injection_id = Uuid::new_v4()
       │  doc_id = injection_doc_id(ws, injection_id)
       │         = "injection::{ws}::{injection_id}"
       │
       ├─► KV meta upsert (injection::{ws}::{id}-metadata)
       ├─► typed_injection_upsert → documents.id = injection_id (UUID) ✅
       └─► enqueue KnowledgeInjectionData { doc_id, injection_id, ... }
                │
                ▼
       injection_processing::process_knowledge_injection
                │
                ├─ resolve_relational_chunk_repo(pg_pool) → Some(repo)
                └─ run_injection_pipeline(..., doc_id=composite, ...)
                        │
                        ├─ pipeline.process_with_resilience(composite)
                        ├─ tag_injection_sources(..., composite)
                        └─ persist_with_providers(... relational_chunks=Some)
                                │
                                ▼
                          if chunk_text_authority_writes_relational:
                            persist_relational_chunks(ctx.document_id=composite)
                                │
                                ▼
                            parse_document_id(composite)
                                │
                                ▼
                            Uuid::parse_str("injection::…::…") ❌
                                │
                                ▼
                            StorageError::InvalidData → task fail ×3
```

## Hard-fail site

```93:99:edgequake/crates/edgequake-pipeline/src/persistence/relational_chunk_writer.rs
fn parse_document_id(raw: &str) -> Result<DocumentId, StorageError> {
    parse_uuid(raw).map(DocumentId)
}

fn parse_uuid(raw: &str) -> Result<Uuid, StorageError> {
    Uuid::parse_str(raw)
        .map_err(|e| StorageError::InvalidData(format!("invalid uuid '{raw}': {e}")))
}
```

## Soft-skip contrast (embeddings)

```34:37:edgequake/crates/edgequake-pipeline/src/persistence/typed_embedding_writer.rs
    let doc_uuid = match Uuid::parse_str(&ctx.document_id) {
        Ok(u) => u,
        Err(_) => return Ok(0),
    };
```

Even after a skip-only chunk hotfix, embeddings would still write **zero** typed rows for injections until the same bridge is applied.

## Dual write already present (Wave B6)

```7:8:edgequake/crates/edgequake-api/src/services/injection_relational.rs
//! title/content/status promoted to columns. The row id IS the injection id
//! (already a UUIDv4 at creation).
```

Parent FK row exists before persist; mapping chunks to injection UUID satisfies `chunks.document_id → documents(id)`.

## CI blind spot

| Test | Why it misses #376 |
|------|--------------------|
| `e2e_injection.rs` | `AppState::test_state()` → `pg_pool: None` |
| Worker harness | Forces `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=kv` |
| `contract_spec091_build_relational_chunks_rejects_bad_document_id` | Asserts fail-closed for `"not-a-uuid"` — reinforces hard-fail, no injection case |

## Local live observation (2026-08-11)

See `10-reproduction.md`. Default workspace hit SPEC-058 dim mismatch (768 vs 1024) **before** the UUID parser; typed `documents` row still created with `source_document_id = injection::…`. Code path and error shape for #376 remain confirmed by unit parse semantics (composite length 85).
