# 06 — UX / UI specification (normative)

## Principles

1. **Page first** — scan + Pass-A markdown dominate the first viewport of the viewer.
2. **Honest chips** — modality and confidence are visible for non-print pages.
3. **No crop theater** — Pass-B region cards are secondary/collapsed on manuscript;
   never show a strip of axis-tick / single-bar thumbnails as the “analysis” of a hand chart (LAW-134-16).
4. **Existing tokens** — reuse documents / side-by-side patterns; no new brand theme.

## Layout

```ascii
  ┌──────────────────────────────────────────────────────────┐
  │ Title · status · [ModalityChip] [ConfidenceBadge]         │
  ├────────────────────────────┬─────────────────────────────┤
  │                            │                             │
  │   Page PNG (scroll/zoom)   │   Pass-A Markdown           │
  │                            │   (reading order)           │
  │                            │                             │
  ├────────────────────────────┴─────────────────────────────┤
  │ ▸ Region notes (Pass-B) — collapsed by default if MS     │
  └──────────────────────────────────────────────────────────┘
```

## Component contracts

### ModalityChip

| modality | Label | Emphasis |
|----------|-------|----------|
| `print` | Print | Subtle |
| `manuscript` | Manuscript | Strong |
| `mixed` | Mixed | Warning |

`data-testid="page-modality-chip"`

### ConfidenceBadge

- Hidden when `transcription_confidence` is null.
- Show `High` (≥0.8), `Medium` (≥0.5), `Low` (<0.5) or percent — pick one SSOT in implement; document in CHANGELOG.
- `data-testid="transcription-confidence"`

### Empty / thin transcript

If MD length below threshold while modality manuscript → banner:
“Transcription looks thin — verify against the scan.”

### Pass-B region list

If `page_modality != print` AND (crop `area_frac < T_noise` OR crop is chart-fragment)
→ do not auto-expand; omit from default list when gated server-side (preferred).

For hand-chart pages, the **Markdown panel** must show the whole-graphic digitization;
a strip of tick-number thumbnails is a UX defect (LAW-134-16).

## API fields (FE consume)

```json
{
  "page": 1,
  "page_modality": "manuscript",
  "transcription_confidence": 0.62,
  "vision_profile": "manuscript"
}
```

## A11y

- Chip and badge have accessible names.
- Contrast AA on badges.
- Keyboard: region accordion operable.

## Out of scope

- Live in-browser HTR
- Editing gold transcripts in UI (v1)
- Replacing PDF.js viewer

## Cross-refs

- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md), [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
