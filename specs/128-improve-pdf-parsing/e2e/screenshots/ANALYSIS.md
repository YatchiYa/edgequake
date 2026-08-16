# SPEC-128 overlay screenshot analysis

## S01–S05 — G-overlay (mocked layout JSON + overlay-letter.pdf)

Captured from `PdfPageOverlay` hosted by `pdf-viewer.tsx` (react-pdf + React Query GET layout), not a cloned HTML harness.
Layout JSON is the rust golden bbox_norm (x=0.1,y=0.25,w=0.4,h=0.25); G-overlay asserts CSS IoU ≥ 0.8.
This is a **coordinate unit**, not ingest proof.

- S01: overlay off — no boxes. Toggle label **Layout**, `aria-pressed=false`.
- S02: default chips (figures/charts/tables) on; 11px class labels.
- S03: paragraphs chip adds text-block overlay.
- S04: 150% zoom uses the same bbox_norm percentages on the measured page.
- S05: noise chip shows abandon regions (not RAG-indexed).

Also mocked (no screenshot prefix): empty `extracted` copy; click figure `asset_path` → markdown `data-layout-asset-focused`.

## R01–R05 — prior mistral paper (persisted)

Document: `skill_zip_2608.11079v1.pdf` (`01a00891-0182-7afc-8843-8222ad85c3c6`), 17 pages, vision ingest (mistral-small-latest).
Unmocked `GET /api/v1/documents/{id}/pages/1/layout` returned 17 regions (paragraph + chart + abandon).
PDF bytes from `GET .../documents/pdf/{pdf_id}/download` (9.6 MiB). Overlay chips on `PDFViewer`.

- R01: overlay toggle off.
- R02: overlay on — chart box on Fig. 1 (default figures/charts/tables chips).
- R03: paragraphs chip — text-block boxes on title/authors/body.
- R04: 150% zoom — boxes stay aligned (`bbox_norm` %).
- R05: noise chip — abandon (logos) visible; not RAG-indexed.

## I01–I05 — mock-geometry ingest (historical)

HIPO ICLR paper `hipo_2607.02303v1.pdf` (`01a00958-0c7c-70a8-940a-db1be0aa66f5`), 12 pages.
This-run admit used vision + `vision_provider=mock` (cloud VLM geo-blocked). Convert still ran pdfium L0/L1; L3 kind is **not** proven here. Replaced as the live path by **M**.

## M01–M05 — pdf_data + mistral-small-latest

Live (`E2E_LIVE_STACK=1` + `MISTRAL_API_KEY`). Primary = smallest PDF under `specs/128-improve-pdf-parsing/pdf_data/`: `01-the-abondance-inversion.pdf` (`01a0097a-9cc8-71ba-a993-5831fbc7fd9c`).
Workspace + upload pin `mistral` / `mistral-small-latest`. Poll layout after convert (no KG wait). Classes: figure, paragraph, column. **Live CSS IoU vs GET bbox_norm (figure) = 1.000.**

- M01: overlay off.
- M02: overlay on.
- M03: paragraphs chip.
- M04: 150% zoom.
- M05: noise chip. Paper had no `abandon`; G-industrial asserts ≥1 figure/chart kept (LAW-128-3: no invented captions).

Corpus smoke (remaining `pdf_data` PDFs, sequential, layout poll only): coin_rag, kg, ssm each persisted ≥1 region.
