# 09 — Acceptance

## Precision release

| # | Criterion | Proof |
|---|-----------|-------|
| A1 | Filter runs on ingest when vision LLM exists | T-wire-1; log `figure filter complete` |
| A2 | `EDGEQUAKE_FIGURE_FILTER=0` disables | T-wire-2 |
| A3 | After success, figure_map == kept | G-prune |
| A4 | Markdown Drawing hrefs == kept | assemble contract |
| A5 | Logo/stamp not RAG figures on industrial fixture | G-industrial |
| A6 | 048 corpus recall not worse than >1 labeled fig | G6 |
| A7 | G1/G2/G3 still green | existing e2e_spec049 |

## Layout + overlay release

| # | Criterion | Proof |
|---|-----------|-------|
| B1 | `document_pages` + regions persist; delete cascades | G-cascade |
| B2 | GET layout returns bbox_pdf + bbox_norm | contract OpenAPI |
| B3 | Overlay toggle draws boxes on current page | G-overlay |
| B4 | Zoom 50–300% alignment | G-overlay |
| B5 | L2 fail / missing model does not fail ingest | T-onnx-1 |
| B6 | L0 not dropped when L2 disagrees | T-l0-1 / G-layout |
| B7 | Columns derived on two-column fixture | T-col-1 |
| B8 | Workspace isolation | G-rls |
| B9 | Default ONNX Apache + sha256 | system lens + CI pin file |
| B10 | AGPL weights absent from GHCR image | image file list / docs |

## Product narrative (PO)

See [05-lenses/001-product-owner.md](05-lenses/001-product-owner.md). Manual: upload paper, overlay on, logo as noise, diagram as figure.

## Done when

`make spec128-proof` green on precision; layout extras green before enabling `layout-onnx` in product default. Overlay MVP (L0/L1) may ship with schema before ONNX.

## Cross-refs

- Test: [08-test-protocol.md](08-test-protocol.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
