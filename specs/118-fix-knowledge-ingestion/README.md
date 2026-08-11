# SPEC-118 — Fix Knowledge Injection under Relational Chunk Authority

> **Mission:** Restore `knowledge_injection` under default SPEC-091 `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational` by bridging composite `injection::{ws}::{id}` pipeline IDs to the injection UUID used by `public.documents` / `public.chunks`.  
> **Trigger:** [GitHub #376](https://github.com/raphaelmansuy/edgequake/issues/376) — worker hard-fails `Uuid::parse_str` on `injection::…` (len 85).

## Short verdict

| Layer | Identity after SPEC-118 |
|-------|-------------------------|
| Graph / legacy vectors / citation filter | Keep composite `injection::{ws}::{id}` (SPEC-0002) |
| `public.documents.id` / `chunks.document_id` / typed embeddings | Bare **injection UUID** (already upserted Wave B6) |
| Chunk metadata bridge | `legacy_document_id` + existing `legacy_chunk_key` |

**Do not skip-only** relational writes for injections — that leaves empty typed chunk SSOT under product default (SPEC-091 LD-01).

```ascii
  PUT /injection
       │
       ├─► documents.id = injection UUID          (typed upsert)
       ├─► pipeline doc_id = injection::ws::uuid  (KG + citations)
       └─► persist_relational_chunks
                 │
                 ▼
           resolve_relational_document_id()
                 │
                 ├─ bare UUID ──────────────► DocumentId
                 └─ injection::…::UUID ─────► DocumentId(trailing UUID)
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-118-1..7)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, AI, KG)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-reproduction
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| R1 | Local + unit reproduction | Done (`10-reproduction.md`) |
| I1 | Shared `resolve_relational_document_id` SSOT | Done (`document_id_resolve.rs`) |
| I2 | Wire `relational_chunk_writer` + `typed_embedding_writer` | Done |
| I3 | Metadata `legacy_document_id` bridge | Done |
| I4 | DELETE/PATCH typed-first meta (relational family) | Done (`load_injection_meta`) |
| T1 | Unit + Memory persist + PG worker e2e (relational) | Done (`e2e_spec118_injection_relational_pg`) |
| T2 | Live smoke on rebuilt :8090 (v0.24.3) | Done — completed + chunks + delete cascade |
| G1 | GitHub #376 comments | Done |

## Related

- [Issue #376](https://github.com/raphaelmansuy/edgequake/issues/376)
- SPEC-0002 knowledge injection (`specifications/0002_knowledge_injection_issue_131/`)
- SPEC-091 relational chunk SSOT (`specs/091-simplify-data-layer/`)
- SPEC-028 EC-07 citation exclusion of `injection::` sources

## Non-goals (v1)

- Redesign glossary UX / onboarding
- Change `injection_doc_id()` format or citation prefix rules
- Rewrite SPEC-091 authority model
- Acc / LightRAG dual-SUT pin changes
