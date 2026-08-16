# Lens 004 — UX / UI Designer

## Stake

The PDF pane must answer: **what did we recognize, and where?** Overlay is opt-in, layered, and honest. It must not fight reading (text layer, zoom, page turn).

## Job

Toggle **Layout overlay**. See color-coded boxes for figures, charts, tables, paragraphs, columns, noise. Filter layers. Click a figure box → markdown pane scrolls to that asset if linked.

## Principles

- Overlay **off** by default (page remains a document, not a HUD).
- Chips are **filters**, not modes that replace the PDF.
- Noise (logos/stamps) visible only when the Noise chip is on — default **off** so the first overlay view is “content”, not clutter.
- Empty: toggle disabled + tooltip “Layout not available for this page”.
- Zoom: boxes stay glued (measured CSS box, not assumed scale).
- Color is not the only signal: label + legend.
- Keyboard: `O` toggle overlay; chips reachable.

## Hierarchy

```ascii
  Toolbar (existing nav + zoom)
    │  + Layout overlay (icon + pressed state)
    │  + chips appear when overlay ON
  Page canvas
    │  boxes 2px stroke, 12% fill, label chip top-left of box
  Legend (compact, below toolbar or in chips row)
```

## Interaction with SPEC-033

Chunk click already sets `?page=N` and markdown highlight. Overlay **follows `currentPage`**. Do not jump the PDF when clicking a box on the current page except to scroll markdown.

## Anti-patterns

- Drawing boxes on markdown images instead of the PDF.
- Auto-enabling overlay on every document (noisy).
- Using list-dialog `PDFMarkdownSplitView` as the design surface.

## Cross-refs

- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front: [005-front-designer.md](005-front-designer.md)
- SPEC-033: [../../033-page-lineage/04-ux-ui-spec.md](../../033-page-lineage/04-ux-ui-spec.md)
