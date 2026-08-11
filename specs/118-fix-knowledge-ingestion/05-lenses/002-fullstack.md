# Lens 002 — Full Stack Developer

## Touch list

| Layer            | Files                                                        |
| ------------------| --------------------------------------------------------------|
| Resolve SSOT     | `edgequake-pipeline/.../document_id_resolve.rs` (new)        |
| Chunk writer     | `relational_chunk_writer.rs`                                 |
| Embedding writer | `typed_embedding_writer.rs`                                  |
| Persistence mod  | `persistence/mod.rs` exports                                 |
| Tests            | writer unit contracts; PG injection e2e under relational     |
| Unchanged        | `injection_doc_id`, `tag_injection_sources`, citation filter |

## SOLID / DRY

- **S** — Resolver only maps identities; writers only persist
- **O** — New injection pattern handled without changing bare-UUID path
- **L** — `DocumentId` semantics unchanged for callers
- **I** — No new ports; reuse `ChunkRepository` / `EmbeddingIndex`
- **D** — Writers depend on resolver function, not injection CRUD
- **DRY** — One parse implementation shared by chunk + embedding writers

## Anti-patterns

- Copy-paste `starts_with("injection::")` into multiple crates
- Changing OpenAPI request shapes for a persistence bug
- Pinning all tests to `kv` authority forever

## Implementation notes

1. Prefer pure function + unit tests in pipeline crate (fast).
2. Wire both writers in the same PR.
3. Add `legacy_document_id` only when bridge applied (avoid noisy metadata on normal docs).
