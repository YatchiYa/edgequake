# Lens 003 — Database Expert

## Stake

Page lineage already exists. Do not invent a second provenance store.

## As-is storage

| Artifact | Location |
|----------|----------|
| `page_start` / `page_end` | Chunk KV, vector metadata, Postgres chunk columns |
| Document title / file_name | Document metadata KV |
| Entity pages | **Must not** denormalize as primary (SPEC-047) |

## v1 decisions

- **No new tables.**
- Catalog is ephemeral per query (built from retrieved chunks + KV titles).
- Optional later: persist catalog JSON on conversation messages (pages already on `MessageSource`).
- Deleted document: href remains honest; viewer may 404 — do not rewrite to a wrong doc.

## Queries

Title resolve: existing `resolve_document_names` / metadata key scan — batch by unique `document_id`.

## Cross-refs

- SPEC-047 lineage: `../../047-rag-evaluation/021-lineage-first-principles-query.md`
- SPEC-033 data model: `../../033-page-lineage/02-data-model.md`
