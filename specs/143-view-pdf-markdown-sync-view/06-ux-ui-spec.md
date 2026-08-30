# 06 — UX / UI Spec

## Layout (unchanged shell)

```ascii
┌──────── Sidebar ─┬──────── PDF pane ────────┬──── Markdown pane ────┐
│                  │  toolbar: </ > N/T zoom  │  [Page N] sticky hint │
│                  │  [sync toggle] (s-b-s)   │                       │
│                  │  continuous page stack   │  anchors data-eq-page │
│                  │  overflow-y-auto         │  overflow-y-auto      │
└──────────────────┴──────────────────────────┴───────────────────────┘
```

## Sync toggle

- Placement: side-by-side header next to view-mode buttons.
- Default: ON when `initialMode === 'side-by-side'`.
- States: `data-sync="on"|"off"`; disabled when `hasPageAnchors === false`.
- Tooltip ON: “Synchronize PDF and Markdown pages”.
- Tooltip OFF: “Independent scrolling”.
- Tooltip disabled: “No page markers in this document”.

## PDF pane behavior

1. Pages stacked vertically with small gap.
2. Wheel / trackpad scrolls the stack.
3. Active page = highest intersection with viewport center band.
4. Toolbar and keyboard scroll the active sheet into view.
5. Indicator always shows active page.

## Markdown pane behavior

1. Markers become invisible anchors (`aria-hidden`).
2. When sync ON and PDF drives: `scrollIntoView` on `#eq-md-page-N`.
3. When user scrolls MD: observer reports page → PDF follows.
4. Sticky “Page N” updates from controller `activePage`.

## Motion

- Prefer `behavior: 'smooth'` for programmatic jumps.
- During driver lock, suppress follower smooth-scroll storms (instant or skip).

## Cross-refs

- Front designer: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
