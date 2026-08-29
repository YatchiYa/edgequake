# 00 — Why SPEC-140

> **Cross-refs**: [Issue data](00-issue-data.md) · [Laws](01-first-principles.md) · [RCA](03-root-cause.md)

## Trigger

A field operator on **EdgeQuake 0.24.2** (Docker + PostgreSQL) created three
workspaces:

```text
  g99-71    g99-72    g99-73
```

All three rows exist in `workspaces`, `is_active = true`, and were assigned to
users (`memberships`). The WebUI shows only **g99-73**.

The names are a numbered fleet. The product still ships the same list contract
on **v0.26.3**.

## User impact

| Layer | Impact |
|-------|--------|
| Operator | Cannot switch into a workspace that exists in Postgres |
| Product | “Create succeeded” is a lie if the selector cannot name the row |
| Trust | DB verification vs UI disagreement → “frontend sync bug” tickets |
| Scale | Any tenant with **>20** workspaces (or >20 orgs) silently drops the rest |

## Why this is a product defect

1. **Pagination without an honest total is truncation.** The handler
   `.take(limit)` then reports `total = items.len()`. Clients cannot discover
   remaining pages. OpenAPI already calls `total` the total count.
2. **A context switcher is not an admin table.** The header popover is the
   only way to choose a workspace. If it only ever sees 20 newest rows, older
   named environments (g99-1 …) are unreachable.
3. **The chip is not the list.** LAW-101-11 paints **one** `Tenant — Workspace`
   pair. Auto-select is newest-first. Looking at the chip after creating
   g99-73 is expected to show g99-73 even when 71 and 72 are in the store.
4. **List is tenant-scoped.** Three *organizations* named g99-71/72/73
   yield one workspace row in the selected org. The reporter’s “assigned”
   language is membership; listing ignores membership.

```text
  WHY the operator sees one name
  ───────────────────────────────

  Postgres: 3 (or 73) workspace rows
        │
        ├─► API default page: 20 newest, total lied as 20
        │         └─► UI never asks for page 2
        │
        ├─► Chip: selected = newest = g99-73
        │
        └─► Wrong tenant selected → Workspaces group has 1 row
```

## Non-goals

- Membership-only listing (would hide unassigned workspaces)
- New REST for assign-user-to-workspace
- Schema train / migrations (identity DDL is frozen from 001/008)

## Success condition

1. `GET /tenants/{id}/workspaces` with default params: `items.len() ≤ 20`
   **and** `total == COUNT(*) FROM workspaces WHERE tenant_id = …`.
2. The WebUI selector, after fetch, contains **every** workspace of the
   selected tenant (exhaust pages).
3. Creating three named workspaces on one tenant → popover lists all three
   (search cleared).
4. Creating a 21st workspace → the oldest name is still reachable.
5. Three tenants × one named workspace: Organizations group lists all three
   orgs; selecting each shows that workspace.
