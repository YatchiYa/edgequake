# 06 — UX / UI Spec

## Inline citation

| State | Rendering |
|-------|-----------|
| Streaming raw `[N]` | Chip `[N]` → resolves via catalog to temporary link |
| Verified (Done / sync) | Compact **page chip** `[p.P](href "FullDocName")` — visible `p.P` / `p.P–Q`; full name on hover `title` |
| Non-PDF | Short title chip `[ShortName](href "FullName")` — no page |
| Span | Link text `p.3–4`; href `page=3` |
| Unknown `[N]` | Removed from text |

Adjacent chips are spaced (`mx-0.5`) so clusters stay readable; do not collapse multi-id cites into one chip (each click keeps its own locator).

## Click behavior

1. Same-tab Next.js navigation to `/documents/{id}?chunk=&page=`.
2. PDF viewer `currentPage` = URL page.
3. Hierarchy selects `chunk` id; sidebar opens if needed.
4. External URLs remain new-tab.

## Citations panel

- Group by page when `page_start` present (SPEC-033).
- Display index = `reference_id` when present.
- Badge = `formatChunkPageBadge(page_start, page_end)`.

## Empty / error

- Bypass: no citation chrome.
- Missing title: short fallback (file_name or truncated id) — still correct doc id.

## Cross-refs

- Front lens: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
