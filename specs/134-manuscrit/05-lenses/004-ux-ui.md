# Lens 004 — UX / UI Designer

## Stake

The side-by-side viewer must answer: **“Did we read this page?”** — not “did we
narrate a scribble?”

## Jobs

1. Show page modality (Print / Manuscript / Mixed).
2. Show transcription confidence when known.
3. Keep full-page scan as the visual SSOT; MD panel shows transcript.
4. Demote Pass-B crop cards when modality is manuscript and crop is noise-class.

## Information hierarchy

```ascii
  [Modality chip] [Confidence]
  ────────────────────────────
  LEFT: page PNG (dominant)
  RIGHT: Pass-A markdown (dominant)
  ────────────────────────────
  Optional: “Region notes” collapsed (Pass-B) — never above page MD
```

## Copy

- Manuscript: “Handwritten / scanned page — transcription may mark `[?]` for unclear ink.”
- Low confidence: “Low transcription confidence — verify against the scan.”
- Never: “Vision Analysis complete” as the only success signal for MS pages.

## Accessibility

Chip text + color; not color-only. Confidence as percentage or High/Med/Low.

## Cross-refs

- Normative: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front: [005-front-designer.md](005-front-designer.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
