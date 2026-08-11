# Lens 005 — Front Designer

## Visual language

Stay inside existing Documents / Workspace settings patterns (no new card chrome). Parser controls remain compact: select + resolved chip.

## Hierarchy

```ascii
  Settings page
    Vision LLM card
    PDF Parser card  ← choice + "Resolves to {Vision|EdgeParse|Auto}" chip
    Extraction language
    Chunking

  Documents dropzone
    Parser for this upload (inherit label includes resolved choice)
```

## Motion / feedback

- Chip updates when workspace/tenant/env context loads (no flash of wrong backend).
- Admission dialog: list large files distinctly from “other files keep X”.

## Accessibility

- Select labels associated; chip text not color-only.
- Announce resolved parser on change for screen readers.
