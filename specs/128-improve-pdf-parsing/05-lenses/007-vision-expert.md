# Lens 007 — Vision Expert

## Stake

Document layout detection is **not** generic COCO detection. Classes, letterbox vs stretch, and Y-axis after projection are the usual production bugs. L2 is a **gate + overlay explainer**, not the sole proposer (LAW-128-1, LAW-128-15).

## Default model (product)

**PP-DocLayoutV3 ONNX**, Apache-2.0, ~800×800 input (do **not** preserve aspect — match upstream preprocess: resize to square, ImageNet normalize). Output includes score, xyxy in **image pixels**, reading order.

Map 25 Paddle classes → canonical overlay classes in **one** function ([13-layout-taxonomy.md](../13-layout-taxonomy.md)).

## Rejected as default

| Model | Why not default |
|-------|-----------------|
| DocLayout-YOLO DocStructBench | **AGPL-3.0** — LAW-128-5; optional research extractor behind the same trait |
| Unpinned HF download | Prod integrity |
| LayoutLMv3 as detector | Wrong task / heavier |

## Geometry gates (deterministic, before VLM)

| Gate | Invariant |
|------|-----------|
| Min page-area **images** | `MIN_IMAGE_AREA_FRAC` start **0.008** (raise only with G6) |
| Max page-area | keep `MAX_AREA_FRAC = 0.55` |
| Max aspect | `MAX_ASPECT ≈ 8.0` (rules already handled as H/V rules) |
| Min edge | 24 pt / 24 px |

Image branch today skips min area so logos reach VLM — that only works if filter is wired **and** prunes. After WP-0/1, still keep a **low** area floor so 1% logos can overlay as noise if L2 labels them `abandon`/`seal` even when no PNG is written.

## L2 as gate (not sole proposer)

```ascii
  L1 proposal kept if
     L2 off
     OR L0 same visual
     OR IoU with layout figure|chart|table ≥ LAYOUT_IOU (0.3)
     OR Form-backed + FORM_LAYOUT_EXEMPT (vector figures layout often misses)
```

Captions (`figure_title`, `vision_footnote`) **label**, never create crops.

## Coordinate pipeline (vision)

```ascii
  page PNG (raster, top-left)
    → model input (letterbox or stretch — MUST match export)
    → xyxy in input space
    → unscale to raster px
    → raster px → PDF user space (bottom-left) using page MediaBox + rotation
    → persist bbox_pdf
```

Golden test: synthetic page with known figure rect, IoU ≥ 0.5 after projection (G-layout-coord). See [12-coordinate-systems.md](../12-coordinate-systems.md).

## Columns

Not a PP-DocLayout class. Cluster `paragraph` boxes with overlapping x-ranges (gap threshold in pts). `source=derived`. Two-column papers are the acceptance fixture.

## Tables (modality)

Selectable cell glyphs → markdown, **no** `-table-` PNG. Image-of-table or failed lattice → visual crop. Overlay still shows `table` from L2 even when no PNG.

## Cross-refs

- Taxonomy: [../13-layout-taxonomy.md](../13-layout-taxonomy.md)
- OCR: [009-ocr-expert.md](009-ocr-expert.md)
- zz-raw WP-6: [../zz-raw.md](../zz-raw.md)
