# 04 — Target Architecture

Laws: [01-first-principles.md](01-first-principles.md). As-is: [03-code-as-is.md](03-code-as-is.md).

## Cascade (closed loop)

```ascii
  PDF bytes
    │
    ├─ [Optional] Page routing — zero Image/Form and empty L1 residual → skip figure PNG writes
    │                            (still persist paragraph/column layout when L2 on)
    │
    ├─ L0  StructTree Figure/Table          pdfium-render (existing)
    ├─ L1  Paint proposals                  Image/Form seeds + path residual
    │         + MIN_IMAGE_AREA_FRAC + MAX_ASPECT + MAX_AREA_FRAC
    │
    ├─ L2  Page layout extraction           feature `layout-onnx` (default off)
    │         full-page raster → PageLayoutExtractor
    │         persist ALL boxes (overlay)
    │         GATE L1 if IoU(proposal, figure|chart|table) < LAYOUT_IOU
    │         NEVER drop L0
    │
    ├─ Persist document_pages + page_layout_regions   BEFORE bbox drop
    │         derived columns from text-box x-overlap
    │
    ├─ Write PNG only for figure/table survivors (L0 ∪ gated L1)
    │
    ├─ L3  FigureFilter Pass-1 concurrent + Pass-2 kept
    │         prune figure_map; optional delete discarded PNGs
    │         overlay regions stay (LAW-128-2)
    │
    └─ Assemble markdown / Drawing tags from KEPT set only
            + OTEL GenAI spans (figure.propose / figure.layout / figure.filter.*)
```

**L2 vs SPEC-049 P2:** L2 is **not** “only if L0+L1 empty”. When enabled it always runs for persistence + overlay, and gates L1. LAW-128-15.

## SOLID module map

```ascii
  pdf2md  pipeline/visual/
    geometry.rs          # constants + area_ok + iou (pure)
    object_cluster.rs    # L1 proposer
    struct_tree.rs       # L0 proposer
    layout_page.rs       # NEW: PageLayoutExtractor + OrtLayoutExtractor
    taxonomy.rs          # NEW: model class → canonical (or EdgeQuake crate)
    visual/mod.rs        # orchestrate L0 then L1 then optional L2 refine

  edgequake-pdf
    figure_filter.rs     # L3 only — does not write PNG, does not persist
    backend/vision.rs    # prune + persist hook + routing
    layout_persist.rs    # NEW: PageLayout → SQL DTO (or api service)

  edgequake-storage
    page_layout_storage.rs  # NEW: document_pages + regions

  edgequake-api
    handlers/documents/query/pages.rs  # NEW GET pages + layout
    services/document_assets.rs        # WP-1 wire provider

  edgequake_webui
    pdf-viewer.tsx                     # toolbar toggle + measured size
    pdf-page-overlay.tsx               # NEW: % boxes from bbox_norm
    types/page-layout.ts               # NEW: codegen from OpenAPI
```

### Traits (open/closed)

```rust
pub trait PageLayoutExtractor: Send + Sync {
    fn extract(&self, page_png: &image::DynamicImage) -> Result<PageLayout, LayoutError>;
}

pub struct PageLayout {
    pub page_width_pt: f32,
    pub page_height_pt: f32,
    pub rotation: i16,
    pub raster_width_px: u32,
    pub raster_height_px: u32,
    pub model_id: String,      // e.g. pp-doclayout-v3
    pub model_sha256: String,
    pub regions: Vec<LayoutRegion>,
}

pub struct LayoutRegion {
    pub class: CanonicalClass, // see 13-layout-taxonomy
    pub source: RegionSource,  // L0 | L1 | L2 | L3 | Derived
    pub bbox_pdf: BBoxPdf,     // x0,y0,x1,y1 bottom-left points
    pub confidence: Option<f32>,
    pub reading_order: Option<i32>,
    pub asset_path: Option<String>,
}

// Keep L1 proposal if:
//   layout disabled
//   OR source == StructTree (L0)
//   OR IoU(proposal, any figure|chart|table layout box) >= LAYOUT_IOU
//   OR (has_form && FORM_LAYOUT_EXEMPT)  // vector figures layout may miss
```

`NoopLayoutExtractor` when feature off or model missing → empty L2, L0/L1 still persist.

## Data model

```ascii
  documents (1)
       │
       ├── pdf_documents (0..1)           existing
       ├── document_mm_assets (N)         existing PNGs
       └── document_pages (N)             NEW
                │  UNIQUE (document_id, page_number)
                └── page_layout_regions (M)   NEW
                         optional asset_path → mm-assets identity
```

See [05-lenses/003-database.md](05-lenses/003-database.md) for DDL.

Write **after** L0/L1/L2 merge, **before** `WrittenFigureAsset.bbox` is dropped, **before** chunking. Reprocess: delete pages/regions for document, rewrite (same as mm-assets).

Optional `pdf_id` on `document_pages` if rows are inserted while `document_id` is still null during processing — prefer waiting until `document_id` is set unless crash-resume requires otherwise.

## API contract (OpenAPI SSOT)

```ascii
  GET /api/v1/documents/{document_id}/pages
      → { document_id, pages: [{ page_number, width_pt, height_pt, rotation,
                                 layout_status, region_count_by_class }] }

  GET /api/v1/documents/{document_id}/pages/{page_number}/layout
      → { page_number, width_pt, height_pt, rotation,
          layout_model, layout_status,
          regions: [{ region_id, class, source, confidence, reading_order,
                      asset_path,
                      bbox_pdf: {x0,y0,x1,y1},
                      bbox_norm: {x,y,w,h} }] }   // top-left 0–1
```

`bbox_norm` is **derived** in the handler (LAW-128-4, LAW-128-10). RLS: workspace isolation identical to mm-assets.

## Overlay architecture

```ascii
  PDFViewer
    toolbar: [prev][page][next]  [zoom]  [Layout overlay]
             chips: Figures Charts Tables Paragraphs Columns Noise
    Page (react-pdf canvas)
    PdfPageOverlay          // absolute, size = onRenderSuccess CSS box
         boxes as % left/top/width/height from bbox_norm
```

FE must not recompute PDF→CSS from `scale` alone (width-fit breaks it). See [12-coordinate-systems.md](12-coordinate-systems.md) and [06-ux-ui-spec.md](06-ux-ui-spec.md).

## Observability

| Signal | Examples |
|--------|----------|
| Counters | `xobjects_seen`, `geometry_kept`, `layout_kept`, `vlm_kept`, `vlm_discarded_by_kind` |
| Histograms | `layout_ms`, `pass1_ms`, `pass2_ms` |
| Spans | `figure.propose`, `figure.layout`, `figure.filter.pass1`, `figure.filter.pass2` |

Align names with [OpenTelemetry GenAI](https://opentelemetry.io/docs/specs/semconv/gen-ai/) where the call is an LLM; layout inference is a model span with pinned `model_id` + sha256 attrs.

## Repo / feature strategy

```ascii
  Dev:   [patch.crates-io] edgequake-pdf2md = { path = "../../edgequake-pdf2md" }
  Ship:  publish pdf2md with optional feature layout-onnx
         bump EdgeQuake pin; GHCR image: CPU EP default
         weights: vendored under models/ + SHA-256 in CI
         NOT downloaded at runtime in production
```

`edgequake-api` `vision` feature is an empty stub today — do not overload it. New feature lives on **pdf2md** (`layout-onnx`) and is optional on the API binary.

## Banned (still)

- Searching `"Figure "` / `"Fig. "` to invent crops
- Magic vertical ceilings as sole geometry
- Emitting Drawing for `page-NNNN.png`
- Manifest-only classification without pruning `figure_map`
- Unpinned weight download
- Bundling AGPL DocLayout-YOLO

## Cross-refs

- Taxonomy: [13-layout-taxonomy.md](13-layout-taxonomy.md)
- Coords: [12-coordinate-systems.md](12-coordinate-systems.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- DB lens: [05-lenses/003-database.md](05-lenses/003-database.md)
