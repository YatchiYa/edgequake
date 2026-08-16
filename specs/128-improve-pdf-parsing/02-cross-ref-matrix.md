# 02 — Cross-Ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Filter classifies, does not prune `figure_map` | [`vision.rs`](../../edgequake/crates/edgequake-pdf/src/backend/vision.rs) ~L245–268 |
| `figure_filter_provider` never set on ingest | [`pdf_processing.rs`](../../edgequake/crates/edgequake-api/src/services/) ~L956–962; [`document_assets.rs`](../../edgequake/crates/edgequake-api/src/services/document_assets.rs) `page_drawing_assets_config_for_vision` |
| Defaults `None` | [`backend/mod.rs`](../../edgequake/crates/edgequake-pdf/src/backend/mod.rs) `PageDrawingAssetsConfig::with_defaults` |
| Images skip `MIN_AREA_FRAC` | pdf2md `object_cluster.rs` `region_area_ok` |
| Geometry constants | pdf2md `geometry.rs`: `MIN_AREA_FRAC=0.02`, `MAX_AREA_FRAC=0.55`, `DEDUP_IOU=0.25` |
| No L2 / no `RegionSource::Layout` | pdf2md `types.rs`; no `ort` in pdf2md `Cargo.toml` |
| Bbox dropped after IoU | `WrittenFigureAsset.bbox` in `embedded_images.rs`; not in `document_mm_assets` |
| No page table | migrations: `022` `pdf_documents.page_count` only; SPEC-032 `pdf_pages` unimplemented |
| Chunk typed page columns unused | migration `066`; writers put `page_start` in JSONB metadata |
| Overlay absent | [`pdf-viewer.tsx`](../../edgequake_webui/src/components/documents/pdf-viewer.tsx) |
| E2E testids missing | `e2e/document-viewer.spec.ts` looks for `data-testid="pdf-viewer"` |
| pdf2md pin crates.io 0.9.10 | [`edgequake/Cargo.toml`](../../edgequake/Cargo.toml); `[patch.crates-io]` is llm-only |
| DocLayout-YOLO AGPL | https://github.com/opendatalab/DocLayout-YOLO |
| PP-DocLayoutV3 Apache-2.0 | https://huggingface.co/PaddlePaddle/PP-DocLayoutV3 |
| ort 2.0.0-rc.13 | https://ort.pyke.io/ |
| Laws | LAW-128-1..18 ([01-first-principles.md](01-first-principles.md)) |
| Intake | [zz-raw.md](zz-raw.md) |

## Code SSOT (as-is → target)

| Concern | As-is path | Target |
|---------|------------|--------|
| Vision convert + figure_map | `edgequake-pdf/src/backend/vision.rs` | prune after filter; persist layout before bbox drop |
| Filter | `edgequake-pdf/src/figure_filter.rs` | concurrent Pass-1; new discard kinds |
| Prompts | `edgequake-pdf/src/vision_prompts.rs` | Stamp/Signature/ScanArtefact/Watermark |
| Config | `PageDrawingAssetsConfig` | set `figure_filter_provider` from ingest LLM |
| Ingest wire | `pdf_processing.rs` + `document_assets.rs` | WP-1 |
| Include-from-pdf | `include_pdf_assets.rs` | prune too |
| L0/L1 geometry | sibling pdf2md `pipeline/visual/*` | WP-2 constants; optional `layout_page.rs` |
| MM assets | `document_mm_assets` | unchanged BYTEA; optional `asset_path` on regions |
| Pages / regions | **missing** | migration ~148 `document_pages` + `page_layout_regions` |
| Layout API | **missing** | `GET /documents/{id}/pages`, `.../pages/{n}/layout` |
| Overlay | `pdf-viewer.tsx` | `PdfPageOverlay`; measured `onRenderSuccess` |
| Coord transform | **missing** | one Rust fn → `bbox_norm`; FE uses % of measured box |
| Taxonomy map | **missing** | one mapper ([13-layout-taxonomy.md](13-layout-taxonomy.md)) |

## Related specs / issues

| Spec / Issue | Relationship |
|--------------|--------------|
| [SPEC-049](../049-improve-figure-extraction/) | Parent cascade; G1–G5 live; prune + L2 persist + overlay **this spec** |
| [SPEC-032](../032-graph/003-lineage-data-model.md) | `pdf_pages` ERD ancestor — implement as `document_pages`, not a dump into pdf_documents |
| [SPEC-033](../033-page-lineage/) | `currentPage` / `?page=`; overlay filters to that page |
| SPEC-047 / mm-assets | PNG identity; layout is not an image |
| [SPEC-091](../091-simplify-data-layer/) | Typed sidecars; do not abuse `document_artifacts.kind` CHECK |
| SPEC-015V | Writer gates; full-page PNG never Drawing |
| SPEC-121 | PDF admit path; this spec does not change format matrix |
| SPEC-048 / 030 | **Do not** cover PDF overlay (list UX only) |
| [zz-raw.md](zz-raw.md) | Intake WP-0..8; L2 model choice **refined** by LAW-128-5 |

## ASCII dependency

```ascii
  SPEC-049 (cascade L0–L3, G1–G5)
       │
       ├─ SPEC-033 (page jump, markdown highlight)
       ├─ SPEC-047 (mm-asset PNG identity)
       ├─ SPEC-091 (typed tables)
       └─ SPEC-128 (this)
              ├─ closes prune loop (049 leftover)
              ├─ L2 persist + overlay (049 P2 rewritten)
              ├─ document_pages (032 ancestor)
              └─ does NOT replace pdfium / Pass-A OCR
```

## DRY rules

1. **Geometry constants** live only in pdf2md `geometry.rs` (plus EdgeQuake contract pins).
2. **PDF → norm** lives only in one Rust transform; OpenAPI DTO carries both `bbox_pdf` and `bbox_norm`.
3. **Taxonomy** lives only in the mapper ([13-layout-taxonomy.md](13-layout-taxonomy.md)); FE never hardcodes PP-DocLayout class ids.
4. **Overlay** lives only in `PDFViewer` / `PdfPageOverlay`. Split viewers inherit.
5. **Prune** is one function: kept set → rebuild map → assemble → persist mm-assets. Ingest and include-from-pdf both call it.

## External refs

- pdfium-render: https://github.com/ajrcarey/pdfium-render
- ONNX Runtime / ort: https://onnxruntime.ai/ · https://ort.pyke.io/
- DocLayout-YOLO (research): https://github.com/opendatalab/DocLayout-YOLO · arXiv:2410.12628
- PP-DocLayoutV3: https://huggingface.co/PaddlePaddle/PP-DocLayoutV3
- OpenTelemetry GenAI: https://opentelemetry.io/docs/specs/semconv/gen-ai/
- ISO 32000 overview: https://pdfa.org/resource/iso-32000-2/

## Cross-refs

- Why: [00-why.md](00-why.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
