/**
 * SPEC-101 — DRY post-create tenant/workspace selection for header + tenant-guard.
 */

import { setTenantContext } from '@/lib/api/client-context';
import { useTenantStore } from '@/stores/use-tenant-store';
import type { Tenant, Workspace } from '@/types';

export type SyncCreatedContextUrlFn = (
  workspace: {
    id: string;
    name: string;
    slug?: string | null;
  },
  tenant?: {
    id: string;
    name: string;
    slug?: string | null;
  },
) => void;

function deriveSlug(name: string, slug?: string | null, fallback?: string): string {
  return (
    slug ||
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '') ||
    fallback ||
    name
  );
}

/**
 * After Create Tenant: select tenant + its configured default workspace.
 */
export function applyCreatedTenantContext(
  tenant: Tenant,
  workspace: Workspace,
  syncUrl?: SyncCreatedContextUrlFn,
): void {
  const store = useTenantStore.getState();
  store.selectTenant(tenant.id);
  // Keep tenant object available for inheritance / selector display.
  if (!store.tenants.some((t) => t.id === tenant.id)) {
    store.setTenants([...store.tenants, tenant]);
  }
  store.setWorkspaces([workspace]);
  store.selectWorkspace(workspace.id);
  setTenantContext(tenant.id, workspace.id);
  syncUrl?.(workspace, tenant);
}

/**
 * After Create Workspace: merge into list and select the new workspace.
 */
export function applyCreatedWorkspaceContext(
  workspace: Workspace,
  syncUrl?: SyncCreatedContextUrlFn,
): void {
  const store = useTenantStore.getState();
  const existing = store.workspaces;
  const merged = existing.some((w) => w.id === workspace.id)
    ? existing.map((w) => (w.id === workspace.id ? workspace : w))
    : [...existing, workspace];
  store.setWorkspaces(merged);
  store.selectWorkspace(workspace.id);
  const tenant = store.tenants.find((t) => t.id === store.selectedTenantId);
  if (store.selectedTenantId) {
    setTenantContext(store.selectedTenantId, workspace.id);
  }
  syncUrl?.(workspace, tenant);
}

/** Build ?tenant=&workspace= from created entities (header sync helper). */
export function buildCreatedContextSearchParams(
  currentSearch: string,
  workspace: { id: string; name: string; slug?: string | null },
  tenant?: { id: string; name: string; slug?: string | null },
): string {
  const params = new URLSearchParams(currentSearch);
  params.set('workspace', deriveSlug(workspace.name, workspace.slug, workspace.id));
  if (tenant) {
    params.set('tenant', deriveSlug(tenant.name, tenant.slug, tenant.id));
  }
  return params.toString();
}
