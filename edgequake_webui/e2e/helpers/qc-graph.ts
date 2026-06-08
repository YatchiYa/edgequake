/**
 * SPEC-020 graph/workspace stats helpers (SRP).
 */
import { expect } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";
import { tenantHeaders } from "./spec013-api";
import type { QcWorkspaceContext } from "./qc-workspace";

export type WorkspaceGraphStats = {
  entityCount: number;
  relationshipCount: number;
  documentCount: number;
  chunkCount: number;
};

export type GraphSearchResult = {
  matchCount: number;
  labels: string[];
};

export async function fetchWorkspaceGraphStats(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
): Promise<WorkspaceGraphStats> {
  const res = await request.get(
    `${BACKEND_URL}/api/v1/workspaces/${ctx.workspaceId}/stats`,
    { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
  );
  expect(res.ok()).toBeTruthy();
  const body = (await res.json()) as {
    entity_count?: number;
    relationship_count?: number;
    document_count?: number;
    chunk_count?: number;
  };
  return {
    entityCount: body.entity_count ?? 0,
    relationshipCount: body.relationship_count ?? 0,
    documentCount: body.document_count ?? 0,
    chunkCount: body.chunk_count ?? 0,
  };
}

export async function pollWorkspaceStats(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  predicate: (s: WorkspaceGraphStats) => boolean,
  timeoutMs = 90_000,
): Promise<WorkspaceGraphStats> {
  const deadline = Date.now() + timeoutMs;
  let latest = await fetchWorkspaceGraphStats(request, ctx);
  while (Date.now() < deadline) {
    if (predicate(latest)) return latest;
    await new Promise((r) => setTimeout(r, 3000));
    latest = await fetchWorkspaceGraphStats(request, ctx);
  }
  return latest;
}

export async function searchGraphNodes(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  query: string,
  limit = 20,
): Promise<GraphSearchResult> {
  const res = await request.get(
    `${BACKEND_URL}/api/v1/graph/nodes/search?q=${encodeURIComponent(query)}&limit=${limit}`,
    { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
  );
  if (!res.ok()) {
    return { matchCount: 0, labels: [] };
  }
  const body = (await res.json()) as {
    nodes?: Array<{ label?: string; id?: string }>;
  };
  const nodes = body.nodes ?? [];
  return {
    matchCount: nodes.length,
    labels: nodes.map((n) => n.label ?? n.id ?? "").filter(Boolean),
  };
}

/** Empty graph search must not 500 — safe empty or validation response. */
export async function assertEmptyGraphSearchSafe(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
): Promise<{ status: number; nodeCount: number }> {
  const res = await request.get(
    `${BACKEND_URL}/api/v1/graph/nodes/search?q=&limit=5`,
    { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
  );
  expect(res.status()).toBeLessThan(500);
  if (!res.ok()) {
    return { status: res.status(), nodeCount: 0 };
  }
  const body = (await res.json()) as { nodes?: unknown[] };
  const nodes = body.nodes ?? [];
  return { status: res.status(), nodeCount: nodes.length };
}

/**
 * Entity extraction proof: ingest entity_count > 0 AND workspace stats sync.
 */
export async function assertEntityExtractionProof(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  ingest: () => Promise<{ entityCount: number; chunkCount: number }>,
): Promise<{
  before: WorkspaceGraphStats;
  after: WorkspaceGraphStats;
  entityDelta: number;
  ingestEntityCount: number;
  search: GraphSearchResult;
  statsSynced: boolean;
}> {
  const before = await fetchWorkspaceGraphStats(request, ctx);
  const uploaded = await ingest();
  expect(uploaded.entityCount).toBeGreaterThan(0);

  const after = await pollWorkspaceStats(
    request,
    ctx,
    (s) =>
      s.documentCount > before.documentCount &&
      s.entityCount > before.entityCount,
    90_000,
  );
  const entityDelta = after.entityCount - before.entityCount;
  expect(entityDelta).toBeGreaterThan(0);

  const search = await searchGraphNodes(request, ctx, "SARAH");
  const hasSarah =
    search.matchCount > 0 &&
    search.labels.some((l) => /SARAH/i.test(l));
  expect(hasSarah).toBeTruthy();

  return {
    before,
    after,
    entityDelta,
    ingestEntityCount: uploaded.entityCount,
    search,
    statsSynced: true,
  };
}

/** Workspace B stats stay empty after ingest only in workspace A. */
export async function assertWorkspaceStatsIsolated(
  request: import("@playwright/test").APIRequestContext,
  owner: QcWorkspaceContext,
  other: QcWorkspaceContext,
  ingest: () => Promise<{ entityCount: number; chunkCount: number }>,
): Promise<{
  ownerStats: WorkspaceGraphStats;
  otherStats: WorkspaceGraphStats;
  ingestEntityCount: number;
}> {
  const otherBefore = await fetchWorkspaceGraphStats(request, other);
  expect(otherBefore.documentCount).toBe(0);

  const uploaded = await ingest();
  expect(uploaded.entityCount).toBeGreaterThan(0);

  const ownerStats = await pollWorkspaceStats(
    request,
    owner,
    (s) => s.documentCount > 0 && s.entityCount > 0,
    90_000,
  );
  const otherStats = await fetchWorkspaceGraphStats(request, other);

  expect(ownerStats.documentCount).toBeGreaterThan(0);
  expect(ownerStats.entityCount).toBeGreaterThan(0);
  expect(otherStats.documentCount).toBe(0);
  expect(otherStats.entityCount).toBe(0);
  expect(otherStats.chunkCount).toBe(0);

  return {
    ownerStats,
    otherStats,
    ingestEntityCount: uploaded.entityCount,
  };
}
