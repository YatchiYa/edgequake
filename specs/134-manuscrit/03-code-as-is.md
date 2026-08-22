# 03 — Code as-is (Slice E)

Grounded snapshot of production PDF→MD **after** WP-1…D wiring and **including**
the Slice E holes that still produced the empty-placeholder + crop-gallery UI.
Paths relative to repo root unless noted.

## End-to-end (pre–Slice E — the live failure)

```ascii
  POST /api/v1/documents/pdf
       │
       ├─ SPEC-038 EdgeParse Auto  (BEFORE modality; skip_edgeparse_fastpath unread)
       │     ≥200 chars/page OCR layer → skip Vision
       │
       ├─ classify_document_majority  (one modality for the whole file)
       │
       ├─ ManuscriptProfile intent: DPI 300, max_pixels 3600, MS prompt
       │
       └─ VisionPdfConverter::convert
              pdf2md convert_from_bytes
                dpi is SET but IGNORED (render.rs `_dpi`)
                max_rendered_pixels NOT forwarded → default 2000 long-edge
              ImageGuard: PNG→JPEG; may downscale to 1024px
              empty VLM page → empty markdown string
              assemble_vision_markdown_with_figures
                empty body → EMPTY_VISION_PAGE_PLACEHOLDER
                THEN prepend fig/table/chart PNG hrefs
              escalate_empty_pages requires body.trim() == PLACEHOLDER
                prepended crops → retry never runs
```

## Slice E production path (this implementation)

```ascii
  PDF bytes
       │
       ├─ [1] classify_pages_from_bytes  (before EdgeParse Auto)
       │      per-page heuristic; unsampled pages inherit majority
       │      skip_edgeparse_fastpath honored when any page is MS-like
       │
       ├─ [2] PageConvertPlan
       │      all print  → one convert (Acc English, 2000px, figure filter ON)
       │      all MS     → one convert (MS prompt, 3600px forwarded to pdf2md)
       │      mixed      → two converts (PageSelection::Set) + stitch by page marker
       │
       ├─ [3] Pass-A  VisionPdfConverter
       │      .dpi(dpi) AND .max_rendered_pixels(profile)
       │      ImageGuard: JPEG q85; MS long-side floor 2000 (print 1024)
       │
       ├─ [4] Assets
       │      page-NNNN.png always for viewer (3600px on MS)
       │      MS / empty Pass-A: do not inject fig/chart hrefs or <drawing/> tiles
       │      no caption-region / chart-residual re-inject after tiling clear
       │
       ├─ [5] escalate if section CONTAINS placeholder and has no other text
       ├─ [6] verify_manuscript_markdown (MS-like, fail-open)
       └─ [7] Pass-B analyze (print crops only; MS suppress remains)
```

## Prompt SSOT

File: `edgequake/crates/edgequake-pdf/src/vision_prompts.rs`

- `RAG_PAGE_VISION_SYSTEM_PROMPT` — print Acc English pin (unchanged).
- `RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT` — source language, `[?]`, whole-graphic.
- `pass_a_system_prompt_for(modality)` routes print vs manuscript|mixed.

Print Acc path must remain byte-identical for all-print documents.

## Render (pdf2md 0.9.11 fact)

`edgequake-pdf2md` `render_pages_blocking` takes `_dpi: u32` (**unused**) and
rasterizes with `PdfRenderConfig::set_target_width(max_pixels)`. Therefore
**`max_rendered_pixels` is the Pass-A long-edge**, not a DPI-independent cap.

| Knob | Print | Manuscript (Slice E) |
|------|-------|----------------------|
| `ConversionConfig.dpi` | adaptive 96–150 | profile 300 (still ignored by pdf2md raster) |
| `ConversionConfig.max_rendered_pixels` | default 2000 | **forwarded 3600** |
| Viewer `page-NNNN.png` | 2000 | 3600 |
| ImageGuard min long side | 1024 | **2000** (JPEG-only first) |

A4 @ 2000px ≈ 163 DPI; @ 3600px ≈ 300 DPI — matches LAW-134-3 without waiting
for an upstream pdf2md DPI fix.

## Assemble / escalate (LAW-134-20)

- Empty Pass-A body → placeholder, **zero** fig/chart/table hrefs.
- Manuscript-like convert group → no fragment inject even when Pass-A has text.
- `section_needs_empty_escalation` matches placeholder **containment** plus no
  remaining prose (crops no longer defeat retry).
- Full-page PNG stays viewer-only (dual-pane); it is the escalate/verify image.

## EdgeParse (LAW-134-12)

`ManuscriptProfile.skip_edgeparse_fastpath` is consulted on the production
path. Classification (or env force) runs **before** `try_edgeparse_fast_path`.
Any sampled manuscript-like page vetoes Auto. All-print dense text still takes
SPEC-038 (no regression).

## Per-page vs majority

`classify_document_majority` remains for document metadata / verify gating.
**Convert policy is per-page grouped** (`PageConvertPlan`): mixed files do not
apply the MS prompt to print pages (Acc English pin preserved on those pages).

## Persistence

Document metadata JSON (WP-5 lite): `page_modality`, `page_modalities` (per-page
list), `grounding_*`, `pages_escalated`, `pages_failed`. Per-page DB columns
and UX chip remain WP-7 / future.

## Tests

- Contracts: `contract_spec134_*` plus Slice E behavioral tests (assemble, pixels
  forwarded, EdgeParse veto, mixed stitch).
- `print_document_byte_identical_regression_guard` — all-print Acc path.
- Python study harness: `specs/134-manuscrit/study/` (private PDF, gitignored out).

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
