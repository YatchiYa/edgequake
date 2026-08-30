# 03 — Code as-is

## Broken sync claim

```ascii
  <!-- edgequake-page:N --> in stored markdown
           │
           ├─► PageAwareChunking → page_start / page_end (KV + vector)
           ├─► FE regex → MM figure asset binding only
           └─► Markdown render → HTML comments gone → NO DOM anchors

  ?page= / chunk page_start
           │
           ▼
  PDFViewer.currentPage ──► single <Page pageNumber={n} />
           │
           ├─ toolbar </ > updates local pageNumber (no onPageChange)
           ├─ wheel scrolls canvas inside overflow-y-auto
           └─ never scrolls Markdown

  SideBySideViewer FEAT0733
           │
           └─ view mode only (pdf / md / side-by-side) — not page sync
```

## Key paths

| Layer | Path | Behavior today |
|-------|------|----------------|
| Marker emit | `edgequake-pdf/src/page_marker.rs` | `<!-- edgequake-page:N -->` |
| Marker parse | `edgequake-pipeline/.../page_marker.rs` | Chunk attribution |
| FE marker | `documents.ts` / `document-assets.ts` | Asset binding only |
| PDF UI | `pdf-viewer.tsx` | Single page; no `onPageChange` |
| Layout | `side-by-side-viewer.tsx` | Resize + mode; no sync |
| Detail page | `documents/[id]/page.tsx` | Inbound `activePdfPage` |
| MD render | `content-renderer.tsx` | Highlight scroll; no page anchors |
| Overlay | `pdf-page-overlay.tsx` | Click → `focusMarkdownAsset` |

## Gaps

1. No shared `activePage` controller.
2. No DOM page anchors from markers.
3. No continuous PDF stack → wheel ≠ page nav.
4. Controlled-mode drift: local prev/next vs `?page=`.
5. Dual split shells (`SideBySideViewer` vs `PDFMarkdownSplitView`) unevenly wired.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- WHY: [00-why.md](00-why.md)
