# 11 — Honest assessment

**Date: 2026-08-22 (Slice E in tree). Do not read the README status board as “the notebook now transcribes.”**

## Verdict

Slice E **closes a real convert-channel bug** (empty placeholder + crop gallery,
Pass-A stuck at 2000px, EdgeParse-before-classify). That is necessary. It is
**not sufficient** for grounded manuscript RAG.

Three evidence layers, ranked:

| Layer | Strength | What it actually shows |
|-------|----------|------------------------|
| Convert mechanics | **Strong** (code + unit/contract tests) | 3600px is forwarded; empty/MS assemble injects zero `fig-` hrefs; classify before Auto; escalate on placeholder *containment*; MS ImageGuard floor 2000; mixed groups keep Acc English on print pages |
| Python page-as-unit study | **Weak for quality** | Same model, only the *input* changes. Metric was `empty_rate`. Everything was non-empty. That is not CER, not recall, not “readable French on 4 pages” |
| Live EdgeQuake convert of the trigger PDF | **Not repeated after Slice E** | Last production-shaped run is **2026-08-20**: classifier/profile wired; markdown still ~0% grounded on pages 2–4; KG ingested confabulation |

A photo-to-VLM path can produce *some* tokens. EdgeQuake’s failure in the
operator screenshot was **assemble + unforwarded pixels**, not “no VLM.”
Fixing that channel does not retract the 2026-08-20 grounding audit.

## What this pack claims

- A first-principles diagnosis of print-centric Pass-A on handwritten / MFD pages.
- Slice E: **page-as-unit convert policy** (LAW-134-20) so the VLM’s input is the
  full-page raster, not PDF XObject tiles, and empty Pass-A cannot grow a crop
  gallery that defeats retry.
- Contracts that pin those mechanics. Print Acc path must stay byte-identical.

## What this pack does not claim

| Claim | Reality |
|-------|---------|
| Archival-grade HTR | VLM CER varies; expect `[?]` and human verify |
| Color chart digitization perfect | Tick/log grids remain hard |
| Trigger notebook is now good markdown | **Not reconverted through EdgeQuake after Slice E** |
| 3600px proven better than 1024px on this doc | Ablation `empty_rate` was 0.0 at **all** long-edges; char counts are not monotonic |
| Task-aware MS prompt proven vs print (DISCO) | Print prompt also non-empty; it was **more verbose** and showed English hits |
| Crop gallery is empty / unusable | Crops also non-empty; `frenchish_pages` 0 vs 1 is a one-page heuristic quirk |
| Per-page DPI inside pdf2md | pdf2md 0.9.11 still **ignores `dpi`**; long-edge is `max_rendered_pixels` |
| UX modality chip | Acceptance must #7 — **not built** |
| Gold CER in CI | No synthetic gold, no anchor list (`anchor_recall: null`) |
| Verify pass stops KG poisoning | WP-9 is wired **fail-open**; never shown to catch the 2026-08-20 confabulation on this doc |

## Slice E Python study (2026-08-22) — what the numbers mean

Private 4-page PDF; gitignored `study/out/`; **no page quotes**. Model:
`mistral-small-latest` (OpenAI project key returned 401; not a scientific no-go).
Go script: `empty_rate < 0.5` on whole-page ≥2000 + MS → `go_for_rust: true`.
That bar is **too low**.

Measured on the 24 page files (no content):

| Fact | Number |
|------|--------|
| Whole-page MS empty_rate (1024 / 2000 / 3600 PNG / 3600 JPEG) | **0.0** all |
| Crop gallery empty_rate | **0.0** |
| Print-prompt empty_rate | **0.0** |
| Whole-page MS wrapped in ` ``` ` fences (prompt forbids fences) | **16 / 16** |
| Whole-page MS `[?]` marks | **0** (crops page 1 had 2) |
| `frenchish_pages` (need ≥2 of le/la/les/des/une/pas/pour/dans) | **1 / 4** at every whole-page MS condition; **0 / 4** for crops |
| Pages 3 and 4 French-function-word hits | **0** at every resolution |
| Whole-page MS body size | **~185–900 chars**/page — a dense handwritten page is not “done” at that length |
| Longest single output | print-prompt page 3 **~1500 chars** (verbosity ≠ grounding) |

**Honest reading:** the VLM returned *something* for a whole page, including at
1024px. The operator screenshot (placeholder + crops) is therefore a **pipeline
bug**, not “1024px is unscorable.” OmniDocBench v1.5 (notes 72 DPI → 200 DPI)
still justifies a **2000px ImageGuard floor** for harder notes; this notebook
did not demonstrate that floor.

Crops are the wrong *unit* for RAG SSOT (LAW-134-20) even when they emit tokens.
Do not cite `frenchish_pages` as a quality score.

## Measured evidence (2026-08-20, live API convert, same model family)

This remains the only **end-to-end** quality audit. Slice E does not overwrite it.

Slice A/B wiring in that run:

| Mechanism | Evidence | Verdict then | Now (Slice E) |
|-----------|----------|--------------|---------------|
| Heuristic classifier | log `modality=manuscript` | ✅ Wired | Still wired; **per-page groups** added |
| Viewer PNG 3600px | 2546×3600 | ✅ Viewer | Pass-A long-edge is now **forwarded** (was the hole) |
| Pass-B suppression | `suppressed=10` | ✅ Wired | Unchanged |
| MS Pass-A prompt | print EN pin used | ❌ Not routed | ✅ Routed per convert group (contract) |
| Crop theater in markdown | 11 strip narrations | ❌ Assemble | ✅ Empty/MS assemble **must not** inject `fig-` (unit tested; **not live-rechecked**) |

Page-by-page grounding (visual vs stored markdown, 2026-08-20):

| Page | Grounded? |
|------|-----------|
| 1 | ~30% (dense ink block accurate; mémento sections dropped) |
| 2 | ~0% (invented histograms) |
| 3 | ~0% (invented tables) |
| 4 | ~0% (wrong-domain fabrication) |

Keyword census on **53 125** chars: real technical anchors **0**; fabricated
tokens dominate. Downstream: **136 entities / 106 relationships** from
confabulated text.

Confabulation reproduced **outside** the pipeline (direct API, same page render)
on mistral-small, mistral-medium, qwen3.6:35b. Failure is region-selective:
dark ink reads; light pencil is dropped or replaced by prior.

The 2026-08-22 study bodies are **~50× shorter** than that poisoned convert.
Shorter is not automatically better — it can be omission. Nobody has scored
Slice E output against the page image.

## Residual risks (still true)

1. **Hallucination under fluency** — the dominant remaining failure. Slice E
   does not add a new grounding oracle.
2. **Verify fail-open** — judge errors leave invented text indexable unless
   `grounding:low` actually fires.
3. **Classifier false print** — lying OCR + image-primary miss → EdgeParse skip
   is vetoed only when the heuristic (or env) marks MS-like.
4. **Cost / latency** — 3600px + JPEG still costs; concurrency caps remain.
5. **Acc conflict** — mixed docs now split groups; homogeneous print must stay
   byte-identical (e2e guard exists; no Acc re-score this slice).
6. **SPEC-133** — arrow-heavy names still a peer spec.
7. **Uncalibrated confidence** — heuristic scores are not probabilities.
8. **pdf2md `dpi` unused** — operators who think `EDGEQUAKE_PDF_DPI=300`
   rasterizes at 300 DPI are still wrong; long-edge pixels are the knob.

## SOTA alignment (honest)

| Technique | Status |
|-----------|--------|
| Page class → render + prompt + asset policy | Slice E **policy** in convert path |
| OmniDocBench notes vs print pipelines | Motivates VLM page-as-unit; **not** our gold |
| OmniDocBench v1.5 200 DPI notes | Motivates ≥2000px floor; **not measured** as a win on this 4-pager |
| DISCO task-aware prompting | Motivates MS vs print prompt; **this ablation did not show MS ≫ print** |
| Judge-and-Refine | WP-9 **wired**, fail-open, mock e2e only |
| Consensus / two-VLM | WP-11 **not built** |
| Classical HTR | Non-goal (LAW-134-9) |
| IAM / CodeSOTA CER | Vendor/community numbers — **not** product CER (LAW-134-10) |

## Recommendation

1. **Do not market Slice E as “manuscript quality shipped.”** Market it as:
   Pass-A finally sees a full-page raster; empty pages no longer become crop
   galleries; EdgeParse cannot skip Vision on MS-like pages.
2. **Re-convert the private notebook through EdgeQuake** (not the Python
   harness) and repeat the 2026-08-20 visual audit: placeholder gone? fig hrefs
   gone? anchors present? invented tables gone? Until that run, the screenshot
   bug is **fixed in code, unverified in product**.
3. **Do not use `empty_rate` as a quality gate.** Need private gold anchors
   (hashed filenames only) or a human scorecard. The Python `go_for_rust`
   predicate over-called.
4. **Treat mistral-small as insufficient for this class** until proven
   otherwise — 2026-08-20 already showed invention on whole-page renders.
   WP-10 env routing exists; a stronger vision model is an ops choice, not a
   code change.
5. Keep WP-9 default-on but **do not trust fail-open** for belief-store
   admission; the quarantine lane only helps if `grounding:low` is written.
6. Defer UX chip, two-VLM consensus, classical HTR. They do not fix grounding.
7. Print Acc: keep the byte-identical guard; do not “improve” print pages.

## Cross-refs

- Why: [00-why.md](00-why.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
