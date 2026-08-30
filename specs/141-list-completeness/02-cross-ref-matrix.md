# 02 — Cross-ref matrix

> **Cross-refs**: [Laws](01-first-principles.md) · [RCA](03-root-cause.md)

| Surface | Contract today | Verdict |
|---------|----------------|---------|
| Tenants / workspaces selector | COUNT + SQL page; `fetchAllPages` | **Fixed (140)** |
| Knowledge injections | API honest (`limit` default 50, `total`, `has_more`). Client sends no limit | **SILENT** |
| Admin quotas | `/tenants?limit=100` ignores `total` | **SILENT** at 101+ |
| Conversations | UI `useInfiniteQuery` + `next_cursor`. Backend ignores cursor | **SILENT** (21st row) |
| Documents inventory | API pager honest; UI `currentPage: 1`; `PaginationControls` unused | Overflow labeled, **page 2 unreachable** |
| `cancelPipeline` | `getTasksList({ status: "processing" })` default 20 | Extra tasks not cancelled |
| MCP `workspace_list` | SDK `list()` / `listWorkspaces()` unwrap items, REST default 20 | **SILENT** |
| Graph / document typeahead / dashboard recent / users pager / task queue card | Caps labeled or intentional preview | **TOP-K / PAGER-OK** — leave |

SPEC-140 Playwright (3 names, 21st workspace) does **not** prove selector
page-2: UI uses `limit=100`. Keep 140 tests; add HTTP `limit=10&offset=20`.
