# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Injection uses `injection::{ws}::{id}` doc IDs | SPEC-0002 ADR; `injection_process::injection_doc_id` |
| Citations must exclude injection sources | SPEC-028 EC-07; `source_reference_builder::is_injection_source` |
| Chunk text authority defaults to relational | SPEC-091 Wave D; `chunk_text_authority.rs` |
| Typed chunk rows require UUID `document_id` | SPEC-091 LD-01/LD-02; `relational_chunk_writer::parse_document_id` |
| Injection metadata row id is injection UUID | SPEC-091 Wave B6; `injection_relational.rs` |
| Production failure under relational authority | [GitHub #376](https://github.com/raphaelmansuy/edgequake/issues/376) |
| Dual identity (map, don't skip-only) | SPEC-118 LAW-118-1..3 |

## Code SSOT (target)

| Concern | Path |
|---------|------|
| Composite doc id builder | `edgequake-api/src/services/injection_process.rs` (`injection_doc_id`) |
| Worker → pipeline | `edgequake-api/src/processor/injection_processing.rs` |
| Typed injection upsert | `edgequake-api/src/services/injection_relational.rs` |
| Authority gate | `edgequake-pipeline/src/persistence/ingestion_persister.rs` |
| **Resolver SSOT** | `edgequake-pipeline/src/persistence/document_id_resolve.rs` (new) |
| Relational chunk write | `edgequake-pipeline/src/persistence/relational_chunk_writer.rs` |
| Typed embedding write | `edgequake-pipeline/src/persistence/typed_embedding_writer.rs` |
| Citation filter | `edgequake-api/src/services/source_reference_builder.rs` |
| Soft-skip contrast | `edgequake-api/src/services/document_stage_mirror.rs` |

## DRY rule

All typed writers that need a UUID `DocumentId` from a pipeline `document_id` string **must** call `resolve_relational_document_id` (or a thin wrapper). No second copy of `injection::` parsing in API handlers, workers, or SQL.

## Related specs

| Spec | Relationship |
|------|--------------|
| SPEC-0002 | Defines injection product + composite id |
| SPEC-091 | Defines relational chunk SSOT + UUID spine |
| SPEC-028 | Citation exclusion of injection sources |
| SPEC-058 | Vector table rebuild guard (local repro blocker on dim mismatch — out of scope) |
| SPEC-117 | Doc-pack template precedent (00–09 + lenses) |
