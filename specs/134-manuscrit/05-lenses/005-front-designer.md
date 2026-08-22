# Lens 005 — Front Designer

## Stake

Implement chips and panel priority with existing design tokens (SPEC-029 / 099).
No new purple-glow aesthetic; match documents viewer patterns.

## Components

| Component | Behavior |
|-----------|----------|
| `PageModalityChip` | Reads `page_modality`; Print muted, Manuscript emphasized, Mixed warning-tint |
| `TranscriptionConfidence` | Hidden if null; else meter or badge |
| Side-by-side layout | Page image + MD primary; region analysis accordion secondary |
| Empty MD | Explicit empty state linking to confidence / retry |

## Data

Prefer existing page layout / document detail queries; extend types once (DRY).
Do not client-side reclassify PDF.

## States

| State | UI |
|-------|-----|
| Processing | Existing spinner; chip “Detecting…” optional |
| Completed print | Chip Print; no confidence required |
| Completed MS | Chip + confidence; MD panel |
| Failed | Existing error; do not show fake Vision Analysis |

## Playwright

`E2E-134-UI-chip` — synthetic MS fixture shows Manuscript chip after complete.

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- SPEC-128 overlay: [../../128-improve-pdf-parsing/06-ux-ui-spec.md](../../128-improve-pdf-parsing/06-ux-ui-spec.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
