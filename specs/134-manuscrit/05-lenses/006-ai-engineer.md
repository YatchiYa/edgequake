# Lens 006 — AI Engineer

## Stake

VLMs are strong at page HTR **when prompted for fidelity** and fed enough pixels.
They also **hallucinate fluently** on crops (operator “today” failure).

## Prompt engineering (MS)

- Explicit: preserve source language; no translate; no modernization.
- Explicit: `[?]` for unreadables; forbid invention.
- Explicit: implicit tables + color series.
- Explicit: output page transcription, not “describe this image” caption mode.
- Temperature: keep low (existing ~0.1); optional omit per SPEC-131.

## Model selection (SOTA Aug 2026)

| Tier | Model | CER (IAM) | Cost/1K pg | Use |
|------|-------|-----------|------------|-----|
| Frontier | GPT-5 | ~1.22% | ~$12 | Default MS if budget allows |
| Frontier | Claude Opus 4.7 | ~1.31% | ~$15 | Long docs, reasoning |
| Frontier | Gemini 3 | ~1.44% | ~$8 | Multilingual |
| Cost | GPT-5-mini | ~1.52% | ~$2 | Cost-sensitive MS |
| Value | Mistral OCR 3 | ~2.1% | ~$2 | Cursive, budget |
| Local | Qwen2.5-VL | ~3.8% | $0 | Privacy, fine-tune |
| **Avoid** | Tesseract | 12.5% | $0 | Not for handwriting |

**Routing:** `EDGEQUAKE_VISION_MODEL` (or MS-specific override WP-10) should point to
frontier class for MS pages. Small/legacy VLMs are not MS-capable.

**Classifier VLM:** Off by default; heuristics first.

## Hallucination controls

1. Page-as-unit Pass-A before Pass-B.
2. **Graphic-as-unit** — never specialize axis ticks / single bars (LAW-134-16).
3. Noise crop gate (area/ink).
4. Gold CER/WER + “no forced English” checks + chart KV recall.
5. Confidence heuristic: length vs ink, `[?]` rate, empty output → low.
6. **Verify pass (WP-9)** — Judge-and-Refine when confidence low.
7. **Consensus (WP-11)** — optional two-VLM agreement.

## SOTA techniques (Aug 2026)

| Technique | Status |
|-----------|--------|
| Frontier VLM routing | WP-10 |
| Judge-and-Refine | WP-9 |
| Consensus entropy | WP-11 |
| Schema-first extraction | MS prompt structured sections |
| Capability reflection | WP-9 verify prompt |
| Calibrated confidence | v2 research (WP-14) |

## Interaction with Acc (SPEC-047)

Print Acc remains English-pinned. Manuscript profile is **orthogonal** — do not
break Acc by changing default print prompt.

## Cross-refs

- Prompt SSOT target: [../04-target-architecture.md](../04-target-architecture.md)
- OCR metrics: [007-ocr-expert.md](007-ocr-expert.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
