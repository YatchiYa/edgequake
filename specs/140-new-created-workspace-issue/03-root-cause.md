# 03 — Root cause (code is law)

> Pre-fix = v0.26.3 (and 0.24.2) list handlers. Post-fix = this pack.

## Track A — Silent 20-cap + lying `total` (must-fix)

Handler ([workspace_crud.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/workspace_crud.rs)):

```text
  list_workspaces(tenant_id)     -- full scan, ORDER BY created_at DESC
           │
           ▼
  skip(offset).take(limit.min(100))     default limit = 20
           │
           ▼
  total = items.len()                   always ≤ 20
```

SQL has **no** LIMIT. Tenants already `LIMIT/OFFSET` in SQL but still set
`total = items.len()` ([tenants.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/tenants.rs)).

WebUI `getWorkspaces(tenantId)` calls `/tenants/${id}/workspaces` with **no**
query and returns `response.items`. Admin quota already uses `/tenants?limit=100`.

```text
  Tenant with 73 workspaces (g99-1 … g99-73), newest = 73

  Default GET: items = g99-73 … g99-54 (20 rows), total=20
  UI: cannot discover g99-1 … g99-53
  g99-71 and g99-72 ARE in that newest-20 window if they are 2nd/3rd newest.

  Track A alone does not hide 71/72 when they are the last three created.
  It DOES hide any older sibling and makes the product lie at N>20.
```

## Track B — Chip is current; auto-select newest

```text
  List order: created_at DESC → [g99-73, g99-72, g99-71, …]
  Auto-select workspaces[0]  → g99-73
  Chip (data-testid=workspace-selector): ONE name (LAW-101-11)

  If the operator never opens the popover, only g99-73 is “displayed”.
```

Popover maps **every** store row — no `.slice()`. Existing Playwright
`workspace-selection.spec.ts` looks for `menuitem`; cmdk uses `option`.

## Track C — Tenant-scoped list (numbered orgs)

```text
  Create 3 TENANTS named g99-71, g99-72, g99-73
  (wizard PATCHes each org’s Default Workspace to that name)

  GET /tenants? (default 20, total lied)
  Auto-select tenants[0] = newest org = g99-73

  Workspaces group: GET /tenants/{g99-73}/workspaces → 1 row

  Postgres still has 3 workspace rows (3 tenant_ids).
  “Assigned to users” = memberships per tenant. Matches DevOps slots.
```

## Track D — Identity collapse (defense)

```text
  byId.set(w.id, w)     -- if id is undefined, one Map key
  CommandItem value={`ws:${name} ${slug}`}
  React key={workspace.id}
```

API DTO serializes `id` ([map.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces_types/map.rs)).
Harden: `value={id}`, `keywords={[name,slug]}`, skip missing ids in merge.

cmdk requires a unique `value` ([docs](https://www.npmjs.com/package/cmdk)).

## Track E — Store race

```text
  applyCreatedWorkspaceContext  → merge into store
  header / tenant-guard          → merge server ∪ store
  TenantProvider                → setWorkspaces(server)   ◄── no merge
  applyCreatedTenantContext     → setWorkspaces([one])
  staleTime 60s
```

After creating tenant 3, the new query key is that tenant’s workspaces (1 row).
That is correct for Track C. Overwrite without merge can drop an optimistic
sibling on the **same** tenant if a stale page returns first.

## Track F — Rejected on HEAD

| Hypothesis | Why rejected |
|-----------|----------------|
| INNER JOIN memberships drops 71/72 | List SQL has no join |
| `is_active` filter | List does not filter `is_active` |
| Unique slug overwrite | Unique rejects (409), does not upsert; 3 rows remain |
| Unique “current workspace” column | Does not exist; selection is client localStorage |
| RLS on `workspaces` | Catalog tables are not RLS-forced |
| SQL `LIMIT 1` on list | Not present |
| Schema change 0.24.2 → 0.26.3 | None on `workspaces` / `memberships` |

## What existing tests miss

| Test | Gap |
|------|-----|
| `e2e_postgres_workspace::test_list_workspaces_by_tenant` | Raw SQL, not HTTP |
| `e2e_api_comprehensive::test_list_workspaces_for_tenant` | HTTP 200 only |
| `workspace-selection.spec.ts` | `menuitem` vs cmdk `option`; no 3-name assert |

## Field symptom vs tracks

The report (“only 73 of the last three”) is **over-determined**: B and C
match exactly; A is a separate unfakable scale bug. This pack fixes A–E.
Membership-only listing is **not** the fix (LAW-140-6).
