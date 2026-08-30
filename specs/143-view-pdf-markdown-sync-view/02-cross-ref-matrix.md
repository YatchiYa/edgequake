# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Marker grammar | SPEC-083 X-13; `PageMarkerWriter` (pdf + pipeline) |
| Chunk page span | SPEC-135; `page_start` / `page_end` |
| Deeplink schema | SPEC-033; `document-url.ts` `?page=` |
| Citation → page | SPEC-142; verified href uses `page_start` |
| Layout overlay | SPEC-128; `GET …/pages/{n}/layout` |
| Panel sync claim | FEAT0733 (as-is: layout only → target: page sync) |
| Controlled PDF page | SPEC-033; `PDFViewer.currentPage` |

## Code SSOT (as-is → target)

| Concern | As-is | Target |
|---------|-------|--------|
| Marker parse (FE) | Asset binding only (`documents.ts`) | Shared `page-markers.ts` + DOM inject |
| PDF pages | Single `<Page>` | Continuous stack + windowed render |
| Page outbound | None | `onPageChange` |
| Sync | Layout mode toggle | `usePageSyncController` + FEAT0733 toggle |
| MD anchors | Comments stripped | `data-eq-page` / `eq-md-page-N` |
| URL | Inbound `?page=` only | Bidirectional debounce write |
| Overlay | Current single page | Active sheet only |

## Related specs

| Spec | Relationship |
|------|--------------|
| SPEC-033 | Deeplink + controlled page inbound |
| SPEC-083 X-13 | Marker SSOT — do not fork grammar |
| SPEC-128 | Overlay stays; bind to active page |
| SPEC-135 | Cross-page chunk → navigate `page_start` |
| SPEC-142 | Opens viewer at page; this pack keeps panes aligned while reading |
| SPEC-038 | Large PDF admission — windowed render required |

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
