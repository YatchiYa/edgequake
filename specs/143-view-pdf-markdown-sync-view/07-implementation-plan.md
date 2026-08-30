# 07 — Implementation Plan

## Phase A — Spec pack (this directory)

Done when README + laws + lenses + matrices land.

## Phase B1 — Marker util

1. Add `edgequake_webui/src/lib/utils/page-markers.ts`.
2. `parsePageMarker`, `listPageMarkers`, `injectPageAnchors`, `hasPageMarkers`.
3. Extend DOMPurify allowlist for `data-eq-page`.
4. Unit tests.

## Phase B2 — Controller

1. `usePageSyncController({ initialPage, settleMs })`.
2. API: `activePage`, `syncEnabled`, `setPageFromPdf|Md|Url`, `toggleSync`, `scrollRequest`.
3. Parent owns URL debounce write.

## Phase B3 — PDF continuous stack

1. Refactor `pdf-viewer.tsx` to stack pages.
2. IO → `onPageChange`.
3. `scrollToPage`, keyboard, windowing >20.
4. Overlay on active sheet only.

## Phase B4 — Markdown anchors

1. Inject in ContentRenderer / MarkdownViewer after asset rewrite.
2. `useMarkdownPageObserver`.
3. Sticky page indicator.

## Phase B5 — Wire FEAT0733

1. Sync toggle on SideBySideViewer.
2. Wire `documents/[id]/page.tsx`.
3. Align dialog + PDFMarkdownSplitView.

## Phase C — E2E

See [08-e2e-test-matrix.md](08-e2e-test-matrix.md).

## Order

```ascii
  A docs → B1 utils → B2 controller → B3 PDF → B4 MD → B5 wire → C e2e
                │           │
                └─ unit ────┘
```

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Edge cases: [09-edge-cases.md](09-edge-cases.md)
