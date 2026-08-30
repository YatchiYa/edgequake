/**
 * SPEC-140 — Merge server list with optimistic extras by stable `id`.
 *
 * Server rows win on collision. Rows with a missing `id` are skipped so
 * `Map.set(undefined, row)` cannot collapse the list to the last item (Track D).
 */

export type Identified = { id?: string | null };

export type TenantScoped = Identified & { tenant_id?: string | null };

/**
 * Keep optimistic extras that belong to `tenantId` (or lack tenant_id).
 * Prevents Track C: previous org's workspaces surviving a tenant switch.
 */
export function extrasInSameTenant<T extends TenantScoped>(
  extras: T[],
  tenantId: string | null | undefined,
): T[] {
  if (!tenantId) return extras;
  return extras.filter((row) => !row.tenant_id || row.tenant_id === tenantId);
}

export function mergeEntitiesById<T extends Identified>(
  server: T[],
  extras: T[] = [],
): T[] {
  const byId = new Map<string, T>();
  for (const row of server) {
    if (!row.id) continue;
    byId.set(row.id, row);
  }
  for (const row of extras) {
    if (!row.id) continue;
    if (!byId.has(row.id)) byId.set(row.id, row);
  }
  return Array.from(byId.values());
}
