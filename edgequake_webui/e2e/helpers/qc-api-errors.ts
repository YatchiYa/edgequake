/**
 * SPEC-020 API error-path helpers (SRP).
 */
import { expect } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";
import { tenantHeaders } from "./spec013-api";
import type { QcWorkspaceContext } from "./qc-workspace";

/** Malformed document upload must not succeed silently. */
export async function assertMalformedUploadRejected(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
): Promise<number> {
  const res = await request.post(`${BACKEND_URL}/api/v1/documents`, {
    headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
    data: { invalid_field: "no title or content" },
  });
  expect([400, 422, 415]).toContain(res.status());
  return res.status();
}

/** Empty content upload must be rejected or no-op safe. */
export async function assertEmptyContentUploadRejected(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
): Promise<{ status: number; rejected: boolean }> {
  const res = await request.post(`${BACKEND_URL}/api/v1/documents`, {
    headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
    data: { title: "spec020-empty", content: "", async_processing: false },
  });
  const rejected = [400, 422].includes(res.status());
  if (!rejected && res.ok()) {
    const body = (await res.json()) as { status?: string };
    return { status: res.status(), rejected: body.status === "failed" };
  }
  return { status: res.status(), rejected };
}
