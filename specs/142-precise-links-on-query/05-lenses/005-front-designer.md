# Lens 005 — Front Designer

## Visual contract

```text
  Answer paragraph …
    ┌─────────────────────────────┐
    │  Q3 Report.pdf, p.4         │  ← markdown link / chip
    └─────────────────────────────┘
           │ click
           ▼
  Document viewer: PDF page 4 + chunk row selected
```

## Components

| Surface | Behavior |
|---------|----------|
| Inline link | Same-origin `/documents/` → client nav; **never** `target=_blank` |
| Streaming `[N]` | Temporary chip resolved via catalog; swap to verified link text on Done |
| Citations panel | Passage index = `reference_id`; badge uses `formatChunkPageBadge` |
| External http(s) | Keep new-tab for non-document URLs |

## A11y

- Links have accessible name including document + page.
- Focus ring on chips; keyboard Enter activates.

## Cross-refs

- `document-url.ts`, `source-citations.tsx`, `MarkdownInlineTokens.tsx`
- UX: [004-ux.md](004-ux.md)
