# Lens — Product Owner

## Outcome

A user reading a scanned multi-page PDF next to its Markdown extraction can
scroll either pane and stay on the same logical page. Mouse wheel on the PDF
feels like a normal document viewer.

## Jobs to be done

1. Flip PDF pages with wheel / keyboard without hunting toolbar chevrons.
2. See the matching Markdown section when the PDF page changes.
3. Scroll Markdown and have the PDF follow (when sync is on).
4. Temporarily unsync when comparing distant sections.
5. Keep SPEC-142 citation deeplinks (`?page=`) working.

## Non-goals (v1)

- Pixel-perfect bbox sync for every paragraph (SPEC-128 covers figure click).
- Changing extraction / marker grammar.
- Query answer citation work (SPEC-142).

## Success metrics

| Metric | Gate |
|--------|------|
| Wheel changes `data-page` | E2E pass |
| Sync PDF→MD / MD→PDF | E2E pass |
| Sync OFF independent | E2E pass |
| No DB migration | Ship without schema PR |

## Risks

| Risk | Mitigation |
|------|------------|
| Large PDF memory | Windowed render >20 pages |
| Sync feedback loop | Driver lock + settle |
| Legacy docs without markers | Degrade; PDF still navigable |

## Cross-refs

- Acceptance: [10-acceptance.md](../10-acceptance.md)
- UX: [004-ux.md](004-ux.md)
