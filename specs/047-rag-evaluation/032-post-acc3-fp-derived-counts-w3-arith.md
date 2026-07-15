# 032 — Post Acc #3 First Principles: derived Chart counts + next levers

**Date:** 2026-07-15  
**Prior:** [031](./031-acc3-dense-scalar-assessment.md) densify Acc #3 negative  
**Acc/F1 SOTA:** Acc #2 `…-1707` (0.562 / 0.480) · Chart long **0.571 PASS** (listmem)

---

## 1. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  REMAINING Chart long MISSES ARE MOSTLY NOT MISSING PIXELS.                  │
│                                                                              │
│  541 ≈ 36% × 1503 (Not-good × sample) — BOTH operands on political p3/p17  │
│  128 ≈ 18% × 710  (Dem neither × Dem n) — operands on p12 / p17            │
│  1251 ≈ 82% × 1526 (PIP 65+ go-online combo) — table n= on p16             │
│                                                                              │
│  Gen law today: "DO NOT invent, assume, or infer" → blocks %×N composition │
│  → Acc zeros despite W1 facts present. This is W3 (Gen), not W1 densify.   │
│                                                                              │
│  afe620 years: page has "1981-82" / "2001-02" — need year-span expand      │
│  (W1 Pass A + fidelity), not a new chart crop (page 2 has 0 chart assets). │
│                                                                              │
│  Acc #3 densify prompts: REGRESS Acc/F1 — REVERT.                            │
│                                                                              │
│  Next execute:                                                               │
│    1) Revert Acc #3 densify callout prompt fluff                             │
│    2) W3-arith — allow grounded %×N when both operands cited in context    │
│    3) Year-span expand (fidelity + Pass A Key values)                        │
│    4) Acc #4 vs Acc #2                                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Causal map (binding law)

```text
PDF → Pass A (% + N on page) → index OK
                              ↓
                     Gen prompt forbids infer
                              ↓
                     pred refuses / wrong → Acc 0
                              ↓
                     a_in_e_long stays false (correct: "541" not literal)
```

| Miss | Class | Lever |
|---|---|---|
| 541, 128, 1251 (±4087) | Derived count | **W3-arith** |
| afe620 1981–2002 | Year-span abbrev | **Year-span expand** |
| Indonesia 198 | Wrong page (in_doc) | W2 later |
| Acc #3 prompt fluff | Noise | **Revert** |

Honesty: do **not** claim W1 Chart representation win from Acc↑ driven by W3-arith. Gate already PASS via listmem.

---

## 3. Execute checklist

- [x] Write this plan; update 026 header
- [x] Revert densify sample-size/year callout lines in `prompts.rs`
- [x] W3-arith in `grounding.rs` (+ tests); soften absolute "no infer" carve-out (text + vision)
- [x] Year-span expand in `fidelity.py` + Pass A prompt bullet
- [x] Unit tests green (fidelity 13 · grounding · prompts · Pass A year-span)
- [x] Acc #4 rebuild + smoke; mid + final assess vs Acc #2 ([034](./034-acc4-w3-arith-assessment.md) Acc/F1 no lift)
- [x] Acc #5 W3-arith-v2 query-only (MUST + example) ([035](./035-acc5-w3-arith-v2-assessment.md) Acc≈#2; 1251 hit; F1 short)
