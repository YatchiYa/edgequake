# 05 — Fix plan

> **Cross-refs**: [Laws](01-first-principles.md) · [E2E](06-e2e-test-matrix.md)

## 1. DRY client

- Keep `fetchAllPages(offset, limit)`.
- Add `fetchAllPagesByIndex(fetchPage: (page, pageSize) => { items, total })`
  for 1-based task lists.
- `listInjections` exhausts via `fetchAllPages`.
- Admin quotas call `getTenants()`; heading shows count.

## 2. Conversations

Decode `cursor` as `usize` offset (garbage → 0). Pass into storage.
`next_cursor` when `offset + items.len() < total`. Same for `list_messages`.
Do not invent a second UI pager.

## 3. Documents inventory

Pass `currentPage` into `useDocumentQueries`. Mount `PaginationControls`.
Send `document_pattern` for title search. Do **not** exhaust the corpus.

Keep `VIRTUAL_PAGE_SIZE = 100` (SPEC-099). Pager can change page size to 10
in e2e so 21 docs prove page 2.

## 4. cancelPipeline

`fetchAllPagesByIndex` on `getTasksList({ status: "processing", page_size: 100 })`.

## 5. MCP / SDK

Exhaust inside SDK `list()` / `listWorkspaces()` (`limit=100`, loop until
`len >= total`). MCP tools inherit it. Graph `per_page` vs REST `page_size`
is out of scope unless a tool claims “list all”.

## Honesty leftover (do not over-claim)

- Entity browser badge = loaded nodes; TruncationBanner already labels it.
- SPEC-140 three-name Playwright stays as UX coverage, not page-2 proof.
- `GET /conversations/{id}` message cap 200 is TOP-K for one thread.
