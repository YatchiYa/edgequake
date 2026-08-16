# 03 — Code as-is

Grounded in the EdgeQuake tree + sibling pdf2md **0.9.10**. Line numbers are approximate and will drift; paths are the SSOT.

## Two-crate stack

```ascii
  edgequake-pdf2md 0.9.10 (crates.io, sibling git)
       │  Pass-A: page raster → VLM OCR markdown
       │  L0 StructTree + L1 object cluster → VisualRegion crops
       ▼
  edgequake-pdf
       │  write PNG assets, optional FigureFilter, assemble MD
       ▼
  edgequake-api  persist mm-assets BYTEA, serve PDF download
       ▼
  edgequake_webui  react-pdf page viewer + markdown images
```

Workspace pin: `edgequake/Cargo.toml` `edgequake-pdf2md = { version = "0.9.10", default-features = false, features = ["bundled"] }`.

`[patch.crates-io]` remaps **only** `edgequake-llm`. Local pdf2md edits are **invisible** until a path patch or published bump.

## Convert flow (`VisionPdfConverter::convert`)

File: `edgequake/crates/edgequake-pdf/src/backend/vision.rs`

```ascii
  PDF bytes
    │
    ├─ Pass A: pdf2md convert_from_bytes (page PNG → VLM → markdown)
    │
    └─ if write_plan().any_writer():
         1)  write_embedded_figure_assets  → figure_map
         1b) write_caption_region_assets   → merge figs + table_map  (L0+L1)
         1c) FigureFilter IF provider Some → manifest JSON only
         2)  write_page_png_assets         → page-NNNN.png (viewer)
         3)  chart residual                → page-NNNN-chart.png
         assemble_vision_markdown_with_figures(full figure_map)
```

### G-prune (open)

After `filter.run`:

- logs `kept` / `discarded`
- `write_manifest(assets_root, results)` → `figure_filter_manifest.json`
- **no** `figure_map.retain`
- **no** PNG delete
- assemble injects **every** map entry

Contract tests (`contract_spec049_figure_filter.rs`) assert Pass-1 kinds, **not** prune of the map.

### G-wire (open)

`PageDrawingAssetsConfig::with_defaults` sets `figure_filter_provider: None`.

`page_drawing_assets_config_for_vision` (`document_assets.rs`) applies extract flags and analyze tags — **never assigns the provider**.

Call site: `pdf_processing.rs` (~L956–962). Parse API `handlers/parse/options.rs` same.

**Production ingest never runs FigureFilter.** Image area skip in pdf2md assumed Pass-1 would drop logos (comment on `region_area_ok`).

`include_pdf_assets.rs` re-extracts from stored PDF bytes: **no OCR, no FigureFilter**.

## pdf2md visual cascade (L0/L1 only)

Files (sibling): `src/pipeline/visual/{mod,geometry,object_cluster,struct_tree,caption_label,render_crop,types,precision}.rs`

```ascii
  extract_page_regions
    ├─ L0 StructTreeProposer (Figure/Table; l0_area_ok)
    ├─ L1 propose_object_clusters (skip if IoU ≥ DEDUP_IOU with L0)
    ├─ refine_proposals (Form/Image-first; path-only suppress)
    ├─ attach_caption_labels (label only; 80 pt)
    └─ render crop PNG
```

`RegionSource` today: `StructTree | ObjectCluster`. **No layout variant. No `ort`.**

| Constant | Value | Notes |
|----------|-------|-------|
| `MIN_AREA_FRAC` | 0.02 | non-image `area_ok` |
| `MAX_AREA_FRAC` | 0.55 | G3 |
| `DEDUP_IOU` | 0.25 | L0 wins |
| Image min | 24 pt | **skips** `MIN_AREA_FRAC` |
| `MIN_FIGURE_EDGE_PX` | 24 | `extract_images.rs` |
| Aspect max | **absent** | WP-2 |

## FigureFilter (L3 code, unwired)

`edgequake-pdf/src/figure_filter.rs` + `vision_prompts.rs`

- Pass-1: `is_figure` + `FigureKind`
- Discard today: `Logo`, `IconLogo`, `TextBlock`, `DecorativeRule`, `Empty`
- Keep: charts, diagrams, photo, `Other` (fail-open unknown)
- Sequential (no `buffer_unordered`)
- Missing kinds: Stamp, Signature, ScanArtefact, Watermark
- Depends on `LLMProvider` only (SOLID)

## Storage as-is

| Grain | Table | Page? |
|-------|-------|-------|
| Document shell | `documents` | no `page_count` column |
| PDF 1:1 | `pdf_documents` | `page_count` INTEGER |
| PDF bytes | `pdf_document_blobs` | — |
| Markdown | `pdf_documents.markdown_content` + `documents.content` | `<!-- edgequake-page:N -->` |
| PNGs | `document_mm_assets` | `page_num`, **no bbox** |
| Sidecars | `document_artifacts` | CHECK kinds; no layout |
| Chunks | `chunks` | typed `page_start` unused; JSONB filled |

SPEC-032 `pdf_pages` was **never migrated**.

`WrittenFigureAsset.bbox: Option<(f32,f32,f32,f32)>` — PDF space, IoU only, **not persisted**.

## API as-is (viewer)

| Route | Role |
|-------|------|
| `GET /documents/{id}` | markdown + `pdf_id` |
| `GET /documents/pdf/{pdf_id}/download` | bytes for react-pdf |
| `GET /documents/{id}/assets` | summaries, **no bbox** |
| `GET /documents/{id}/assets/{asset_id}` | PNG |
| `GET /documents/{id}/lineage` | chunks + `page_start` |

**No** `/pages/{n}` or `/layout`.

## Frontend as-is

`edgequake_webui/src/components/documents/pdf-viewer.tsx` — `react-pdf` ^10.3.0, one page at a time, zoom 0.5–3.0, native TextLayer + AnnotationLayer, **no overlay children**, **no `onRenderSuccess`**, **no `data-testid="pdf-viewer"`**.

Primary surface: `documents/[id]/page.tsx` → `SideBySideViewer`. Inherit via `DocumentViewerDialog`. Do **not** start on `PDFMarkdownSplitView` (no `currentPage`).

Figures: markdown images via `AuthenticatedMarkdownImage` + URL rewrite (`documents.ts` **and** `document-assets.ts` — DRY debt). `listDocumentAssets()` unused.

Highlights: markdown `highlightLineRange` / `?page=` (SPEC-033). **Not** PDF quads.

## Tests as-is

| Suite | Covers | Missing |
|-------|--------|---------|
| `contract_spec049_figure_filter.rs` | Pass-1/2, manifest | G-prune |
| `contract_spec049_visual_regions.rs` | G2, E4, E9, E18, E20 | layout |
| `e2e_spec049_visual_regions.rs` | 048 corpus | overlay |
| `document-viewer.spec.ts` | viewer smoke | overlay; testid gap |

## Cross-refs

- Gaps → target: [04-target-architecture.md](04-target-architecture.md)
- Matrix: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
