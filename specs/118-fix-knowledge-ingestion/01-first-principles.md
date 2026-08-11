# 01 — First Principles

## Axioms

1. A durable relational document identity must satisfy the `documents(id)` UUID FK.
2. A retrieval/citation identity may be a **namespaced string** when product rules require exclusion (SPEC-0002 / SPEC-028).
3. These two identities can coexist if a single, explicit bridge maps between them.
4. Soft-skip of typed SSOT under product-default `relational` authority is a product failure, not a safe degradation.
5. Fail-closed remains correct for **unknown** non-UUID document ids.
6. One resolve function (DRY) must serve every typed writer that keys on `document_id`.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-118-1** | Relational FK identity for injections is the bare **injection UUID** (already `documents.id`) |
| **LAW-118-2** | Pipeline / graph / citation identity remains `injection::{workspace_id}::{injection_id}` |
| **LAW-118-3** | One SSOT resolver maps composite → UUID for typed writers (`chunks`, `chunk_embeddings`) |
| **LAW-118-4** | Unknown non-UUID ids stay fail-closed in the chunk writer; soft-skip only at sites that already soft-skip |
| **LAW-118-5** | Do not change `is_injection_source` prefix rules to paper over FK failures |
| **LAW-118-6** | Metadata must preserve the bridge (`legacy_document_id` / `legacy_chunk_key`) |
| **LAW-118-7** | At least one CI/e2e path must run injection under `CHUNK_TEXT_AUTHORITY=relational` |

## Causal diagram

```ascii
  content
     │
     ▼
  pipeline.process(doc_id=injection::ws::uuid)
     │
     ├─► extract + graph merge (composite source ids) ──► citations exclude injection::
     │
     └─► persist (authority=relational)
            │
            ▼
       resolve_relational_document_id(doc_id)
            │
            ├─ Ok(UUID) ─► insert chunks(document_id=UUID)
            │                 │
            │                 └─► typed embeddings load_for_document(UUID)
            │
            └─ Err ──────► task fail (today for injection::)
```

## Identity bridge (normative)

```ascii
  ┌─────────────────────────────────────────────────────────┐
  │  Composite (artifact / citation namespace)              │
  │  injection::{workspace_uuid}::{injection_uuid}          │
  └───────────────────────────┬─────────────────────────────┘
                              │ resolve trailing segment
                              ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Relational FK                                          │
  │  documents.id = injection_uuid                          │
  │  chunks.document_id = injection_uuid                    │
  └─────────────────────────────────────────────────────────┘
```
