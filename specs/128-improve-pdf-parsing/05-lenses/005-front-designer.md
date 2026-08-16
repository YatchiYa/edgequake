# Lens 005 — Front Designer

## Component SSOT

`PdfPageOverlay` is the only drawing surface. Host: `PDFViewer`. Consumers (`SideBySideViewer`, `DocumentViewerDialog`) pass no extra layout logic.

```ascii
  PDFViewer
    props += overlayEnabled, onOverlayChange, documentId
    fetch GET .../pages/{pageNumber}/layout  when overlayEnabled
    onRenderSuccess → { cssWidth, cssHeight }
    <div relative data-testid="pdf-viewer">
      <Page ... />
      {overlayEnabled && layout && (
        <PdfPageOverlay
          regions={filtered}
          pageCss={{ width, height }}
          chips={...}
        />
      )}
    </div>
```

Do **not** implement overlay on `PDFMarkdownSplitView`.

## Layout of a box

`bbox_norm` `{x, y, w, h}` is 0–1, origin **top-left**. CSS:

```
left:   x * 100%
top:    y * 100%
width:  w * 100%
height: h * 100%
```

Parent is the measured page wrapper (same box as canvas). Ignore `scale` for math.

## Tokens (semantic, theme-aware)

| Class | Stroke | Fill |
|-------|--------|------|
| figure | `--layout-figure` | 12% |
| chart | `--layout-chart` | 12% |
| table | `--layout-table` | 12% |
| paragraph | `--layout-paragraph` | 8% |
| column | `--layout-column` dashed | 6% |
| abandon / noise | `--layout-noise` | 10% |
| title / caption / other | `--layout-meta` | 8% |

Define CSS variables in the viewer stylesheet; dark mode via existing theme.

## Chips

Default ON: Figures, Charts, Tables. Default OFF: Paragraphs, Columns, Noise. Persist chip state in `sessionStorage` keyed by `eq-layout-chips` (not server).

## Testids (E2E already expects some)

- `data-testid="pdf-viewer"` on wrapper (missing today)
- `data-testid="side-by-side-viewer"` on split
- `data-testid="pdf-layout-overlay"`
- `data-testid="pdf-layout-box"` + `data-class={class}`
- `data-testid="pdf-layout-toggle"`

## DRY

One `rewriteMarkdownMmAssetUrls` module. Click-through uses `asset_path` stem → existing asset URL helper.

## a11y

- Toggle `aria-pressed`
- Overlay `aria-hidden` when decorative; legend text for each visible class
- `prefers-reduced-motion`: no box fade
- Focus order: toggle → chips → page nav

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Coords: [../12-coordinate-systems.md](../12-coordinate-systems.md)
- Types: OpenAPI → `src/types/page-layout.ts`
