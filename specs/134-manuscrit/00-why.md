# 00 — Why SPEC-134

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) — handwritten / MFD-scanned technical notebooks fail
product RAG quality even when Vision is “healthy.” Operator side-by-side shows:

1. **Pass-B analyzing a scribble crop** while the page body (prose + implicit table)
   is not faithfully transcribed.
2. **Hand graphics shredded into atomic crops** (axis tick digits, single bars) with
   geometric narration — the graphic is never treated as one semantic unit.

Hard rule: never name the trigger file; never quote its content in specs or fixtures.

## Product WHY

```ascii
  Operator: “I uploaded a scanned notebook. The viewer shows Vision Analysis
             of tiny crops (scribbles, axis numbers, one bar) —
             where is my table / my whole histogram?”
       │
       ▼
  Today:
       Pass-A prompt assumes PRINT + English Acc
       Render DPI optimized for cost, not thin ink
       Region extract + Pass-B amplify FRAGMENTS of graphics
       Scanner OCR / EdgeParse look “textual” but lie
              │
              ▼
         Indexed MD is thin / fluent-wrong
         UI looks “smart” on atoms of the chart
         Query / KG miss facts that only exist in the whole graphic
```

## Five WHYs

1. **Why is the notebook useless in RAG?** Indexed markdown does not carry the page’s
   readable facts (aligned numbers, labels, chart series).
2. **Why doesn’t Vision capture them?** Pass-A is tuned for printed scientific PDFs;
   manuscript pages need page-as-unit fidelity, higher resolution, and a different prompt.
3. **Why does the UI show confident Vision Analysis cards?** Pass-B / figure specialize
   runs on crops; scribbles **and** chart fragments (ticks, single bars) get narrated
   as if they were the whole figure.
4. **Why aren’t crops suppressed?** Filter taxonomy targets logos/stamps/signatures for
   *print* corpora; it does not enforce **graphic-as-unit** (LAW-134-16) or page-first
   transcription before fragment theater.
5. **Root cause:** EdgeQuake has **one print-centric vision profile**. There is no
   `PageModality` routing that changes render, prompt, asset policy, and honesty signals
   for handwritten scans — and region extract treats paint atoms as figures.

## Job to be done

> When I ingest a handwritten or MFD-scanned technical page, EdgeQuake treats the
> **entire page** (and each hand graphic) as the primary visual unit, transcribes text
> and implicit tables with fidelity (source language, uncertainty markers), digitizes
> hand charts **as wholes** (series, axes, Key values — not tick-crop geometry), keeps
> the full-page image for review, and shows an honest modality / confidence chip — so
> RAG and the side-by-side viewer reflect the page, not a fragment monologue.

## Success criteria

1. Manuscript-class pages select `ManuscriptProfile` (DPI floor + raised pixel cap).
2. Pass-A uses manuscript system prompt SSOT (no forced English paraphrase; `[?]` for unreadables).
3. Implicit tables and color-series hand charts land as GFM / Key values when readable;
   **no** Pass-B gallery of axis-tick / single-bar crops for that graphic.
4. Pass-B does not dominate UX with tiny noise-crop or chart-fragment analyses when
   page modality is manuscript.
5. UI shows modality chip + confidence; low confidence is visible, not silent.
6. Synthetic gold fixtures pass CER/WER + table F1 + chart recall gates in [08-test-protocol.md](08-test-protocol.md).
7. EdgeParse Auto fast-path never replaces Vision on manuscript-classified pages.
8. Pass-A actually receives the manuscript long-edge raster (pdf2md
   `max_rendered_pixels` forwarded); empty pages do not grow crop galleries
   (LAW-134-20).
9. Mixed documents convert print pages with the Acc print profile and
   manuscript pages with the MS profile (no whole-doc MS prompt on figures).

## Non-goals (product)

- Classical HTR engine in the product binary (v1)
- Human-proof archival accuracy claims
- Replacing pdfium / Pass-A architecture
- Publishing or quoting trigger document content
- Re-implementing SPEC-133 delimiter fix (depend on it)

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Intake: [zz-raw.md](zz-raw.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
