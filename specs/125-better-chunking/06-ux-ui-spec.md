# 06 — UX / UI Spec

## Workspace Chunking card

Existing card (SPEC-116). Additive copy only.

| Element | Copy | Testid |
|---------|------|--------|
| Description | How documents are split into chunks before entity extraction. | (existing) |
| Markdown pack hint | Markdown files pack small headings into the token budget so a heading is not its own chunk. | `chunking-markdown-pack-hint` |
| Future-only | Applies to future document ingestions. Use Rebuild Knowledge Graph to re-chunk existing documents. | `chunking-future-only-hint` |

## Lineage

No new control. After heading-dense ingest, first chunk content must include more than the parent heading (Playwright optional; e2e API sufficient).

## Kill switch

Not in UI. Document in `.env.example` next to `EDGEQUAKE_CHUNK_SIZE`.

## Cross-refs

- Front: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Playwright: [08-test-protocol.md](08-test-protocol.md)
