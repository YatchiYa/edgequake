# SPEC-141 — List completeness (pagination audit)

> **Trigger:** After SPEC-140, catalogs still silently drop rows on other
> surfaces (injections, admin quotas, conversations, documents pager, pipeline
> cancel, MCP `workspace_list`).
> **Method:** Three laws (exhaust / honest pager / labeled top-K) + unfakable
> HTTP + Playwright + MCP.
> **Target cut:** next patch after 0.26.3.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  A surface that looks like “the list of X” is broken if the user cannot        │
│  name every X that exists.                                                   │
│                                                                              │
│  Catalog / selector / admin grid  → exhaust pages (or honest N of total)    │
│  Unbounded table (documents)      → page with honest COUNT; do not fetch-all│
│  Search / graph / dashboard recent → TOP-K only if labeled                   │
│                                                                              │
│  Remaining silent caps (this pack):                                         │
│    knowledge injections (client ignores total, default 50)                  │
│    admin quotas (GET /tenants?limit=100, ignores total)                    │
│    conversations (UI sends cursor; backend ignores it)                       │
│    documents inventory (currentPage hardcoded 1; PaginationControls unused) │
│    cancelPipeline (first 20 processing tasks)                                 │
│    MCP workspace_list (SDK unwraps first page, REST default 20)              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| C1 | Knowledge injections client | **Fix** | `listInjections` exhausts `fetchAllPages` |
| C2 | Admin quota tenant grid | **Fix** | `getTenants()` exhaust |
| C3 | Conversation `cursor` | **Fix** | offset + `next_cursor` when `offset+len < total` |
| C4 | Documents inventory pager | **Fix** | `currentPage` + `PaginationControls` + `document_pattern` |
| C5 | `cancelPipeline` | **Fix** | `fetchAllPagesByIndex` |
| C6 | MCP / SDK workspace list | **Fix** | SDK exhaustes offset pages |
| E2E | HTTP-141 + Playwright + MCP | **Required** | [06](06-e2e-test-matrix.md) |

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-141-1..6)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 05-fix-plan
   → 06-e2e-test-matrix
   → 07-edge-cases
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Tenant/workspace selector | Already SPEC-140; keep; add HTTP page-2 |
| Documents corpus | Honest pager; **do not** `fetchAllPages` |
| Conversation UI | Keep infinite query; honor cursor |
| `GET /conversations/{id}` 200-message cap | TOP-K for one thread; out of scope |
| Membership-scoped workspace lists | Out of scope |
| REST max above 100 | Out of scope (injections already max 200) |
| Merge six React Query tenant sites | Out of scope (cache already dedupes) |

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|----------|
| [SPEC-140](../140-new-created-workspace-issue/) | Honest COUNT + selector exhaust |
| [SPEC-099](../099-ux-ui-improvement/) | Overflow honesty; `VIRTUAL_PAGE_SIZE=100` |
| [SPEC-017](../017-ui-dry/) | DRY client helpers |
