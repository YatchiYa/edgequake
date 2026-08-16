# 00 — Why SPEC-128

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) — figure extraction is **recall-heavy**; the VLM filter classifies but historically does not prune the authoritative `figure_map`. Product addition: persist **page layout** so the PDF viewer can overlay figures, charts, classified objects, paragraphs, and columns.

## Product WHY

```ascii
  User: “Show me where the figure / paragraph / column is on the original page.”
  Operator: “Why did GraphRAG index this logo as a figure?”
       │
       ▼
  Today:
       L0/L1 propose almost every Image XObject (≥24 pt)
       FigureFilter (if it ran) writes a manifest and stops
       figure_map unchanged → markdown + mm-assets keep noise
       WrittenFigureAsset.bbox dropped after IoU
       PDF pane is a dumb page viewer (react-pdf, no boxes)
              │
              ▼
  Blind spots:
       1. Control loop open (classify ≠ prune)
       2. Filter never wired on ingest
       3. Page is not a row — nowhere to store layout
       4. Overlay has no API and no coordinate SSOT
```

## Five WHYs

1. **Why do logos become RAG figures?** L1 is a conservative proposer; images skip `MIN_AREA_FRAC` so small ornaments reach disk.
2. **Why doesn’t the VLM drop them?** `FigureFilter::run` writes `figure_filter_manifest.json` and **does not rebuild `figure_map`**. Worse: `figure_filter_provider` is never set on ingest, so Pass-1 **does not run at all**.
3. **Why can’t the viewer draw boxes?** Bboxes are PDF-space tuples used for IoU then discarded. No `document_pages` table. No `GET .../layout`.
4. **Why not dump layout into `documents.metadata` or `document_artifacts`?** Overlay is **per-page**, RLS-scoped, joinable to mm-assets. A closed CHECK JSON blob is the wrong grain (SPEC-091).
5. **Root cause:** The pipeline optimized for **recall of paint** and **page OCR markdown**. It never closed the semantic prune loop, and it never treated **page geometry** as a first-class persisted product.

## Job to be done

> When I open a processed PDF, I can toggle a layout overlay that shows figures, charts, tables, paragraphs, and columns **on the original page**, aligned under zoom; GraphRAG indexes only **kept** figure assets; discarded logos still appear on the overlay as noise, not as Drawing targets.

## Success criteria

1. After a successful filter, `|figure_map| == kept count` (G-prune). Assemble and mm-assets match kept set.
2. Filter runs on ingest when a vision LLM exists unless `EDGEQUAKE_FIGURE_FILTER=0`.
3. Image proposals obey page-area + aspect gates (corpus-locked constants).
4. Each PDF page is a row; each region is a row; `GET /documents/{id}/pages/{n}/layout` returns PDF-space SSOT + derived `bbox_norm`.
5. Overlay toggle on `PDFViewer`; boxes track zoom via measured CSS size; L0 never overridden by L2/L3.
6. Default L2 weights are Apache-licensed and SHA-256 pinned; AGPL DocLayout-YOLO is not in the product image.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Taxonomy: [13-layout-taxonomy.md](13-layout-taxonomy.md)
