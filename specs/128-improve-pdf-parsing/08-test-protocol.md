# 08 — Test Protocol

Unfakable proof. Prefer mock LLM + synthetic PDFs. Live VLM gated on API keys (existing pattern).

## Command SSOT

```bash
# Precision
cargo test -p edgequake-pdf --lib figure_filter
cargo test -p edgequake-api --test contract_spec049_figure_filter
cargo test -p edgequake-api --test contract_spec049_visual_regions
cargo test -p edgequake-api --test e2e_spec049_visual_regions

# Layout / pages (new)
cargo test -p edgequake-storage --lib page_layout
cargo test -p edgequake-api --test contract_spec128_page_layout
cargo test -p edgequake-api --test contract_spec128_prune_assemble

# pdf2md (sibling, after path patch)
cargo test -p edgequake-pdf2md --test spec049_corpus

# FE
cd edgequake_webui && pnpm exec playwright test e2e/spec128-layout-overlay.spec.ts

# Bundle
make spec128-proof   # NEW: unit + contract + clippy subset documented here
```

## Gates (CI)

| Gate | Assert |
|------|--------|
| G1 | No invented asset paths |
| G2 | Full-page `page-NNNN.png` never Drawing |
| G3 | Crop area ≤ 55% page |
| G4 | L0 wins L1 at DEDUP_IOU |
| G5 | Keyword chart not sole proposer |
| G6 | Labeled figure recall on 048 corpus — no regression > 1 fig |
| G7 | Unlabeled crop rate bounded (existing soft bound, tighten after prune) |
| **G-prune** | After filter, `\|figure_map\| == kept`; assemble hrefs == kept; mm-assets fig count == kept |
| **G-layout** | Layout `abandon`/logo not in final figure assets; L0 preserved |
| **G-layout-coord** | Synthetic page: known figure rect IoU ≥ 0.5 after PDF projection |
| **G-industrial** | Logo/stamp discarded from RAG; real diagram kept |
| **G-cost** | VLM calls/page ≤ budget when gates on (mock counts) |
| **G-overlay** | Playwright: toggle shows boxes; zoom 150% IoU of box vs fixture ≥ 0.8 in CSS |
| **G-rls** | Other workspace GET layout → 404/403 |
| **G-cascade** | Delete document → 0 page/region rows |

## Unit / contract matrix

| ID | Case | Expected |
|----|------|----------|
| T-prune-1 | 3 keep / 2 discard mock | assemble 3 hrefs |
| T-prune-2 | filter error | fail-open, all kept |
| T-prune-3 | include-from-pdf after prune | discarded not resurrected |
| T-wire-1 | LLM present, extract_figures | provider Some |
| T-wire-2 | `EDGEQUAKE_FIGURE_FILTER=0` | provider None |
| T-geo-1 | image aspect 12:1 | rejected |
| T-geo-2 | image area 0.004 | rejected at 0.008 gate |
| T-kind-1 | stamp Pass-1 | not in figure_map |
| T-conc-1 | 8 crops concurrency 4 | 8 Pass-1 results, order stable by path |
| T-page-1 | GET layout page 3 | only page 3 regions |
| T-page-2 | page 0 / 999 | 400 / 404 |
| T-norm-1 | PDF y-up → bbox_norm y-down | golden numbers |
| T-col-1 | two-column synthetic | ≥2 `column` derived |
| T-l0-1 | tagged figure + L2 miss | L0 box still overlay + kept |
| T-onnx-1 | missing weights | layout_status=skipped, ingest ok |
| T-onnx-2 | sha256 mismatch | failed + fail-open |

## Playwright

File: `edgequake_webui/e2e/spec128-layout-overlay.spec.ts`

1. **S (G-overlay):** mocked layout JSON + `overlay-letter.pdf` (CSS IoU ≥ 0.8). Coordinate unit only. Also: 11px class labels, empty `extracted` copy, click box with `asset_path` → markdown `data-layout-asset-focused`.
2. **R (live persisted):** `SPEC128_LIVE_*` env IDs → unmocked GET layout + ingested PDF (`R01–R05`).
3. **I (historical):** mock-geometry HIPO ingest — not the current live path.
4. **M (live mistral):** `pdf_data/*.pdf` + workspace `vision_llm_provider=mistral` / `mistral-small-latest`. Skip unless `E2E_LIVE_STACK=1` **and** `MISTRAL_API_KEY`. Primary (smallest PDF): poll layout (no KG wait), Layout on, CSS IoU ≥ 0.8 vs GET `bbox_norm` (`M01–M05`). Remaining PDFs: sequential, ≥1 persisted region each.
5. Open `/documents/{id}`, wait PDF.
6. Assert `data-testid="pdf-viewer"`.
7. Overlay toggle **Layout** off → 0 `pdf-layout-box`; `aria-pressed=false`.
8. Toggle on → ≥1 box; Figures chip; class labels.
9. Zoom in → box still overlaps fixture rect (CSS) on G-overlay; live asserts boxes remain visible.
10. Noise chip off → logo box absent (G-overlay).
11. Non-PDF document → no toggle.
12. `extracted` + 0 regions → empty copy. `failed` → toggle disabled.

## Fixtures

Keep SPEC-049 `ideas_*`, `hierar_*`, `lighrad_*`. **Add** synthetic multi-object page (logo + stamp strip + large diagram) under `specs/128-improve-pdf-parsing/fixtures/` (PDF generated in test or committed minimal PDF).

## make spec128-proof

Must fail if G-prune is unwired. Bundle:

- `rg` prune helper in `vision.rs` + overlay testid + ingest attach
- clippy `-D warnings` on `edgequake-pdf` and `edgequake-storage`
- unit: `figure_filter`, `page_layout`, `page_layout_storage`, persist, T-wire-2 env
- contract: `contract_spec049_figure_filter`, `contract_spec128_page_layout` (`--features postgres`)
- sibling pdf2md `text_blocks` (T-col-1) when the path patch crate exists
- Playwright `e2e/spec128-layout-overlay.spec.ts`: G-overlay IoU on fixture (always); live persisted when stack is up; live mistral/`pdf_data` only when stack **and** `MISTRAL_API_KEY`.

No “pass because filter unwired”. No HTML-harness overlay.

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- SPEC-049: [../049-improve-figure-extraction/004-acceptance-and-tests.md](../049-improve-figure-extraction/004-acceptance-and-tests.md)
