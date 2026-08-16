# 13 — Layout Taxonomy

One mapper. Frontend never switches on model class ids. LAW-128-6: `column` is derived.

## Canonical overlay classes (product)

| Canonical | Overlay chip | Typical RAG asset? |
|-----------|--------------|--------------------|
| `figure` | Figures | Yes if kept by L3 |
| `chart` | Charts | Yes if kept |
| `table` | Tables | PNG only if visual modality |
| `formula` | (Figures or Other) | Usually no PNG |
| `paragraph` | Paragraphs | No |
| `title` | (meta) | No |
| `caption` | (meta) | Label only |
| `header` | Noise (optional) | No |
| `footer` | Noise | No |
| `column` | Columns | No (`source=derived`) |
| `abandon` | Noise | Never |
| `other` | (meta) | Fail-open |

`RegionSource`: `l0_struct` | `l1_paint` | `l2_layout` | `l3_vlm` | `derived`.

L3 `FigureKind` (bar_chart, logo, …) may live in `extra.figure_kind` on a region or only on the filter manifest. Overlay class stays canonical (`chart` vs `abandon`).

## PP-DocLayoutV3 → canonical (default L2)

Apache-2.0. 25 classes ([PaddleX layout analysis](https://github.com/PaddlePaddle/PaddleX/blob/develop/docs/module_usage/tutorials/ocr_modules/layout_analysis.en.md)):

| Paddle class                                                                             | Canonical   |
| ------------------------------------------------------------------------------------------| -------------|
| `image`                                                                                  | `figure`    |
| `chart`                                                                                  | `chart`     |
| `table`                                                                                  | `table`     |
| `display_formula` / `inline_formula`                                                     | `formula`   |
| `text` / `vertical_text` / `abstract` / `algorithm` / `reference_content` / `aside_text` | `paragraph` |
| `doc_title` / `paragraph_title`                                                          | `title`     |
| `figure_title` / `vision_footnote` / `formula_number`                                    | `caption`   |
| `header` / `header_image`                                                                | `header`    |
| `footer` / `footer_image` / `footnote` / `number`                                        | `footer`    |
| `seal`                                                                                   | `abandon`   |
| `content` (TOC)                                                                          | `other`     |
| `reference`                                                                              | `other`     |
| unknown                                                                                  | `other`     |

Input size: **800×800 stretch** (community ONNX examples). Pin SHA-256 of the exact `.onnx` file in repo `models/` or ops path.

## DocLayout-YOLO DocStructBench → canonical (research only)

AGPL-3.0 — **not in product binary** (LAW-128-5). If a self-hosted extractor is plugged into the trait:

| YOLO class | Canonical |
|------------|-----------|
| `figure` | `figure` |
| `table` | `table` |
| `plain_text` / `plain text` | `paragraph` |
| `title` | `title` |
| `figure_caption` / `table_caption` / `formula_caption` | `caption` |
| `abandon` | `abandon` |
| `isolate_formula` | `formula` |
| `table_footnote` | `footer` |

No native `chart` vs `figure` — both `figure` unless L3 kind refines `extra`.

**Do not** trust ONNX metadata `names` blindly (upstream mapping bugs). Pin a table next to the sha256.

## Derived `column`

```ascii
  take regions class=paragraph (and title if needed)
  cluster by x-overlap (IoU_x ≥ threshold, e.g. 0.6)
  union bbox per cluster → class=column source=derived
  skip if only one cluster spanning > 80% page width
```

## L0/L1 without L2

| Source | Canonical |
|--------|-----------|
| StructTree Figure | `figure` |
| StructTree Table | `table` |
| ObjectCluster Image/Form | `figure` (L3 may reclassify overlay extra) |
| ObjectCluster table cue | `table` |

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Vision: [05-lenses/007-vision-expert.md](05-lenses/007-vision-expert.md)
- Intake model list: [zz-raw.md](zz-raw.md) (YOLO links kept for research)
