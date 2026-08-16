# 06 — UX / UI Spec

Lenses: [004-ux-ui.md](05-lenses/004-ux-ui.md), [005-front-designer.md](05-lenses/005-front-designer.md). Coords: [12-coordinate-systems.md](12-coordinate-systems.md).

## Surface

Primary: document detail left pane (`SideBySideViewer` → `PDFViewer`). Same `PDFViewer` in `DocumentViewerDialog`. Not `PDFMarkdownSplitView`.

## Overlay off (default)

```ascii
  ┌─────────────────────────────────────────────────────────┐
  │  ⟨  3 / 12  ⟩     100%  −  +   ⛶     [Layout]          │
  ├─────────────────────────────────────────────────────────┤
  │                                                         │
  │                    PDF page canvas                      │
  │                    (text layer on)                      │
  │                                                         │
  └─────────────────────────────────────────────────────────┘
```

`[Layout]` is an outline toggle, `aria-pressed=false`.

## Overlay on

```ascii
  ┌─────────────────────────────────────────────────────────┐
  │  ⟨  3 / 12  ⟩     100%  −  +   ⛶     [Layout●]         │
  │  [Figures●] [Charts●] [Tables●] [¶] [Columns] [Noise]   │
  ├─────────────────────────────────────────────────────────┤
  │  ┌──────── figure ────────┐                             │
  │  │                        │  ┌ paragraph ┐              │
  │  │     architecture       │  │  body…    │              │
  │  │                        │  └───────────┘              │
  │  └────────────────────────┘                             │
  │  ╎        column 1        ╎  ╎     column 2     ╎      │
  └─────────────────────────────────────────────────────────┘
```

- Boxes: 2px stroke, translucent fill, 11px label.
- Columns: dashed, lower z-index than figures.
- Noise (logo): only if Noise chip on.

## States

| State | UI |
|-------|-----|
| `layout_status=extracted` + regions | Toggle enabled |
| `extracted` + 0 regions | Toggle on shows empty overlay + “No regions on this page” |
| `skipped` (L2 off, L0/L1 present) | Toggle enabled; boxes from L0/L1/derived only |
| `skipped` + no regions | Toggle disabled, tooltip |
| `failed` | Toggle disabled, tooltip “Layout failed”; PDF still works |
| `pending` | Toggle disabled, “Layout not ready” |
| Non-PDF document | Control hidden |

## Keyboard

| Key | Action |
|-----|--------|
| `O` | Toggle overlay (when enabled) |
| `1`–`6` | Toggle chips in chip order (optional v1; chips clickable is P0) |
| Existing | Page arrows / zoom unchanged |

## Click

Figure/chart/table box with `asset_path` → scroll markdown to matching image / drawing tag. No `asset_path` → no markdown jump; optional tooltip with class + confidence.

## Zoom / resize

Re-measure on `onRenderSuccess` and window resize. Overlay parent = page CSS box. See [12-coordinate-systems.md](12-coordinate-systems.md).

## i18n

Keys under `documents.viewer.layout.*` (toggle, chips, empty, failed, skipped). English defaults in source.

## Playwright (UX acceptance)

See [08-test-protocol.md](08-test-protocol.md) T-overlay-*. Fixture: synthetic two-column page with one figure.

## Cross-refs

- Front: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- API: [04-target-architecture.md](04-target-architecture.md)
