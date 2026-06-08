/**
 * SPEC-020 — browser API routing for UI tests (DRY).
 * First principle: proxy to EQ_BACKEND_URL so :3001 UI works with :8081 backend.
 */
import type { Page } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";
import type { QcWorkspaceContext } from "./qc-workspace";

function tenantHeaders(
  ctx: QcWorkspaceContext,
  existing: Record<string, string>,
): Record<string, string> {
  return {
    ...existing,
    "X-Tenant-ID": ctx.tenantId,
    "X-Workspace-ID": ctx.workspaceId,
  };
}

/** Inject tenant headers on matching API requests (same origin only). */
export async function wireTenantApiHeaders(
  page: Page,
  ctx: QcWorkspaceContext,
): Promise<void> {
  await page.route("**/api/v1/**", async (route) => {
    await route.continue({
      headers: tenantHeaders(ctx, route.request().headers()),
    });
  });
}

/**
 * Proxy all /api/v1 calls to BACKEND_URL — fixes NEXT_PUBLIC_API_URL port drift.
 * Use for UI upload/query proofs when frontend dev server is on :3001.
 */
export async function wireTenantApiProxy(
  page: Page,
  ctx: QcWorkspaceContext,
): Promise<void> {
  await page.route("**/api/v1/**", async (route) => {
    const req = route.request();
    const incoming = new URL(req.url());
    const target = `${BACKEND_URL}${incoming.pathname}${incoming.search}`;
    const headers = tenantHeaders(ctx, req.headers());
    try {
      const response = await route.fetch({
        url: target,
        method: req.method(),
        headers,
        postData: req.postDataBuffer() ?? undefined,
        maxRedirects: 0,
      });
      await route.fulfill({ response });
    } catch {
      await route.continue({ headers });
    }
  });
}
