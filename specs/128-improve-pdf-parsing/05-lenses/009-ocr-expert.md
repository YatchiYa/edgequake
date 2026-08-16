# Lens 009 — OCR Expert

## Stake

Pass-A **page OCR** (full-page VLM markdown) stays the reading-order text SSOT. Layout overlay does **not** replace OCR. L2 boxes explain **where** blocks are; Pass-A (and pdfium text) explain **what they say**.

## Split

| Need | Channel |
|------|---------|
| Body text for RAG chunks | Pass-A markdown + page markers (existing) |
| Text-native tables | Markdown/HTML from text extract — **no** table PNG |
| Scanned page (no glyphs) | Pass-A VLM; L1 images; L2 still runs on raster |
| Figure meaning | Kept crop + optional Pass-2 / Pass-B analyze |
| Overlay paragraphs | L2 `text` boxes — **not** OCR tokens |

Do **not** project OCR character boxes in v1 (no PDF.js text-quad highlight beyond native TextLayer).

## Reading order

PP-DocLayoutV3 emits `read_order`. Persist as `reading_order`. Overlay does not reorder markdown. Future chunker may use it — **non-goal v1**.

## Headers / footers / page numbers

Map to `header` / `footer` / `abandon`. Overlay Noise chip. Do **not** index as figures. Pass-A may still transcribe them in markdown (existing); layout does not strip markdown.

## Stamps / seals

PP class `seal` → canonical `abandon`. Overlay noise. L3 discard kinds Stamp/Signature align with this for **crops**, not for page-level seals without a crop.

## Failure modes

| Case | OCR | Overlay |
|------|-----|---------|
| Skewed scan | Pass-A quality drops | L2 V3 claims robustness; still fail-open |
| Two-column | Pass-A may snake columns | derived `column` boxes show structure |
| Vertical text | Pass-A | class `other` or paragraph if mapped |

## Cross-refs

- Vision: [007-vision-expert.md](007-vision-expert.md)
- Taxonomy: [../13-layout-taxonomy.md](../13-layout-taxonomy.md)
- Modality split: [../049-improve-figure-extraction/001-first-principles.md](../../049-improve-figure-extraction/001-first-principles.md)
