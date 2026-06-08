/**
 * SPEC-020 multi-tenant isolation helpers (SRP: cross-tenant leak checks only).
 */
import { expect } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";
import { tenantHeaders } from "./spec013-api";
import { createMockQcWorkspace, type QcWorkspaceContext } from "./qc-workspace";
import { listDocumentTitles } from "./qc-documents";

export type DualTenantContexts = {
  tenantA: QcWorkspaceContext;
  tenantB: QcWorkspaceContext;
};

export async function createDualTenantContexts(
  request: import("@playwright/test").APIRequestContext,
  label: string,
): Promise<DualTenantContexts> {
  const tenantA = await createMockQcWorkspace(request, `${label}-a`);
  const tenantB = await createMockQcWorkspace(request, `${label}-b`);
  expect(tenantA.tenantId).not.toBe(tenantB.tenantId);
  expect(tenantA.workspaceId).not.toBe(tenantB.workspaceId);
  return { tenantA, tenantB };
}

/** Document uploaded in tenant A must not appear in tenant B list. */
export async function assertCrossTenantDocumentIsolation(
  request: import("@playwright/test").APIRequestContext,
  owner: QcWorkspaceContext,
  other: QcWorkspaceContext,
  documentTitle: string,
): Promise<void> {
  const ownerTitles = await listDocumentTitles(
    request,
    owner.tenantId,
    owner.workspaceId,
  );
  expect(ownerTitles).toContain(documentTitle);

  const otherTitles = await listDocumentTitles(
    request,
    other.tenantId,
    other.workspaceId,
  );
  expect(otherTitles).not.toContain(documentTitle);
}

/**
 * Unscoped document list must not leak tenant data.
 * Dev mode may return 200 + empty (default tenant) rather than 4xx — both are safe.
 */
export async function assertUnscopedDocumentsRequestSafe(
  request: import("@playwright/test").APIRequestContext,
): Promise<{ status: number; safe: boolean; documentCount: number }> {
  const res = await request.get(`${BACKEND_URL}/api/v1/documents`);
  const status = res.status();
  const isRejected = [400, 401, 403, 422].includes(status);
  if (isRejected) {
    return { status, safe: true, documentCount: 0 };
  }
  if (status === 200) {
    const body = (await res.json()) as {
      documents?: unknown[];
      items?: unknown[];
      total?: number;
    };
    const docs = body.documents ?? body.items ?? [];
    const count = Array.isArray(docs) ? docs.length : (body.total ?? 0);
    // Safe: empty list (default tenant) — not a cross-tenant leak vector in QC workspace
    return { status, safe: count === 0, documentCount: count };
  }
  return { status, safe: false, documentCount: -1 };
}

/** Wrong tenant + foreign workspace must not leak data (403/404 or empty). */
export async function assertInvalidTenantWorkspaceRejected(
  request: import("@playwright/test").APIRequestContext,
  wrongTenantId: string,
  foreignWorkspaceId: string,
): Promise<{ status: number; empty: boolean }> {
  const res = await request.get(`${BACKEND_URL}/api/v1/documents`, {
    headers: tenantHeaders(wrongTenantId, foreignWorkspaceId),
  });
  const status = res.status();
  const isRejected = status === 403 || status === 404;
  const isEmpty = status === 200;
  expect(isRejected || isEmpty).toBeTruthy();
  if (isEmpty) {
    const body = (await res.json()) as {
      documents?: unknown[];
      items?: unknown[];
    };
    const docs = body.documents ?? body.items ?? (Array.isArray(body) ? body : []);
    return { status, empty: Array.isArray(docs) && docs.length === 0 };
  }
  return { status, empty: false };
}
