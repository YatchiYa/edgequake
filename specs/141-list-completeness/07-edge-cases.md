# 07 — Edge cases

> **Cross-refs**: [Laws](01-first-principles.md)

| ID | Case | Expected |
|----|--------|----------|
| EC-1 | Empty catalog | `items=[]`, `total=0`, no extra requests |
| EC-2 | `total` lie, short last page | Exhaust helper stops on short page |
| EC-3 | Runaway server (full pages, huge total) | `FETCH_ALL_PAGES_MAX` |
| EC-4 | Garbage conversation cursor | Treat as offset 0 |
| EC-5 | Last conversation page | `has_more=false`, `next_cursor` absent/null |
| EC-6 | Document search | Server `document_pattern`; do not client-filter only page 1 |
| EC-7 | Page size change on documents | Reset to page 1 |
| EC-8 | SDK legacy array body | Treat as complete list (no loop) |
| EC-9 | MCP tenant bootstrap | `tenants.list()` exhausts so multi-tenant warning is honest |
| EC-10 | `cancelPipeline` with 0 processing | No-op |

## Out of scope

- Membership-scoped workspace lists
- Raising REST max above 100 (injections already max 200)
- Acc re-score
- Refactoring TenantProvider / header / guard into one module
- Graph viz “full catalog”
- Raising per-conversation GET message cap (200)
