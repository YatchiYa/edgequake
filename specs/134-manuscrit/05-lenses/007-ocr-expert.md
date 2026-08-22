# Lens 007 — OCR Expert

## Stake

Production “OCR” is **Pass-A VLM page transcription**, not Tesseract/Kraken.
Handwriting (HTR) needs different success metrics and render floors than print OCR.

## Split (extends SPEC-128 OCR lens)

| Need | Channel |
|------|---------|
| Manuscript body text | Pass-A MS prompt @ high DPI |
| Implicit tables | Pass-A → GFM |
| Hand charts | Pass-A Key values + series (**whole graphic**) |
| Chart fragments (ticks, one bar) | **Suppress** Pass-B; not OCR units |
| Print body | Existing Pass-A print prompt |
| Overlay boxes | SPEC-128 layout (orthogonal) |
| Classical HTR | **Non-goal v1** (LAW-134-9) |

## Metrics

| Metric | Use |
|--------|-----|
| CER / WER | Text regions vs gold (Unicode NFKC normalize) |
| Table cell F1 | Implicit + explicit tables |
| Chart KV recall | Readable axis callouts / series points |
| Hallucination flags | Invented words not in ink; forced English when source ≠ EN |
| Crop theater rate | Pass-B cards on area_frac < T (should → 0 on MS gold) |
| Verify pass rate | WP-9: % MS pages needing refinement |
| Consensus entropy | WP-11: agreement score distribution |

Literature (Aug 2026): frontier VLMs (GPT-5 ~1.22% CER, Opus 4.7 ~1.31%, Gemini 3
~1.44%) dominate IAM HTR; specialized TrOCR/DTrOCR are now fine-tune baselines, not
ceilings. Prompt-based VLMs beat classical OCR when fidelity prompts + adequate DPI
are used, but low CER can hide semantic substitutions — hence fidelity rules +
verify pass + human-visible confidence.

## SOTA alignment

| Technique | Evidence | SPEC-134 |
|-----------|----------|----------|
| Frontier VLM | IAM leaderboard | WP-10 routing |
| 300 DPI min | Universal HTR guidance | WP-2 floor |
| `[?]` abstention | Reduces hallucination | MS prompt |
| Judge-and-Refine | MinerU2.5-Pro | WP-9 |
| Consensus entropy | CVPR 2026 | WP-11 |
| Calibrated confidence | MF Smart case study | v2 (WP-14) |

## Preprocess (v1)

Prefer **resolution** over aggressive binarization (color is data). Deskew optional
later; not required for first ship.

## Failure modes

| Case | Mitigation |
|------|------------|
| Faint pencil | Higher DPI; `[?]`; low confidence |
| Graph-paper grid | Prompt: ignore grid as content; keep axis ticks |
| Cursive ambiguity | `[?]`; never invent |
| Scanner OCR layer | Ignore for MS classify; Vision required |

## Cross-refs

- SPEC-128 OCR: [../../128-improve-pdf-parsing/05-lenses/009-ocr-expert.md](../../128-improve-pdf-parsing/05-lenses/009-ocr-expert.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
