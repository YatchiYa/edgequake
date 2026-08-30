# Lens — PDF Viewer Expert

## Current stack

- `react-pdf` ^10 + `pdfjs-dist` 5.4.x
- Same-origin worker `/pdf.worker.min.mjs`
- Dynamic import, `ssr: false`

## Target render model

```ascii
  overflow-y-auto scroll container
    ├─ sheet 1  <Page pageNumber={1} />   data-testid=pdf-page-sheet
    ├─ sheet 2  <Page … />   (or placeholder height if windowed)
    ├─ …
    └─ sheet N
```

### Why continuous stack (not edge-turn)

react-pdf does not provide native “scroll to next page” on a single Page.
Community practice ([issue #930](https://github.com/wojtekmaj/react-pdf/issues/930)):
map all pages into a scrollable Document. Active page = max intersection ratio.

### Windowing

When `numPages > 20`, render real `<Page>` only for `active ± 2`; offscreen
sheets use fixed-height placeholders (estimate from first rendered page height
× scale) to keep scroll metrics stable.

### Navigation APIs

| Input | Behavior |
|-------|----------|
| Wheel / trackpad | Native scroll of stack |
| Toolbar prev/next | `scrollToPage` |
| Keyboard | PageUp/Down, ArrowUp/Down |
| `currentPage` prop | `scrollToPage` (deeplink / controller) |
| IntersectionObserver | Emit `onPageChange` |

### Overlay (SPEC-128)

Fetch layout for **active** page only. Mount overlay absolutely on that sheet’s
wrapper. Do not fetch N layouts for the whole stack.

### Zoom

On scale change, remeasure page height for placeholders; keep active page
centered / start-aligned after zoom.

## Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| Controlled prop fights user scroll | Controller is SSOT; prop used for external sets |
| Text layer + many pages cost | Windowing; disable text layer offscreen if needed |
| Annotation layer memory | Same windowing |

## Cross-refs

- Architecture: [04-target-architecture.md](../04-target-architecture.md)
- Edge cases: [09-edge-cases.md](../09-edge-cases.md)
