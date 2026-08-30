# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Page is a chunk property | SPEC-047 L1/L5; `RetrievedChunk.page_start` |
| Deeplink schema | SPEC-033 FR-011; `document-url.ts` |
| Stable `[N]` | SPEC-083 X-20; `assign_stable_citation_ids` |
| Acc gold forbids `[N]` | SPEC-082; `strip_gold_citation_artifacts` |
| Cross-page pack | SPEC-135; deeplink uses `page_start` |
| Sources side-channel | `source_reference_builder.rs`; `resolve_chunk_file_paths` |
| Prompt cite mandate | `grounding.rs` `grounding_instructions` |
| Query / MCP DTO | SPEC-028 |

## Code SSOT (as-is → target)

| Concern | As-is | Target |
|---------|-------|--------|
| Citation policy | `grounding.rs` | Same + forbid locators; few-shot |
| Chunk headers | `context_format.rs` (`page=`, optional UUID `doc=`) | Always `doc="{title}"` when known |
| Catalog | Implicit in `sources[]` | `CitationCatalog` in query crate |
| Rewrite | None | `citation_verify::rewrite_verified_citations` |
| Sync query | `query_execute.rs` raw answer | Rewrite after generate |
| Stream | Tokens then Done | Context(catalog) + Done(verified) |
| Chat | Persist raw | Persist rewritten |
| MCP | Tool answer raw | Same rewrite |
| UI mapper | `extractDocumentId(s.id)` | Prefer `s.document_id` |
| UI answer links | `target=_blank` | Next.js `/documents/` nav |
| URL builder | `document-url.ts` | Mirror rules in Rust |

## Related specs / issues

| Spec | Relationship |
|------|--------------|
| SPEC-033 | Panel + viewer; this pack closes **inline** |
| SPEC-047 L-B1 | Answer-inline half of lineage citations |
| SPEC-082 / 001 Acc | Gold freeze — rewriter must skip |
| SPEC-135 | Span badge vs href start |
| SPEC-028 | Surfaces + MCP |
| SPEC-083 X-20 | Stable ids |

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
