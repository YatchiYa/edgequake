# 04 — Lenses

> **Cross-refs**: [WHY](00-why.md) · [Laws](01-first-principles.md) · [Fix](05-fix-plan.md)

## Product Owner

**Job:** An operator who created g99-71/72/73 can **switch into each** from
the header without SQL.

```text
  MUST
    • Every workspace of the selected org is in the popover
    • Chip shows current only, but heading/count makes inventory obvious
    • Organizations with >20 tenants are also complete (same contract)

  MUST NOT
    • Hide unassigned workspaces (admins list the tenant catalog)
    • Ship a “sync” story — the catalog was never incomplete in Postgres

  COPY
    • Assigned ≠ listed. Listing is the tenant catalog.
    • Workaround until upgrade: GET ?limit=100 (see 10-ops-workaround)
```

Success = E2E-140-03/04/05 green on a live stack.

## Full Stack Developer

```text
  API                         WebUI
  ────                        ─────
  COUNT + page                fetchAllPages(limit=100)
  total honest                mergeEntitiesById
  list_workspaces(full)       TenantProvider = same merge as header
  stays for internals         cmdk value=id
```

Do not duplicate skip/take in the handler **and** SQL. Handler: count +
`list_workspaces_page`. Internals keep `list_workspaces(tenant_id)`.

Trait additions: `count_tenants`, `count_workspaces`, `list_workspaces_page`.
Both `InMemoryWorkspaceService` and `WorkspaceServiceImpl` implement them.

## Database Expert

```text
  workspaces
    PK workspace_id
    UNIQUE (tenant_id, slug)     -- reject, not upsert
    INDEX tenant_id
    INDEX is_active               -- unused by list (LAW: show inactive too)

  List (post-fix):
    SELECT ... FROM workspaces
     WHERE tenant_id = $1
     ORDER BY created_at DESC
     LIMIT $2 OFFSET $3

    SELECT COUNT(*) FROM workspaces WHERE tenant_id = $1

  memberships
    UNIQUE (user_id, tenant_id, workspace_id)
    workspace_id NULL = all workspaces in tenant
    NOT referenced by list SQL
```

No migration. Identity DDL frozen. `created_at` ties need a stable secondary
order — `workspace_id` as tie-breaker is allowed if tests need determinism;
not required for #388.

## UX / UI designer

```text
  ┌─────────────────────────────────────────┐
  │  OrgName — g99-73                    v  │  ← chip (current)
  └─────────────────────────────────────────┘
           │ open
           ▼
  ┌─────────────────────────────────────────┐
  │  Search organizations and workspaces │  ← cleared on open
  │  1 · Organization                        │
  │     … all orgs (count in heading)        │
  │  2 · Workspace · OrgName (N)            │  ← N = store length
  │     g99-73  ✓                           │
  │     g99-72                              │
  │     g99-71                              │
  └─────────────────────────────────────────┘
```

Chip `title` may append workspace count so hover is not “the list”.
`data-testid="workspace-option-{slug}"` / `tenant-option-{slug}` for e2e.

## Front designer

- cmdk: unique `value={id}`; `keywords` for name/slug search (do not put UUID
  in the visible label).
- One `CommandInput` still filters both groups; leftover “73” hides 71/72 →
  remount Command on open.
- Stale `workspace-selection` e2e (`menuitem`) is not the SSOT; use
  `[data-testid=workspace-option-…]` / `[cmdk-item]`.

## AI Engineer

Unfakable means the model cannot “pass” by asserting SQL while the handler
still lies:

| Fake | Why it fails LAW-140-7 |
|------|-------------------------|
| `SELECT COUNT(*)` in a test that never GET-lists | Handler `total` untested |
| Playwright: chip text includes 73 | Track B always true |
| HTTP 200 only | `e2e_api_comprehensive` already does this |
| Mock `getWorkspaces` returning 3 names | Never hits skip/take |

Required: real handler JSON `total` vs `items.length` at N=25; Playwright
opens popover and asserts three option testids **and** intercepts list GET.
