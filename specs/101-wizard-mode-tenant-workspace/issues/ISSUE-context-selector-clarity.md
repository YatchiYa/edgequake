# ISSUE — Context selector clarity (Tenant + Workspace)

> **SPEC-101** · **LAW-101-11** · Findings F-101-19…21

## Problem

Users cannot tell which **Tenant** and **Workspace** are active. A two-line cramped chip + mixed list feels messy; selection must be **Organization first, then Workspace**, with a clear one-line readout.

```ascii
BEFORE (messy)
┌ Header ────────────────────────────┐
│ TENANT  spec101…                   │
│ WORKSPACE spec101…               ▾ │  ← cramped two lines
└────────────────────────────────────┘
Popover: Current + Workspaces + Orgs mixed / redundant

AFTER (target)
┌ Header ────────────────────────────┐
│ 📁 Acme — Research               ▾ │  ← one line Tenant — Workspace
└────────────────────────────────────┘
Popover:
  1 · Organization     ← pick tenant (keeps open)
  2 · Workspace · Acme ← then pick workspace (closes)
```

## Acceptance

1. Trigger is **one line**: `Tenant — Workspace` (`context-line`; parts `context-tenant-label` / `context-workspace-label`).
2. `title` / `aria-label` / `data-full-name` include full names; chrome may use end-biased `smartTruncate`.
3. Popover order: **1 · Organization** then **2 · Workspace**; no redundant “Current” strip.
4. Tenant select keeps popover open and reveals/scrolls to Workspaces; workspace select closes.
5. Search: “Search organizations and workspaces…”; empty: “No matches.”
6. Playwright `e2e/spec101-context-selector.spec.ts` green.

## Test IDs

| ID | Surface |
|----|---------|
| `workspace-selector` | Trigger (compat) |
| `context-line` | One-line chip content |
| `context-tenant-label` | Tenant segment (`data-full-name`) |
| `context-workspace-label` | Workspace segment (`data-full-name`) |
| `context-selector-tenants` | Step 1 Organizations |
| `context-selector-workspaces` | Step 2 Workspaces |
| `header-create-tenant` / `header-create-workspace` | Create actions |

## Code

- `components/layout/context-selector/*`
- `lib/layout/format-context-labels.ts`
- Thin `header-tenant-selector.tsx` wiring
