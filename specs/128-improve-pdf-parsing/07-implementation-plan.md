# 07 — Implementation Plan

Principles: DRY / SOLID / LAW-128-*. Docs in this folder are the contract; code follows.

## Sequence

```ascii
  Week 1  Precision (no ONNX)
          WP-0 prune · WP-1 wire filter · WP-2 geometry · WP-3 taxonomy
          WP-8 fixtures start · G-prune green

  Week 2  WP-4 concurrency · WP-5 router · persist L0/L1 pages (schema)
          GET layout · overlay MVP (L0/L1 boxes) · WP-7 telemetry start

  Week 3+ WP-6 layout-onnx feature-flagged · derived columns
          G-layout / G-industrial · overlay chips · docs CHANGELOG
```

**Precision release (ship without L2):** WP-0, WP-1, WP-2, WP-3, G-prune + G6/G7.

**Layout release:** schema + API + overlay (even L2 off) then WP-6.

**Overlay can ship on L0/L1 boxes** before ONNX — do not block UX on model packaging.

## WP-0 — Close the filter control loop (P0)

**Modules:** `backend/vision.rs`, `figure_filter.rs`, `include_pdf_assets.rs`

**Behavior**

1. After successful `FigureFilter::run`:
   - `kept` = `is_figure == true`
   - rebuild `figure_map` to kept paths; drop empty pages
2. Optionally delete discarded PNGs under `assets_root`
3. Log kept / discarded by `FigureKind`
4. Fail-open on filter error (keep all) unless tiny-crop fail-closed config
5. **Same helper** on include-from-pdf if filter runs there (or skip VLM but still do not resurrect pruned files from a previous run)

**Acceptance:** mock 3 keep / 2 discard → assemble references only 3. Extend `contract_spec049_figure_filter.rs` + new assemble contract.

**Estimate:** 0.5–1 day

## WP-1 — Default-enable filter when vision LLM exists

**Modules:** `document_assets.rs`, `pdf_processing.rs`, parse `options.rs`

If Pass-A has a resolved LLM and `extract_figures` and `EDGEQUAKE_FIGURE_FILTER` not `0`, set `figure_filter_provider` to that `Arc`. Document env in README + `.env.example`.

**Estimate:** 0.5 day

## WP-2 — Deterministic geometry for images (P1)

**Repo:** sibling pdf2md — path patch during dev, then publish.

| Gate | Invariant |
|------|-----------|
| Min page-area images | `MIN_IMAGE_AREA_FRAC` start `0.008` |
| Max page-area | `MAX_AREA_FRAC = 0.55` |
| Max aspect | `MAX_ASPECT ≈ 8.0` |
| Min edge | 24 pt / 24 px |

Files: `geometry.rs`, `object_cluster.rs` `region_area_ok`, `struct_tree.rs` `l0_area_ok`, `extract_images.rs`.

Process rule: constant change requires corpus Δ (048 e2e + pdf2md `test_cases/`).

**Estimate:** 1–2 days

## WP-3 — Discard taxonomy + prompts

`FigureKind` + `FIGURE_FILTER_PASS1_SYSTEM`: `Stamp`, `Signature`, `ScanArtefact`, `Watermark`.

**Estimate:** 0.5 day

## WP-4 — Concurrent Pass-1 / budget

`futures::stream` + `buffer_unordered`; `figure_filter_concurrency` default 4; `max_figure_vlm_per_page` default 12 (drop lowest-area first).

**Estimate:** 1 day

## WP-5 — Page routing

Skip figure asset writes when page has zero Image/Form and empty L1 residual. Still allow L2 overlay persist.

**Estimate:** 1 day

## WP-schema — Pages + regions (needed for overlay, before or with WP-6)

Migration ~148 per [05-lenses/003-database.md](05-lenses/003-database.md). Persist L0/L1 bboxes **before drop**. Storage trait + API GET. Overlay MVP.

**Estimate:** 2 days

## WP-6 — L2 ONNX (feature-gated)

New `layout_page.rs`; `PageLayoutExtractor`; PP-DocLayoutV3 pinned; CPU EP; SHA-256; fail-open; derived columns; G-layout / G-layout-coord.

**License:** Apache default. DocLayout-YOLO optional research impl, not in GHCR.

**Estimate:** 3–5 days

## WP-7 — Observability

Counters / histograms / spans per [04-target-architecture.md](04-target-architecture.md).

**Estimate:** 1–2 days

## WP-8 — Evaluation harness

Extend SPEC-049 gates; add G-prune, G-layout, G-layout-coord, G-industrial, G-cost. Synthetic page: logo + stamp + diagram. Playwright overlay.

**Estimate:** 1–2 days

## File-level checklist

### edgequake-pdf2md

- [ ] `geometry.rs` — `MIN_IMAGE_AREA_FRAC`, `MAX_ASPECT`
- [ ] `object_cluster.rs` — image branch uses new gates
- [ ] `struct_tree.rs` — align `l0_area_ok`
- [ ] `extract_images.rs` — page-area + aspect after bbox
- [ ] `layout_page.rs` — new (`layout-onnx`)
- [ ] `visual/mod.rs` — L2 refine hook
- [ ] `Cargo.toml` — optional `ort`

### edgequake-pdf / api / storage / webui

- [ ] `vision.rs` — prune; persist; routing; telemetry
- [ ] `figure_filter.rs` — concurrent; kinds
- [ ] `vision_prompts.rs` — Pass-1 taxonomy
- [ ] ingest wire `figure_filter_provider`
- [ ] migration `document_pages` + `page_layout_regions`
- [ ] GET pages / layout + OpenAPI + codegen
- [ ] `pdf-viewer.tsx` + `PdfPageOverlay` + testids
- [ ] tests: G-prune, industrial fixture, Playwright

### specs

- [x] This folder
- [ ] `specs/049-improve-figure-extraction/000-index.md` pointer
- [ ] `specs/CHANGELOG` user-facing notes at ship

## Cross-refs

- Test: [08-test-protocol.md](08-test-protocol.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Intake WP list: [zz-raw.md](zz-raw.md)
