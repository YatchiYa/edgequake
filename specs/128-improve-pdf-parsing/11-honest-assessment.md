# 11 — Honest Assessment

## What is true today (2026-08-16)

- **Prune is wired and default-on.** Vision ingest attaches the filter unless `EDGEQUAKE_FIGURE_FILTER=0` (LAW-128-14). Pass-1/2 runs after figure **and** chart crops; artefacts are dropped from `figure_map` / chart overrides; kept Pass-2 Markdown is injected into the page body. Fail-open keeps the map on `Err`. Include-from-pdf prunes after region writes and strips discarded hrefs so logos cannot resurrect.
- **`bbox_norm` has one transform** (`edgequake-storage::bbox_norm_from_pdf`); the API derives it at GET time (LAW-128-4). Overlay uses `%` of the measured page box. Rotation arms **90/180/270 have unit goldens** (not live-page IoU).
- **Pages are rows.** Migration 148 + memory/postgres adapters; persist from `page_layout.json` is fail-open (LAW-128-7). `persisted_pages` / `persist_errors` / `persist_skipped` counters + `page_layout_persist` ingest-stage histogram; persist errors **warn only** (not `record_ingestion_failure`).
- **Overlay CSS math is real (G-overlay / S).** Playwright drives `PdfPageOverlay` inside `PDFViewer`. Mocked golden `bbox_norm` CSS IoU on `overlay-letter.pdf` (`S01–S05`). Labels, empty copy, click-through to markdown `img[data-layout-asset]`. Coordinate unit, **not** an ingested paper.
- **R = prior mistral paper.** Persisted live overlay (`R01–R05`) when `SPEC128_LIVE_*` is set.
- **I = mock-geometry ingest (historical).** HIPO + `vision_provider=mock` after a geo-blocked cloud VLM. pdfium L0/L1 boxes; Pass-1 kind **not** proven there (`I01–I05`).
- **M = pdf_data + mistral-small-latest.** Live only with stack + `MISTRAL_API_KEY`. Poll layout after convert (do not wait for KG). CSS IoU ≥ 0.8 vs GET `bbox_norm` for a figure/chart box (`M01–M05`). Corpus remaining `pdf_data` PDFs: ≥1 persisted region each.
- **HTTP + RLS + cascade** are contract-tested (`GET` page 3, page 0→400 / 999→404, other workspace → 404, postgres `DELETE documents` empties `page_layout_regions`).
- **T-col-1** lives in sibling pdf2md `text_blocks::derive_columns` (two-column synthetic → 2 columns).
- **L2 ONNX is still out of slice (WP-6 Planned).** Overlay is L0/L1 (+ VLM `abandon`). Do not market “AI layout” until WP-6 is gated green.

## What this spec will not magically fix

- Pass-A VLM OCR quality (wrong reading order, invented numbers).
- Citation highlighting as PDF text quads (still markdown lines, SPEC-033).
- Unused `chunks.page_start` typed columns.
- Perfect column detection on magazines/newspapers (derived AABB).
- GPU ONNX in the default Docker image.

## Residual risks

| Risk                                 | Residual after mitigations                                                 |
| --------------------------------------| ----------------------------------------------------------------------------|
| Small-chart recall                   | Medium — 0.008 floor + G6                                                  |
| L2 false figure on decorative bitmap | Medium — L3 prune + overlay noise                                          |
| Coord bugs (rotation/crop)           | Lower for unrotated Letter; 90/180/270 unit goldens, not Playwright-zoomed |
| pdf2md publish lag                   | Process — path patch then pin bump                                         |
| Overlay “looks messy” on dense pages | UX — chips default paragraphs/columns/noise **off**                        |
| pdf.js worker                        | Same-origin `/pdf.worker.min.mjs` (copied from `pdfjs-dist`); no unpkg     |
| ort binary size / musl               | High for Alpine — document glibc image; feature off by default             |

## Sequence honesty

Shipping overlay **before** ONNX (L0/L1 boxes + L3 abandon) is the honest UX slice. L2 adds paragraphs/columns/charts split from a detector. Do not market “AI layout” until WP-6 is gated green.

## What not to “improve” (2026)

SCAN (arXiv 2505.14381) and layout-aware RAG guides: **fine-grained detector boxes as VLM/chunk units hurt retrieval**. Overlay is display (LAW-128-2). Index figure+caption (and markdown reading order), not every paragraph AABB. Do not retune `MIN_IMAGE_AREA_FRAC` without G6/G7 (LAW-128-9).

## Next (if we keep investing)

1. WP-6 ONNX / PP-DocLayoutV3 as **L1 gate + overlay classes**, fail-open, never as RAG ontology.
2. Rotation/crop Playwright IoU on live pages (unit goldens already exist).
3. Optional caption quality on industrial papers (do not invent captions — LAW-128-3).

## Cross-refs

- Why: [00-why.md](00-why.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
