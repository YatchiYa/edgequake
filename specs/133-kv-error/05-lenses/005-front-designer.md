# Lens 005 — Front designer

## Visual system (Documents detail)

Preserve existing Failed treatment from the design system — do not invent a new
“KV error” chrome. This incident is backend identity resolution.

```ascii
  [ Failed ! ]   Download   Graph   [ Reprocess ]
  ┌─ error banner (destructive soft) ─────────────┐
  │  message + SPEC-098 miss sample list           │
  └────────────────────────────────────────────────┘
```

## Layout constraints

- Banner must not truncate the miss sample list below ~3 entries (support RCA).
- Reprocess remains the primary recovery CTA after upgrade.
- No new cards, pills, or hero treatments on the detail header for this bug.

## Tokens

Reuse existing destructive / warning tokens already used for pipeline Failed.
No purple/glow/new gradient language.

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- Product: [001-product-owner.md](001-product-owner.md)
