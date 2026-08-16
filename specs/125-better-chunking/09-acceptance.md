# 09 — Acceptance

## Partner

- [ ] Heading-dense markdown note is **not** one chunk per heading.
- [ ] First chunk is not heading-only when children have bodies.
- [ ] Lineage shows heading path in continuation chunk text.
- [ ] Rebuild required messaging visible on workspace card.

## Engineering

- [ ] Packer SSOT; MarkdownChunking thin.
- [ ] tiktoken packing; Recursive Acc tests green.
- [ ] Kill switch restores 4-way heading-hard split.
- [ ] Fence ATX ignored; table header repeat on overflow.
- [ ] Langfuse/OTEL distribution keys without chunk text.
- [ ] Playwright hint testid.

## Ops

- [ ] `.env.example` documents `EDGEQUAKE_MARKDOWN_PACK`.
- [ ] Default ON (unset).

## Residual (honest)

- Setext / HTML headings not parsed.
- PDF strategy still page-aware recursive (not this packer).
- Tenant not in cascade.
