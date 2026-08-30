# 06 — E2E test matrix

> **Cross-refs**: [Laws](01-first-principles.md) · [Edges](07-edge-cases.md)

Unfakable = the assertion fails if the handler still sets `total = items.len()`
or the popover never received page 2.

| ID | Layer | Setup | Assert |
|----|-------|--------|--------|
| **E2E-140-01** | HTTP in-memory | Tenant `plan=pro`; POST 25 extra workspaces (plus auto Default) | Default GET: `items.len()==20`, `total == items_from_limit100.len()`, `total >= 26`. `?limit=100`: all names present including last three. Each item `tenant_id` matches. |
| **E2E-140-02** | HTTP in-memory | POST 25 tenants | Default GET `/tenants`: `items.len()==20`, `total >= 25` and `total > items.len()`. `?limit=100` returns ≥25. |
| **E2E-140-03** | Playwright live | Same tenant; API-create `g99-71`, `g99-72`, `g99-73` (unique suffix) | Open selector; clear search; all three `workspace-option-{slug}` visible. Intercept list GET: names in JSON, `total >= 3`. Chip may show only the selected name. |
| **E2E-140-04** | Playwright live | 21 workspaces on one tenant (quota ok) | Oldest name reachable via search. Fail if options==20 and intercepted `total>=21`. |
| **E2E-140-05** | Playwright live | 3 tenants × 1 named workspace | `tenant-option-*` for all three orgs. Select each → matching workspace option. |
| **U-140-PAGES** | Vitest | Mock fetchPage | Loops until `len>=total`; stops on short page; safety cap. |
| **U-140-MERGE** | Vitest | Server 2 + extra 1; missing id | Distinct ids preserved; `undefined` id skipped (no collapse to last). |

## Run

```bash
# HTTP (no Docker required — AppState::test_state)
cargo test -p edgequake-api --test e2e_spec140_list_pagination

# Playwright (live stack)
cd edgequake_webui && pnpm exec playwright test e2e/spec140-workspace-list.spec.ts

# Units
cd edgequake_webui && bun test src/lib/api/__tests__/fetch-all-pages.test.ts \
  src/lib/tenant/__tests__/merge-entities-by-id.test.ts
```

Playwright uses `skipUnlessLiveStack()` like SPEC-101.

## Anti-patterns (do not add)

- Asserting chip text equals all three names
- `SELECT COUNT` without GET
- `toBeVisible` on the first option only
