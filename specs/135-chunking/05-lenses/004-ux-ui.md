# Lens 004 — UX / UI

## Stake

Operators cannot see **fill**. Citations assume one page. After 135, a packed
chunk may cover `p.3–4`. The viewer must still open a **single** page
(`page_start`) so the deeplink stays valid.

## Surfaces

| Surface | Change |
|---------|--------|
| Workspace Chunking card | One hint: PDFs pack to the token budget (future-only). Testid `chunking-pdf-pack-hint`. |
| Document hierarchy / lineage chunk row | Badge `p.N` or `p.N–M`. Testid `chunk-page-badge`. |
| PDF viewer | Deeplink `#page={page_start}` unchanged in *mechanism*; copy may say “starts on page N”. |
| Langfuse (ops, not WebUI) | `fill_p50` on `ingest.chunking` — no new UI chart in v1. |

## Copy (v1)

| Element | Copy |
|---------|------|
| PDF pack hint | PDF conversions pack headings, figures, and short pages into the token budget so extract is not one job per page. |
| Future-only | Applies to future document ingestions. Use Rebuild Knowledge Graph to re-chunk existing documents. |
| Span badge | `p.{start}–{end}` when `end > start`; else `p.{start}`. |
| Badge title | `Open PDF at page {start}` (always start). |

## Non-UI

Kill switches stay env-only (same as SPEC-125 `EDGEQUAKE_MARKDOWN_PACK`).
Document in `.env.example`.

## Playwright

`E2E-135-UI`: fixture with a span chunk → visible `p.1–2`; click uses start page.

## Cross-refs

- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
