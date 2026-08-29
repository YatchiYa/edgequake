# 02 — Cross-ref matrix

> **Cross-refs**: [Laws](01-first-principles.md) · [RCA](03-root-cause.md) · [E2E](06-e2e-test-matrix.md)

| This pack | Code / spec SSOT | Contract |
|-----------|-------------------|----------|
| LAW-140-1 | [header-tenant-selector.tsx](../../edgequake_webui/src/components/layout/header-tenant-selector.tsx) · [context-selector-popover.tsx](../../edgequake_webui/src/components/layout/context-selector/context-selector-popover.tsx) | E2E-140-03 / 04 |
| LAW-140-2 | [workspace_crud.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/workspace_crud.rs) `list_workspaces` · [tenants.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/tenants.rs) `list_tenants` | E2E-140-01 / 02 |
| LAW-140-3 | [workspaces.ts](../../edgequake_webui/src/lib/api/edgequake/workspaces.ts) `getWorkspaces` / `getTenants` · `fetchAllPages` | U-140-PAGES · E2E-140-04 |
| LAW-140-4 | [workspace_ops.rs](../../edgequake/crates/edgequake-core/src/workspace_service_impl/workspace_ops.rs) · [tenant_ops.rs](../../edgequake/crates/edgequake-core/src/workspace_service_impl/tenant_ops.rs) · [in_memory.rs](../../edgequake/crates/edgequake-core/src/workspace_service/in_memory.rs) | E2E-140-01 (in-memory HTTP) |
| LAW-140-5 | popover cmdk `value={id}` · chip title + count | E2E-140-03 · SPEC-101 chip tests |
| LAW-140-6 | SQL `WHERE tenant_id = $1` — no `memberships` join | E2E-140-01 tenant isolation |
| LAW-140-7 | This matrix vs tests | If they disagree, **HTTP+DOM tests win** |
| Merge | `mergeEntitiesById` · header · tenant-guard · TenantProvider | U-140-MERGE |
| DTO `id` | [map.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces_types/map.rs) `id: workspace.workspace_id` | normalize `id ?? workspace_id` |
| SPEC-101 | [EC-101-15](../101-wizard-mode-tenant-workspace/05-edge-cases.md) | Tenant switch must reload that org’s workspaces |
| Quota | [quota_ops.rs](../../edgequake/crates/edgequake-core/src/workspace_service_impl/quota_ops.rs) · plan Pro=500 | E2E-140-01 uses `plan=pro` |
| OpenAPI | `WorkspaceListResponse.total` | Snapshot must stay “total count” |
| Stale docs | [handlers/workspaces/mod.rs](../../edgequake/crates/edgequake-api/src/handlers/workspaces/mod.rs) still says `GET /api/v1/workspaces` | Fix comment only |

## Divergence rule

If this matrix and a code comment disagree, **code + contract test** win.
Update this file in the same PR as the code change.
