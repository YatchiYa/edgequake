# 01 — First principles (LAW-141)

> **Cross-refs**: [WHY](00-why.md) · [RCA](03-root-cause.md) · [Fix](05-fix-plan.md)

## Laws

1. **LAW-141-1 — Exhaust catalogs.** A selector, admin grid, MCP “list all”,
   or knowledge catalog must follow pages until `accumulated >= total` (or
   show honest “N of total” **and** a next control). Dropping the 51st row
   with no pager is a bug.
2. **LAW-141-2 — Page unbounded tables.** Documents and similar corpora must
   not be fetched entirely into RAM. `total` is `COUNT` of the filtered set.
   Wire a pager. Do not `fetchAllPages` the document inventory.
3. **LAW-141-3 — Label top-K.** Search, graph viz, and “recent” previews may
   cap **only** if the UI says so (`has_more`, truncation banner, “showing 10 of
   Y”). Unlabeled truncation is a silent catalog.
4. **LAW-141-4 — `total` is COUNT.** Never `items.len()` after `take`. Cursor
   `has_more` is `offset + items.len() < total`, not `total > items.len()`
   (the latter is true on every non-empty last page when `total > page`).
5. **LAW-141-5 — One helper per encoding.** Offset/limit → `fetchAllPages`.
   1-based `page` → `fetchAllPagesByIndex`. No ad-hoc `?limit=100`.
6. **LAW-141-6 — Do not membership-filter workspace lists.** SPEC-140 lock.
   List remains tenant-scoped.

## Causal diagram

```text
  Surface claims “list of X”
           │
           ▼
  GET first page (default 20 / 50 / 100)
           │
           ├─ client maps items only, ignores total     → SILENT
           ├─ backend ignores cursor / hardcodes page 1 → SILENT
           └─ UI has pager component but currentPage=1 → SILENT
```
