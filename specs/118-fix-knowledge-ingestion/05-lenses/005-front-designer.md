# Lens 005 — Front Designer

## v1 stance

**No visual redesign.** SPEC-118 is a persistence identity fix.

## Guardrails

- Do not add cards, badges, or dual-id debugging chrome to the injection UI.
- Preserve existing status chips / empty states.
- If a future “advanced” panel shows document ids, show both composite + UUID only behind an existing debug pattern — not in the default composition.

## data-testid

No new testids required for v1. E2E coverage is API/backend-first; Playwright optional later for status honesty only.
