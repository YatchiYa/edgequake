# 12 — SOTA assessment (August 2026)

Deep assessment of OCR / PDF / AI-engineering / VLM state of the art, mapped to
EdgeQuake code anchors and SPEC-134 improvements. Sources: IAM HTR leaderboard,
OmniDocBench v1.6, Dr.DocBench, OCRBench v2, OCR-Agent (CVPR 2026), Consensus
Entropy (CVPR 2026), MinerU2.5-Pro, PlotPick / ChartZero, confidence-calibration
case studies. All mapped to verified code paths.

## 1. Handwriting / manuscript OCR

### SOTA ranking (IAM CER, lower is better)

| Tier | Model | CER | Notes |
|------|-------|-----|-------|
| Frontier VLM | GPT-5 | ~1.22% | Best overall |
| Frontier VLM | Claude Opus 4.7 | ~1.31% | Long docs, reasoning |
| Frontier VLM | Gemini 3 | ~1.44% | Multilingual, word boxes |
| Cost VLM | GPT-5-mini | ~1.52% | ~6x cheaper than GPT-5 |
| Cloud API | Azure Doc Intel v4.0 | ~1.8% | Enterprise + bboxes |
| Cloud API | Mistral OCR 3 | ~2.1% | Best value, cursive |
| Specialized | DTrOCR | 2.38% | WACV 2024 research |
| Open-weight | TrOCR-Large | 2.89% | Fine-tune baseline |
| Open VLM | Qwen2.5-VL | ~3.8% | Multilingual |
| Legacy | Tesseract 5 | 12.5% | Not for handwriting |

**Implication:** Specialized HTR (TrOCR/Kraken) is no longer the accuracy ceiling;
frontier VLMs dominate. EdgeQuake’s VLM Pass-A architecture is the right substrate —
the gap is **prompt + render + routing + honesty**, not the absence of classical HTR.

### SOTA techniques to adopt

| Technique | Evidence | EdgeQuake hook |
|-----------|----------|----------------|
| **Frontier VLM routing** for MS | GPT-5 / Opus 4.7 / Gemini 3 top IAM | `EDGEQUAKE_VISION_MODEL` + profile override |
| **Schema-first extraction** | +15% field accuracy (2026 forms benchmark) | MS prompt emits structured sections |
| **300 DPI minimum** | Universal HTR guidance | `ManuscriptProfile` DPI floor |
| **Preserve source language** | Prompt-based HTR best practice | MS prompt removes EN pin |
| **`[?]` abstention** | Reduces hallucination | MS prompt + confidence |
| **Line-level alignment** | Ukrainian cards, IAM | Future L2; not v1 |

## 2. PDF document parsing

### Benchmarks

| Benchmark | Scope | Leader (Aug 2026) |
|-----------|-------|-------------------|
| OmniDocBench v1.6 | 1651 pages, 10 doc types incl. handwritten notes | PaddleOCR-VL-1.6 / MinerU2.5-Pro |
| Dr.DocBench | Expert-level, long-tail difficulty | Highlights where standard parsers saturate |
| OCRBench v2 | Text, tables, formulas, reading order | MinerU2.5 leads layout F1 |

### Paradigms

```ascii
  Pipeline (MinerU2.5-Pro)          End-to-end VLM (PaDoc / EdgeQuake Pass-A)
  ─────────────────────────          ───────────────────────────────────────
  layout → OCR → table → formula     full page → VLM → Markdown
  + Judge-and-Refine loop            + prompt controls structure
  + expert annotation for hard       + fast, one pass
```

**SOTA lesson:** Even pipeline leaders add a **visual-comparison Judge-and-Refine**
loop (render structured output back to image, compare, correct). EdgeQuake should
adopt a lightweight version for MS pages (WP-9).

### Implication for EdgeQuake

- Keep **end-to-end VLM Pass-A** (right for RAG speed/cost).
- Add **layout-aware region routing** (SPEC-128) to avoid crop theater.
- Add **self-correction / verification** for MS pages (agentic OCR).

## 3. AI engineering / agentic OCR

### SOTA patterns (2026)

| Pattern | Source | Benefit |
|---------|--------|---------|
| **Capability Reflection** | OCR-Agent (CVPR 2026) | Model diagnoses errors, plans only executable fixes |
| **Memory Reflection** | OCR-Agent | Avoids repeated failed attempts |
| **Consensus Entropy** | CVPR 2026 | Multi-VLM agreement → accept / route to stronger model |
| **Judge-and-Refine** | MinerU2.5-Pro | Render output back to image, compare, correct |
| **Calibrated confidence** | MF Smart Research case study | Uncalibrated “certain” was 56% correct; calibrated threshold → 90% |
| **Schema / field lists** | 2026 forms benchmark | +15% field accuracy |

### EdgeQuake gap

No generic self-correction / reflection wrapper exists. Existing two-pass patterns:

- SPEC-049 figure filter: Pass-1 classify → Pass-2 describe
- Gleaning re-extraction (`MAX_GLEANING_CAP=2`)

**Improvement:** Add **MS Verify pass** (WP-9): after Pass-A MS, a second VLM call
judges the MD against the page image (fidelity, missing tables, invented text) and
either accepts or refines. Gate by confidence heuristic.

## 4. Chart / graphic digitization

### SOTA (2026)

| Approach | Result | Limitation |
|----------|--------|------------|
| DePlot / MatCha | Strong on trained chart types | Poor generalization to in-the-wild / scientific |
| General VLMs (PlotPick) | 88–96% recall on ChartX, beat DePlot | Still need whole-chart input |
| ChartZero | Zero-shot via synthetic priors + GOI loss | Research; not product binary |
| Self-ensembling | Repeated sampling improves accuracy | Cost |

**Key finding:** VLMs outperform dedicated chart-to-table models **when given the
whole chart**. Feeding axis ticks / single bars destroys accuracy — exactly the
operator failure mode.

**Implication:** LAW-134-16 (graphic-as-unit) is SOTA-aligned. Pass-A MS must
digitize the whole chart; Pass-B must suppress fragments.

## 5. Confidence calibration

### SOTA practice

- **Do not trust raw model confidence.** Calibrate against ground truth.
- **Consensus entropy:** run 2+ VLMs; low entropy → accept; high → route to expert.
- **Decouple visual vs reasoning confidence.**
- **Isotonic regression / threshold tuning** on a manually transcribed dev set.

### EdgeQuake gap

- `page_layout_regions.confidence` exists but is always `None`.
- Figure-filter Pass-1 emits `confidence` in JSON but `FigureFilterResult` drops it.
- No Pass-A quality score.

**Improvement (WP-5 / WP-9):**

1. Heuristic confidence v1: `[?]` rate, MD length vs ink, empty→0.
2. Persist to `document_pages.transcription_confidence`.
3. v2: optional consensus check (two VLMs on MS page) → calibrated confidence.
4. Document calibration protocol in honest assessment.

## 6. Code anchor verification (Aug 2026)

Verified against working tree. Key blockers for ManuscriptProfile:

| Blocker | Location | Impact |
|---------|----------|--------|
| DPI clamp caps at safe profile | `pdf_processing.rs:953` `.clamp(96, safe_dpi.max(96))` | Cannot express MS floor 300 via env |
| `max_rendered_pixels` hardcoded 2000 | `backend/vision.rs:149` | High-DPI downscaled at render |
| Prompt select path exists | `vision_page_system_prompt` metadata → `PageDrawingAssetsConfig` | Not blocked; needs profile bridge |
| Pass-B suppress needs new gate | `analyzer.rs:128`, `gates.rs:66` | Profile-level suppress independent of user options |
| No profile field on `VisionConversionConfig` / `PdfProcessingData` | `backend/mod.rs:~130`, `data.rs:159` | Need plumbing |
| No Pass-A confidence hook | — | Need new scoring signal |

**SPEC-096 vs Pass-A English pin:** No code conflict today — SPEC-096 language is
consumed only by KG extraction and keyword extraction, never by pdf2md vision path.
A ManuscriptProfile must explicitly bridge `EDGEQUAKE_EXTRACTION_LANGUAGE` →
`vision_page_system_prompt` override if non-English Pass-A is desired.

## 7. Improvement plan (SOTA-aligned)

### v1 (ship)

| WP | SOTA technique | Code anchor |
|----|----------------|-------------|
| WP-1 | PageModality classifier | `page_modality.rs` (new) |
| WP-2 | 300 DPI floor + max_px 3600 | bypass clamp `pdf_processing.rs:953`; profile `vision.rs:149` |
| WP-3 | MS fidelity prompt (no EN pin, `[?]`, whole-graphic) | `vision_prompts.rs` new constant |
| WP-4 | Graphic-as-unit Pass-B suppress | `figure_filter.rs` / `analyzer.rs` gates |
| WP-5 | Heuristic confidence persist | `document_pages` migration |
| WP-6 | Env / override | `.env.example`, AGENTS.md |
| WP-7 | UX chip + confidence | FE |
| WP-8 | Synthetic gold + CER/WER | `fixtures/` |

### v1.5 (fast follow)

| WP | SOTA technique |
|----|----------------|
| WP-9 | **MS Verify pass** — Judge-and-Refine lite: second VLM compares MD to page image, refines if low confidence |
| WP-10 | **Frontier VLM routing** — recommend `gpt-5` / `claude-opus-4.7` / `gemini-3` for MS; document cost/accuracy tradeoff |
| WP-11 | **Consensus confidence** — optional two-VLM agreement on MS pages |

### v2 (research)

| WP | SOTA technique |
|----|----------------|
| WP-12 | Line-level alignment for HTR (Ukrainian cards pattern) |
| WP-13 | ChartZero-style synthetic priors for hand charts |
| WP-14 | Calibrated confidence via isotonic regression on dev set |
| WP-15 | Optional TrOCR/DTrOCR fine-tune for domain-specific cursive |

## 8. Model recommendation matrix (MS pages)

| Priority | Model | CER | Cost/1K pg | When |
|----------|-------|-----|------------|------|
| Accuracy | GPT-5 | ~1.22% | ~$12 | Default MS if budget allows |
| Balanced | GPT-5-mini | ~1.52% | ~$2 | Cost-sensitive MS |
| Long docs | Claude Opus 4.7 | ~1.31% | ~$15 | Multi-page context |
| Multilingual | Gemini 3 | ~1.44% | ~$8 | Non-Latin scripts |
| Value | Mistral OCR 3 | ~2.1% | ~$2 | Cursive, budget |
| Local | Qwen2.5-VL | ~3.8% | $0 | Privacy, fine-tune |

**Do not use for MS:** Tesseract (12.5% CER), small/legacy VLMs.

IAM / CodeSOTA CER figures above are **vendor/community leaderboard numbers**,
not a substitute for EdgeQuake gold (LAW-134-10). Do not cite them as product CER.

## 10. Slice E science (Aug 2026)

Facts that lock page-as-unit + class-routed render (not more crop theater):

- **OmniDocBench (CVPR 2025) Table 3:** pipeline tools trained on academic
  papers collapse on handwritten notes (MinerU Notes edit distance **0.984**).
  General VLMs generalize better on notes / degraded scans than print pipelines.
- **OmniDocBench v1.5:** note and newspaper page images raised from **72 DPI to
  200 DPI** because low-res notes were unscorable. EdgeQuake ImageGuard 1024px
  long-side (~87 DPI A4) is below that floor; MS Pass-A must keep ≥2000px.
- **DISCO (2026):** generic VLM prompts lose to OCR on IAM; **task-aware
  prompting** closes the gap — MS vs print prompt routing is not optional.
- **pdf2md 0.9.11:** `render_pages_blocking` ignores `dpi` and uses
  `max_rendered_pixels` as target width. WP-2 DPI floor is decorative until
  that value is forwarded (Slice E WP-E2).

## 11. Risks and honest limits

- Frontier VLM cost at 300 DPI × 3600px is non-trivial; concurrency caps required.
- Judge-and-Refine adds latency; gate by confidence.
- Consensus entropy needs 2+ VLM calls; optional.
- Calibrated confidence needs a labeled dev set; v1 heuristic is uncalibrated.
- Hand-drawn log-grid charts remain hard even for SOTA VLMs; expect `[?]`.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
