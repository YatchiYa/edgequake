# zz-raw — Intake (not the contract)

> Structural assessment only. **Do not** name source files or quote / transcribe page content.

## Document class (observed)

| Axis | Observation |
|------|-------------|
| Producer | MFD / office scanner PDF (image-primary pages) |
| Pages | Small multi-page set (order of 4) |
| Geometry | A4 MediaBox; **mixed portrait / landscape** across pages |
| Paint | Large JPEG RGB page images + many tiny CCITT bi-level tiles (scanner OCR residue) |
| Text layer | Sparse extractable glyphs — **not** a reliable reading-order source |
| Ink | Low–medium ink fraction on sparse note pages; high density on graph-paper / plot pages |
| Modalities | Cursive prose, **implicit tables** (alignment without ruled grids), hand charts with **color-as-series**, brace/arrow joins, strikeouts, faint annotations, bleed-through, doodles, rotated margin labels |
| Language | Non-English technical notebook (diacritics, local decimal commas) + Greek / math symbols |

## “What we get today” (product failure modes)

### Failure A — Scribble crop theater

Side-by-side viewer evidence (operator screenshot, abstract):

```ascii
  LEFT: full handwritten page (prose + table + sketches)
  RIGHT: Pass-B / multimodal “Vision Analysis” of a TINY crop
         → invents arrows / “technical drawing” narrative
         → ignores high-value page body (text + implicit table)
```

### Failure B — Graphic shredded into atomic crops

Hand-drawn multi-panel chart page (histograms / axes / color blocks). Viewer shows:

```ascii
  LEFT:  whole graphic = header + N stacked hand charts (one semantic unit)
  RIGHT: gallery of MICRO-crops
           axis tick digits ("50", "1100", …)
           single bar / box fragment
         → each crop gets geometric “lines / frame” narration
         → NO chart-as-unit: no series-by-color, no axis binding, no KV table
```

| Defect | First-principles read |
|--------|----------------------|
| Scope failure | System optimizes a **crop** instead of the **page-as-unit** |
| Graphic atomization | Chart meaning is **holistic**; tick digits / one bar are not the graphic |
| Hallucination | Low-information scribble or fragment gets fluent English “analysis” |
| Priority inversion | Noise / fragment regions indexed / shown; structured data under-served |
| Language loss | Source-language notebook not preserved as primary transcript |

Concrete product WHY for LAW-134-1 (page-as-unit), **LAW-134-16 (graphic-as-unit)**,
LAW-134-4 (display ≠ index), LAW-134-5 (color is data), LAW-134-14 (Pass-B budget).

## Pipeline touchpoints (as-is)

1. Upload → `TaskType::PdfProcessing`
2. Adaptive DPI (default 150, down to 96 for large docs) + `max_rendered_pixels=2000`
3. Pass-A: `RAG_PAGE_VISION_SYSTEM_PROMPT` (English pin, print chart rules)
4. Figure filter may discard `signature` / `scan_artefact`
5. Pass-B multimodal analyze on `<drawing/>` / crops
6. Insert → KG (arrow-heavy names → SPEC-133 risk)

## Privacy / fixtures rule

- Do **not** commit the operator’s private scan into the repo.
- Gold corpus = **synthetic** pages under `fixtures/` with abstract labels.
- Specs and tests may refer to modality classes, never to source filenames or transcribed strings from the trigger.

## Links

- Contract starts at [00-why.md](00-why.md)
- Laws: [01-first-principles.md](01-first-principles.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
