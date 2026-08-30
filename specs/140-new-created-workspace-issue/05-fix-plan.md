# 05 — Fix plan

> **Cross-refs**: [Laws](01-first-principles.md) · [E2E](06-e2e-test-matrix.md) · [Edges](07-edge-cases.md)

## Locked approach

| Step | Action | Law |
|------|--------|-----|
| 1 | `count_tenants` / `count_workspaces` on `WorkspaceService` | LAW-140-2 |
| 2 | `list_workspaces_page(tenant, limit, offset)` SQL + in-memory `created_at DESC` | LAW-140-4 |
| 3 | Handlers: `total = count`, `items = page` (cap limit at 100) | LAW-140-2 |
| 4 | Keep `list_workspaces(tenant_id)` full for internals | I |
| 5 | `fetchAllPages` in WebUI; `getTenants` / `getWorkspaces` use it | LAW-140-3 |
| 6 | `mergeEntitiesById`; header, guard, TenantProvider | Track E |
| 7 | cmdk `value={id}`, keywords, option testids, search remount, heading count | LAW-140-5 |
| 8 | Chip title includes workspace count (optional, header) | LAW-140-5 |
| 9 | Normalize `id ?? workspace_id` | Track D |
| 10 | Fix stale `GET /api/v1/workspaces` module comment | Honesty |
| 11 | E2E-140-01..06 + unit pages/merge | LAW-140-7 |
| 12 | CHANGELOG Unreleased | Honesty |

## Rejected alternatives

| Idea | Reject reason |
|------|----------------|
| Membership-filter the list | LAW-140-6; hides unassigned catalog rows |
| Raise default limit to 100 only | Still silent-drops at 101; `total` would still lie if unchanged |
| Frontend `limit=100` without honest `total` | Cannot loop; 101st row still gone |
| Auto-membership on create | Does not cause the miss; no REST assign API this pack |
| Client-only “load more” button without COUNT | User cannot know M |

## SOLID mapping

- **S:** trait count / page / full-list; `fetchAllPages` vs merge; DTO map unchanged.
- **O:** tenants already had SQL page — they gain COUNT only.
- **L:** in-memory page order matches Postgres (`created_at DESC`, then skip/take).
- **D:** HTTP e2e uses `AppState::test_state()` (in-memory) so CI does not need
  Postgres for 140-01/02; Playwright remains live-stack gated.

## Implementation notes

### Backend files

- [workspace_service/mod.rs](../../edgequake/crates/edgequake-core/src/workspace_service/mod.rs) — trait
- [service_trait_impl.rs](../../edgequake/crates/edgequake-core/src/workspace_service_impl/service_trait_impl.rs)
- [workspace_ops.rs](../../edgequake/crates/edgequake-core/src/workspace_service_impl/workspace_ops.rs)
- [tenant_ops.rs](../../edgequake/crates/edgequake-core/src/workspace_service_impl/tenant_ops.rs)
- [in_memory.rs](../../edgequake/crates/edgequake-core/src/workspace_service/in_memory.rs)
- [workspace_crud.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/workspace_crud.rs)
- [tenants.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/tenants.rs)
- [handlers/workspaces/mod.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/mod.rs) comment table

### Frontend files

- New: `edgequake_webui/src/lib/api/fetch-all-pages.ts`
- New: `edgequake_webui/src/lib/tenant/merge-entities-by-id.ts`
- [workspaces.ts](../../edgequake_webui/src/lib/api/edgequake/workspaces.ts)
- [header-tenant-selector.tsx](../../edgequake_webui/src/components/layout/header-tenant-selector.tsx)
- [tenant-guard.tsx](../../edgequake_webui/src/components/layout/tenant-guard.tsx)
- [tenant-provider.tsx](../../edgequake_webui/src/providers/tenant-provider.tsx)
- [context-selector-popover.tsx](../../edgequake_webui/src/components/layout/context-selector/context-selector-popover.tsx)
- [context-trigger-chip.tsx](../../edgequake_webui/src/components/layout/context-selector/context-trigger-chip.tsx)

### Tests

- `edgequake/crates/edgequake-api/tests/e2e_spec140_list_pagination.rs`
- `edgequake_webui/e2e/spec140-workspace-list.spec.ts`
- Vitest: `fetch-all-pages`, `merge-entities-by-id`

## Acceptance

- [x] Spec pack (this directory)
- [x] Honest `total` workspaces + tenants
- [x] SQL / in-memory page
- [x] Selector exhausts pages
- [x] Merge DRY
- [x] cmdk + count + testids
- [x] E2E-140-01..06
- [x] GitHub #388 comment
- [x] CHANGELOG Unreleased
