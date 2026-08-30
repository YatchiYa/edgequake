# 01 — First principles (LAW-140)

> **Cross-refs**: [WHY](00-why.md) · [RCA](03-root-cause.md) · [Fix](05-fix-plan.md)

## Axioms

1. **A name that exists in the catalog must be choosable.** The header
   popover is the product’s workspace list. Truncation without a remaining
   count is data loss, not pagination.
2. **`total` means COUNT, not page length.** OpenAPI and `WorkspaceListResponse`
   already say so. Implementing `total = items.len()` after `.take(n)` is a
   lie: the client cannot know to request `offset=20`.
3. **Chip ≠ inventory.** One selected pair (LAW-101-11) is correct chrome.
   The popover must still enumerate every workspace of the **selected tenant**.
4. **Tenant scope is intentional.** Workspaces belong to one tenant.
   Cross-tenant “my memberships” is a different product surface. This pack does
   not hide unassigned rows.
5. **Membership is not the list query.** `create_workspace` does not insert
   `memberships`. Listing must not wait on assignment to show a created row.
6. **Identity is UUID.** Map/React/cmdk keys must be `id` (workspace UUID),
   never display name alone. Missing `id` must not collapse the list to the
   last row (`Map.set(undefined, …)` last-write-wins).

## Causal diagram

```text
  CREATE workspace (g99-71, then 72, then 73)
           │
           ▼
  INSERT workspaces (random UUID PK, name/slug as typed)
           │
           ▼
  GET list (no ?limit)  ── default limit=20, ORDER BY created_at DESC
           │
           ├─ handler: skip/take → items = newest 20
           ├─ total = items.len()     ◄── LIE (LAW-140-2)
           │
           ▼
  WebUI getWorkspaces()  ── no query, no loop
           │
           ├─ store.workspaces = that page (or TenantProvider overwrite)
           ├─ chip = selected = [0] = newest = g99-73
           └─ popover maps store (incomplete if N>20; one org if Track C)
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-140-1** | **Completeness** — For the selected tenant, the selector store contains every `workspaces` row (all pages). Unreachable catalog rows are a defect. |
| **LAW-140-2** | **Honest total** — List JSON `total` equals `COUNT(*)` for that tenant (workspaces) or all tenants (tenants). Never `items.len()`. |
| **LAW-140-3** | **Selector exhausts pages** — `getWorkspaces` / `getTenants` request `limit=100` (API max) and loop until `accumulated >= total` or a short last page. |
| **LAW-140-4** | **SQL page = handler page** — Workspace list LIMIT/OFFSET lives in SQL (same as tenants). In-memory slices the same order: `created_at DESC`. |
| **LAW-140-5** | **Chip is current only** — Popover heading shows **count**. Search clears on open. cmdk `value` is the UUID. |
| **LAW-140-6** | **Tenant scope stays** — List is not membership-filtered. Document assigned vs listed. |
| **LAW-140-7** | **Unfakable proof** — HTTP hits the handler (not raw SQL). Playwright asserts **DOM names** and the list GET `total`. Tests that only `SELECT COUNT` from Postgres are not sufficient. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | Count vs page vs DTO map stay separate. Merge helper does not fetch. |
| **O** | New list resources reuse the same `{items,total,offset,limit}` contract. |
| **L** | In-memory list page has the same order and total honesty as Postgres. |
| **I** | Internal `list_workspaces(tenant_id)` (full) stays for non-HTTP callers. |
| **D** | E2E drives HTTP + DOM, not a second parser of SQL. |
| **DRY** | One `fetchAllPages`; one `mergeEntitiesById`; one COUNT+page pattern for tenants and workspaces. |
