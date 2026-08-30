# SPEC-140 — Newly created workspaces missing in UI (#388)

> **Trigger:** GitHub [#388](https://github.com/raphaelmansuy/edgequake/issues/388).
> Field on **v0.24.2** Docker: workspaces `g99-71`, `g99-72`, `g99-73` exist in
> Postgres (`is_active=true`, assigned). Only **g99-73** appears in the WebUI.
> **Method:** First principles (code is law) + unfakable HTTP + Playwright.
> **Broken through:** **v0.24.2** (and HEAD until this pack). Same list contract
> is still in **v0.26.3** — no workspace DDL change.
> **Target cut:** next patch after 0.26.3.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  A context switcher that cannot name every selectable workspace is broken.    │
│                                                                              │
│  Track A: GET /tenants/{id}/workspaces defaults limit=20 and sets               │
│           total = items.len() (always ≤ 20). WebUI never sends limit and     │
│           never follows pages. Same lie on GET /tenants.                   │
│  Track B: Header chip shows the selected workspace only. Auto-select [0]    │
│           = newest (ORDER BY created_at DESC) → last created name (g99-73). │
│  Track C: List is tenant-scoped. Three orgs named g99-71/72/73 →            │
│           Workspaces group has one row for the selected (newest) org.        │
│                                                                              │
│  Membership does NOT feed the list (no JOIN). Assignment is why the           │
│  reporter expected the rows — not why they vanish.                           │
│                                                                              │
│  Fix: honest COUNT total; SQL LIMIT/OFFSET; selector exhausts pages;        │
│       mergeById; cmdk unique id; popover count; unfakable e2e.              │
│  Do NOT switch the list to membership-only (hides unassigned admin rows).     │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | `total = items.len()` after `.take(limit)` | **Fixed** | COUNT + SQL page; [03](03-root-cause.md) Track A |
| F2 | WebUI `getWorkspaces` omits `limit` / next page | **Fixed** | `fetchAllPages` in [workspaces.ts](../../edgequake_webui/src/lib/api/edgequake/workspaces.ts) |
| F3 | Chip ≠ list; auto-select newest | **Code** | LAW-101-11 + [03](03-root-cause.md) Track B |
| F4 | List is tenant-scoped, not membership | **Code** | SQL `WHERE tenant_id = $1` |
| F5 | TenantProvider overwrite vs header merge | **Fixed** | Track E — `mergeEntitiesById` |
| E2E | Unfakable 140-01..06 | **Passing** | [06](06-e2e-test-matrix.md) |

## Document map

```ascii
 00-why / 00-issue-data
   → 01-first-principles (LAW-140-1..7)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 04-lenses (PO / Full Stack / DB / UX / Front / AI)
   → 05-fix-plan
   → 06-e2e-test-matrix
   → 07-edge-cases
   → 08-similar-issues
   → 09-github-reply
   → 10-ops-workaround
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| List scope | Tenant-scoped (not membership-only) |
| `total` | `COUNT(*)` for that resource, never page length |
| Selector client | Exhaust pages at `limit=100` until `len >= total` |
| Workspace SQL | `ORDER BY created_at DESC LIMIT/OFFSET` (tenants already paginate in SQL) |
| cmdk | `value={id}`, `keywords={[name, slug]}` |
| Merge | One `mergeEntitiesById` (server wins, extras fill gaps, skip missing id) |
| Membership REST / auto-assign on create | Out of scope |

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|----------|
| [SPEC-101](../101-wizard-mode-tenant-workspace/) | Dual-label chip LAW-101-11; EC-101-15 tenant switch |
| [SPEC-017](../017-ui-dry/) | DRY client helpers |
| [#388](https://github.com/raphaelmansuy/edgequake/issues/388) | Field report |
| [cmdk unique value](https://www.npmjs.com/package/cmdk) | Items need a unique `value` |

## DRY rule

Pagination honesty is **one** contract: `items` = page, `total` = table count,
`offset`/`limit` echoed. Tenants and workspaces share it. Selector fetch-all
is one helper. Merge-by-id is one helper. If OpenAPI and handler disagree,
**the HTTP e2e wins**.

## Out of scope

- Membership-filtered “my workspaces” payload
- Auto-insert membership on `create_workspace`
- Raising API max limit above 100
- Acc re-score / AGE / PDF geometry

## Start here

1. [00-why.md](00-why.md)
2. [00-issue-data.md](00-issue-data.md)
3. [01-first-principles.md](01-first-principles.md)
4. [03-root-cause.md](03-root-cause.md)
5. [05-fix-plan.md](05-fix-plan.md)
6. [09-github-reply.md](09-github-reply.md)
