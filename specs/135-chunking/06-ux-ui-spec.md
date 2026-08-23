# 06 — UX / UI Spec

Additive copy and citation badge. No new settings panel. Kill switches stay
env-only.

## Workspace Chunking card

Existing card (SPEC-116 / SPEC-125). Additive copy only.

| Element | Copy | Testid |
|---------|------|--------|
| Markdown pack hint | (existing SPEC-125) | `chunking-markdown-pack-hint` |
| PDF pack hint | PDF conversions pack headings, figures, and short pages into the token budget so extract is not one job per page. | `chunking-pdf-pack-hint` |
| Future-only | Applies to future document ingestions. Use Rebuild Knowledge Graph to re-chunk existing documents. | `chunking-future-only-hint` |

## Lineage / hierarchy tree

File: `edgequake_webui/src/components/document/document-hierarchy-tree.tsx`

Today: badge `p.{page_start}` and comment “Always equals page_start.”

Target:

```ascii
  page_start && page_end > page_start  →  "p.{start}–{end}"
  else if page_start                   →  "p.{start}"
  else                                 →  (no badge)
```

| Element | Behavior | Testid |
|---------|----------|--------|
| Page badge | Span or single page as above | `chunk-page-badge` |
| `aria-label` | `Pages {start} to {end}` or `Page {start}` | |
| `title` | `Open PDF at page {start}` | |
| Click / deeplink | `buildDocumentPageUrl(..., page_start)` — **never** `page_end` | |

`E2E-135-UI` gold: visible text `p.1–2` on the span fixture chunk.

## PDF viewer

Unchanged mechanism: `#page={page_start}` (1-indexed). A span chunk opens
the **first** page of the span. No multi-page highlight in v1.

## Query / chat citations

If the UI renders `SourceReference.page_start` / `page_end`, use the same
badge rule. OpenAPI copy: `page_end` **may** exceed `page_start`.

## Fill (ops, not a user control)

No fill gauge on the documents table in v1. Operators diagnose via Langfuse
`ingest.chunking` → `fill_p50`. A future badge on the document row is
out of scope (do not block 135).

## Kill switch

Not in UI. Document next to existing chunking env in `.env.example`:

```
# SPEC-135 — PDF pack-to-budget (default ON)
# EDGEQUAKE_PDF_PACK=0
# EDGEQUAKE_PDF_CROSS_PAGE_PACK=0
```

## Cross-refs

- UX lens: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md)
- Playwright: [08-test-protocol.md](08-test-protocol.md)
- SPEC-033 amendment: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
