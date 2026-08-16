# SPEC-128 — High-precision PDF vision + persisted page layout overlay

> **Mission:** Close the figure-filter control loop (classify → prune), tighten deterministic geometry, extract and **persist** per-page layout, and overlay classified regions on the PDF viewer — without replacing pdfium, without AGPL weights in the product binary, and without inventing crops from English captions.
>
> **Trigger:** [`zz-raw.md`](zz-raw.md) (SPEC-049 precision rewrite) plus product need: overlay figures, charts, classified objects, paragraphs, and columns on the original page.

## Short verdict

| Layer       | Finding                                                                                                                                                                                                                                                                              |
| -------------| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Gap (intake) | VLM `FigureFilter` classified crops but **did not prune** `figure_map`. Filter was **unwired** on ingest. Bboxes died after IoU. Pages were not rows. PDF viewer had no overlay. |
| Now          | Prune is SSOT after filter (fail-open on error). `document_pages` + `page_layout_regions` persist. `GET .../pages/{n}/layout` derives `bbox_norm`. Overlay on `PDFViewer` (chips; noise = display ≠ index). L2 ONNX remains **out of slice**. |
| Product     | Logos/stamps stay off GraphRAG when Pass-1 discards; overlay still shows `abandon`. Users can see *where* extraction landed.                                                                                                                                                         |
| Fix posture | Cascade L0>L1>L2>L3; persist `document_pages` + `page_layout_regions`; overlay on `PDFViewer` only; default L2 = Apache PP-DocLayoutV3 (DocLayout-YOLO AGPL = research only).                                                                                                        |

```ascii
  PDF bytes
       │
       ├─ L0 StructTree (ISO tags)          keep
       ├─ L1 Paint (Image/Form/path)        propose, then gate
       ├─ L2 Page layout (ONNX, optional)   persist + gate L1
       ├─ Write PNG only for survivors
       ├─ L3 FigureFilter (VLM)             prune figure_map
       └─ Overlay GET .../pages/{n}/layout  display ≠ index
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-128-*)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, AI, vision, system, OCR)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-edge-cases
   → 11-honest-assessment
   → 12-coordinate-systems
   → 13-layout-taxonomy
   → zz-raw.md (intake, not the contract)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D0 | Intake `zz-raw.md` | Done |
| D1 | Doc pack (this folder) | Done |
| I0 | WP-0 prune `figure_map` after filter | Done |
| I1 | WP-1 wire `figure_filter_provider` | Done |
| I2 | WP-2 image area + aspect gates (pdf2md) | Done |
| I3 | WP-3 discard taxonomy | Done |
| I4 | WP-4 concurrent Pass-1 | Done |
| I5 | WP-5 page routing | Done |
| I6 | WP-6 L2 ONNX + persist pages/regions | Planned (feature-gated; out of this slice) |
| I7 | Overlay + `GET .../pages/{n}/layout` | Done |
| I8 | WP-7 OTEL GenAI counters/spans | Lite (`figure_filter_kept` / `discarded_by_kind` tracing) |
| T1 | G-prune / G-layout / Playwright overlay | Done (`make spec128-proof`; G-overlay IoU on fixture `S01–S05`; live real-PDF `R01–R05` / ingest `I01–I05` when stack is up) |

## Related

- [SPEC-049](../049-improve-figure-extraction/) — cascade ontology; this spec **completes** prune + L2 persist + overlay (049 P2 “only if L0+L1 empty” is **superseded** here)
- [SPEC-032](../032-graph/003-lineage-data-model.md) — sketched `pdf_pages`; **never migrated**
- [SPEC-033](../033-page-lineage/) — page jump `?page=`; overlay filters to `currentPage`
- [SPEC-047](../047-rag-evaluation/) / mm-assets — PNG identity; layout is **not** BYTEA
- [SPEC-091](../091-simplify-data-layer/) — typed sidecars, not KV blobs
- Sibling crate: `/Users/raphaelmansuy/Github/03-working/edgequake-pdf2md` (crates.io `0.9.10`)

## Non-goals (v1)

- Replacing pdfium-render as render/StructTree engine
- English caption regex as **primary** region detector
- Raising `MAX_AREA_FRAC` to chase recall
- Unpinned runtime download of layout weights
- Shipping AGPL DocLayout-YOLO in GHCR / product binary
- Full substitution by MinerU / PDF-Extract-Kit (offline A/B only)
- Storing layout as one document JSON blob (`document_artifacts`)
- Filling unused `chunks.page_start` typed columns (orthogonal; do not block overlay)

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
