# 01 — First Principles (LAW-134)

## Domain

A handwritten / MFD-scanned page is **not** a born-digital PDF with a trustworthy
glyph stream. Meaning lives in **pixels**: stroke geometry, alignment, ink color,
arrows/braces, strikeouts, and grid paper. Pass-A VLM “page OCR” can recover that
meaning only if render resolution, prompt contract, and asset policy match the class.

```ascii
  Scanner PDF
       │
       ├─ JPEG page image     → truth of the page (primary)
       ├─ CCITT / OCR tiles   → optional hint (often lies)
       └─ EdgeParse text      → useless or harmful on pure scans
              │
              ▼
  PageModality
    print | manuscript | mixed
              │
              ▼
  ManuscriptProfile (when manuscript|mixed-hand)
    Render  +  Prompt  +  Assets  +  Honesty
```

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-134-1 | **Page-as-unit** — For manuscript modality, the full page image is the primary visual unit. Crop specialize must not replace page transcription as the RAG SSOT. | Operator “today” failure: scribble crop narrative |
| LAW-134-2 | **Fidelity over fluency** — Preserve source language, orthography, numbers, symbols. Do not translate, modernize, or “fix OCR.” Unreadables → `[?]`. Never invent. | VLM hallucination risk; Acc English pin is wrong for notebooks |
| LAW-134-3 | **Class-routed render** — Manuscript DPI floor and `max_rendered_pixels` floor override large-PDF adaptive downscale unless operator explicitly opts out. | Thin ink / ticks need ≥~250–300 DPI effective |
| LAW-134-4 | **Display ≠ index** — Full-page PNG always available for review; indexed MD may omit strikeouts/doodles; UI must not present noise-crop analysis as the page answer. | SPEC-049 / SPEC-128 display≠index |
| LAW-134-5 | **Color is data** — On hand charts, ink/color series are semantic; MD must preserve series identity when readable. | Color-coded histograms |
| LAW-134-16 | **Graphic-as-unit** — A hand chart / multi-panel histogram / log plot is one semantic object. Do **not** promote axis-tick glyphs, single bars, or legend fragments to Pass-B specialize. Pass-A digitizes the whole graphic (axes + series + Key values). | Operator “graphic analyzed as atoms” failure |
| LAW-134-6 | **Implicit structure** — Spatial alignment → GFM tables; braces/arrows → explicit relation lists with **delimiter-safe** names (SPEC-133). | Implicit tables; diagram joins |
| LAW-134-7 | **Fail honest** — Low confidence / empty Pass-A surfaces in status + UI. Never silent EdgeParse garbage for manuscript class. | Reliability |
| LAW-134-8 | **DRY / SOLID** — One `PageModality`, one `ManuscriptProfile`, one prompt SSOT in `vision_prompts.rs`. Classifier does not assemble MD. Filter policy *reads* modality; does not re-detect. UX reads persisted fields only. | Maintainability |
| LAW-134-9 | **No classical HTR in v1 product binary** — TrOCR/Kraken/AGPL weights stay research / optional L2 (same posture as SPEC-128 ONNX). | License + ship surface |
| LAW-134-10 | **Synthetic gold only** — Fixtures never contain trigger PII/content; metrics move only with corpus Δ. | Privacy + non-flaky |
| LAW-134-11 | **Prompt select is SSOT** — `pass_a_system_prompt_for(modality)` returns print vs manuscript constant; API overrides still max 32 KiB. | SPEC-015V |
| LAW-134-12 | **Manuscript skips EdgeParse fast-path** — Auto backend must not short-circuit to text extract when modality is manuscript. | SPEC-038 interaction |
| LAW-134-13 | **Uncertainty is first-class** — Persist optional `transcription_confidence`; UI chip required when modality ≠ print. | Honesty |
| LAW-134-14 | **Pass-B budget under manuscript** — Prefer page MD + full-page drawing; suppress or de-prioritize tiny low-ink crops **and** chart-fragment crops (ticks, single bars) from specialize. | Fix priority inversion + graphic atomization |
| LAW-134-15 | **Unfakable proof** — Classifier unit tests, prompt/DPI contracts, CER/WER gold, Playwright modality chip; assert zero Pass-B cards on axis-tick crops for MS chart fixture. | Acceptance |
| LAW-134-17 | **Verify before trust** — Manuscript Pass-A output gets a Judge-and-Refine check when confidence is low; uncalibrated confidence is decoration. | SOTA: MinerU2.5-Pro, OCR-Agent |
| LAW-134-18 | **Frontier VLM for MS** — Default MS vision model should be a frontier VLM (GPT-5 / Opus 4.7 / Gemini 3 class); small/legacy VLMs are not MS-capable. | SOTA: IAM leaderboard Aug 2026 |
| LAW-134-19 | **Consensus over single-pass** — Optional two-VLM agreement on MS pages; low entropy → accept, high → route to stronger model. | SOTA: Consensus Entropy CVPR 2026 |
| LAW-134-20 | **Full-page raster is the VLM input** — For a manuscript page, Pass-A must see the whole-page render (long-edge floor), not PDF XObject tiles. Crops may exist on disk for the dual-pane viewer; they must not become the RAG SSOT or the only image the model sees. Empty Pass-A must not grow fig/chart hrefs (that defeats empty-page retry). | Operator screenshot: placeholder + crop gallery; photo-to-VLM recovers the page |

## Env / config contract (normative intent)

| Key | Default | Meaning |
|-----|---------|---------|
| `EDGEQUAKE_PDF_MANUSCRIPT_DPI` | `300` (clamp 200–400) | DPI floor when modality is manuscript |
| `EDGEQUAKE_PDF_MANUSCRIPT_MAX_PIXELS` | `3600` | `max_rendered_pixels` floor for manuscript |
| `EDGEQUAKE_PDF_PAGE_MODALITY` | unset (auto) | Force `print` \| `manuscript` \| `mixed` for tests |
| `EDGEQUAKE_PDF_MANUSCRIPT_SKIP_EDGEPARSE` | `true` | Block Auto text fast-path on manuscript |
| `EDGEQUAKE_PDF_MANUSCRIPT_VERIFY` | `true` | Judge-and-Refine pass when confidence low (WP-9) |
| `EDGEQUAKE_PDF_MANUSCRIPT_CONSENSUS` | `false` | Two-VLM agreement for confidence (WP-11) |
| `EDGEQUAKE_VISION_MODEL` | existing | Recommend frontier VLM for MS (WP-10) |
| `EDGEQUAKE_FIGURE_FILTER` | existing | Manuscript policy: do not discard signature-like crops *or* skip filter for MS pages (WP-4 chooses one SSOT) |

## SOLID / DRY checklist

| Principle | Application |
|-----------|-------------|
| S | Classifier → modality only; Profile → render/prompt/assets; Assembler unchanged except prompt inject |
| O | New modality without rewriting print Pass-A |
| L | ManuscriptProfile substitutes RenderProfile without breaking VisionConversionConfig |
| I | UX depends on persisted modality/confidence, not on re-running VLM |
| D | pdf_processing depends on edgequake-pdf modality API, not ad-hoc env parsing in three crates |
| DRY | Prompt text only in `vision_prompts.rs`; DPI math only in one profile helper |

## Cross-refs

- Why: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
- Parent OCR: [../128-improve-pdf-parsing/05-lenses/009-ocr-expert.md](../128-improve-pdf-parsing/05-lenses/009-ocr-expert.md)
- Delimiter: [../133-kv-error/](../133-kv-error/)
