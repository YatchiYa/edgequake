# 01 — First Principles (LAW-128)

## Domain

A PDF has **no** native `Figure` / `Table` / `Chart` / `Paragraph` paint types ([ISO 32000](https://pdfa.org/resource/iso-32000-2/)). Visuals are Image XObjects, Form XObjects, paths, text, and optional StructTree roles. Layout overlay is a **display of classified geometry**. RAG figures are a **kept subset** of that geometry.

```ascii
  ISO paint / tags     →  propose regions     (truth of the file)
  Layout model         →  explain the page    (statistical, pinned)
  VLM filter           →  keep/discard assets (semantic, fail-open)
  Overlay              →  show all classes    (UX)
  figure_map / MD      →  index survivors     (RAG)
```

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-128-1 | **Cascade of truth** — L0 StructTree > L1 paint > L2 layout > L3 VLM. Lower layers never override a higher layer for the same visual (IoU ≥ `DEDUP_IOU`). | ISO tags beat statistics beat prompts |
| LAW-128-2 | **Split of duties** — overlay shows all classified regions (including `abandon` / logos); RAG `figure_map` shows kept figures only | Display ≠ index |
| LAW-128-3 | **Captions label** — caption text never invents a crop | SPEC-049 axiom |
| LAW-128-4 | **PDF user space is coordinate SSOT** — origin bottom-left, points, MediaBox + rotation. API derives `bbox_norm` (0–1, origin top-left). FE does not invent a third space | [12-coordinate-systems.md](12-coordinate-systems.md) |
| LAW-128-5 | **No AGPL in the product binary** — DocLayout-YOLO is AGPL-3.0 (research / self-hosted). Default L2: PP-DocLayoutV3 ONNX, Apache-2.0, SHA-256 pinned | License; GHCR |
| LAW-128-6 | **Columns are derived** — cluster paragraph/text boxes by x-overlap; `source=derived`. Detectors do not emit `column` | No fake class |
| LAW-128-7 | **L2 fail-open** — layout errors skip the gate, persist L0/L1, never fail ingest | Reliability |
| LAW-128-8 | **No invented asset paths; no full-page Drawing** — `page-NNNN.png` is viewer context only (G1/G2) | SPEC-049 G1/G2 |
| LAW-128-9 | **Constant changes require corpus Δ** — `MIN_*` / `MAX_*` / IoU / conf only move with G6/G7 | Non-flaky |
| LAW-128-10 | **DRY / SOLID** — one geometry module, one coord transform, one taxonomy mapper, one overlay component. Filter does not write PNGs; overlay does not re-detect; persist before bbox drop | Maintainability |
| LAW-128-11 | **Page is a first-class row** — `document_pages` 1:N `page_layout_regions`; not `document_artifacts` JSON; not `documents.metadata` | SPEC-091 grain |
| LAW-128-12 | **Lazy overlay fetch** — `GET .../pages/{n}/layout` per visible page; list endpoint returns counts only | 500-page PDFs |
| LAW-128-13 | **Prune is authoritative** — after successful filter, rebuild `figure_map`, drop empty pages, optionally delete discarded PNGs; assemble from kept set only | Close the loop |
| LAW-128-14 | **Filter default-on when vision LLM exists** — `EDGEQUAKE_FIGURE_FILTER=0` forces off | WP-1 |
| LAW-128-15 | **L2 always persists when enabled** — not “only if L0+L1 empty” (supersedes SPEC-049 P2). L2 **gates** L1; L0 always kept | Overlay needs full page |
| LAW-128-16 | **Pdfium vs ONNX isolation** — Pdfium on `spawn_blocking`; ONNX sessions not holding page locks; one `ort` session per worker (`Session::run` is `&mut self`) | Thread safety |
| LAW-128-17 | **Unfakable proof** — G-prune, G-layout-coord, Playwright overlay alignment, RLS cascade | Honest acceptance |
| LAW-128-18 | **Visual recovers what text cannot** — text-native tables stay markdown; visual `-table-` only when glyphs fail | SPEC-049 modality split |

## Env / config contract

| Key | Default | Meaning |
|-----|---------|---------|
| `extract_figures` | true | Existing vision extract flag |
| `EDGEQUAKE_FIGURE_FILTER` | unset (= on if LLM present) | `0` / `false` forces off |
| `min_image_area_frac` | start `0.008` | WP-2; raise only with G6 Δ |
| `max_figure_aspect` | `8.0` | WP-2 |
| `figure_filter_concurrency` | `4` | WP-4 |
| `max_figure_vlm_per_page` | `12` | Drop lowest-area first after geometry |
| `layout_onnx` / feature `layout-onnx` | **off** until gates pass | WP-6 |
| `layout_onnx_model_path` | pinned path | No runtime download in prod |
| `layout_onnx_model_sha256` | required when path set | Integrity |
| `layout_conf` | `0.25`–`0.4` | Detector |
| `layout_iou` | `0.3` | Match L1 to layout figure/chart/table |
| `layout_imgsz` | model card (800 PP-DocLayoutV3 / 1024 YOLO) | Match export |

## Authority order (same visual)

```ascii
  L0 StructTree box
       │  IoU ≥ DEDUP_IOU (0.25)
       ▼
  L1 paint proposal     →  DROP L1 (keep L0)
  L2 layout box         →  may GATE L1 if no L0; never drop L0
  L3 VLM                →  may DROP from figure_map; overlay region remains
```

## Cross-refs

- Why: [00-why.md](00-why.md)
- Ontology: [13-layout-taxonomy.md](13-layout-taxonomy.md)
- Coords: [12-coordinate-systems.md](12-coordinate-systems.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Parent cascade: [../049-improve-figure-extraction/001-first-principles.md](../049-improve-figure-extraction/001-first-principles.md)
