/**
 * QC workspace factory — deterministic workspaces for reproducible E2E.
 * @implements SPEC-020 — isolates workspace creation (SRP) from test orchestration.
 */
import type { Page } from "@playwright/test";
import { expect } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";
import { wireTenantApiProxy } from "./qc-api-route";
import { seedTenantStoreOnPage } from "./spec013-bootstrap";
import { tenantHeaders } from "./spec013-api";
import {
  OLLAMA_EMBEDDING_MODEL,
  resolveOllamaLlmModel,
} from "./llm-availability";

export type QcWorkspaceContext = {
  tenantId: string;
  workspaceId: string;
  workspaceName: string;
  workspaceSlug: string;
  llmProvider?: string;
};

const DEFAULT_ENTITY_TYPES = [
  "PERSON",
  "ORGANIZATION",
  "TECHNOLOGY",
  "CONCEPT",
] as const;

type WorkspaceProvider = "mock" | "ollama";

async function createQcWorkspace(
  request: import("@playwright/test").APIRequestContext,
  label: string,
  provider: WorkspaceProvider,
): Promise<QcWorkspaceContext> {
  const suffix = Date.now();
  const tenantRes = await request.post(`${BACKEND_URL}/api/v1/tenants`, {
    data: { name: `${label} tenant ${suffix}` },
  });
  expect(tenantRes.ok()).toBeTruthy();
  const tenant = (await tenantRes.json()) as { id: string };

  const workspaceName = `${label} ws ${suffix}`;
  const workspaceSlug = `${label}-${suffix}`.toLowerCase().replace(/[^a-z0-9]+/g, "-");
  let payload: Record<string, unknown>;

  if (provider === "mock") {
    payload = {
      name: workspaceName,
      slug: workspaceSlug,
      llm_provider: "mock",
      llm_model: "mock-model",
      embedding_provider: "mock",
      embedding_model: "mock-embedding",
      embedding_dimension: 1536,
      entity_types: [...DEFAULT_ENTITY_TYPES],
    };
  } else {
    const llmModel = await resolveOllamaLlmModel();
    payload = {
      name: workspaceName,
      slug: workspaceSlug,
      llm_provider: "ollama",
      llm_model: llmModel,
      embedding_provider: "ollama",
      embedding_model: OLLAMA_EMBEDDING_MODEL,
      embedding_dimension: 768,
      entity_types: [...DEFAULT_ENTITY_TYPES],
    };
  }

  const wsRes = await request.post(
    `${BACKEND_URL}/api/v1/tenants/${tenant.id}/workspaces`,
    { data: payload },
  );
  expect(wsRes.ok()).toBeTruthy();
  const ws = (await wsRes.json()) as { id: string };
  return {
    tenantId: tenant.id,
    workspaceId: ws.id,
    workspaceName,
    workspaceSlug,
    llmProvider: provider,
  };
}

/** Mock LLM workspace — reliable ingestion without external API keys. */
export async function createMockQcWorkspace(
  request: import("@playwright/test").APIRequestContext,
  label: string,
): Promise<QcWorkspaceContext> {
  return createQcWorkspace(request, label, "mock");
}

/** Ollama workspace — live LLM grounding proofs when Ollama is reachable. */
export async function createOllamaQcWorkspace(
  request: import("@playwright/test").APIRequestContext,
  label: string,
): Promise<QcWorkspaceContext> {
  return createQcWorkspace(request, label, "ollama");
}

export async function syncUploadMarkdown(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  title: string,
  content: string,
): Promise<{
  documentId: string;
  chunkCount: number;
  entityCount: number;
  status: string;
}> {
  // Ollama/LM Studio sync uploads use 600s server timeout (v0.12.10+); mock stays 120s.
  const uploadTimeoutMs = ctx.llmProvider === "ollama" ? 660_000 : 180_000;
  const upload = await request.post(`${BACKEND_URL}/api/v1/documents`, {
    headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
    data: { title, content, async_processing: false },
    timeout: uploadTimeoutMs,
  });
  expect([200, 201]).toContain(upload.status());
  const body = (await upload.json()) as {
    document_id?: string;
    id?: string;
    status?: string;
    chunk_count?: number;
    entity_count?: number;
  };
  const documentId = body.document_id ?? body.id;
  expect(documentId).toBeTruthy();
  return {
    documentId: documentId!,
    chunkCount: body.chunk_count ?? 0,
    entityCount: body.entity_count ?? 0,
    status: body.status ?? "unknown",
  };
}

/** Re-upload identical markdown — exercises duplicate/re-ingestion edge. */
export async function reuploadSameMarkdown(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  title: string,
  content: string,
): Promise<{
  first: Awaited<ReturnType<typeof syncUploadMarkdown>>;
  second: Awaited<ReturnType<typeof syncUploadMarkdown>>;
}> {
  const first = await syncUploadMarkdown(request, ctx, title, content);
  const second = await syncUploadMarkdown(request, ctx, title, content);
  return { first, second };
}

/** Seed tenant/workspace and land on /documents (reuses spec013-bootstrap DRY). */
export async function bootstrapQcUiContext(
  page: Page,
  request: import("@playwright/test").APIRequestContext,
  label: string,
  options?: { provider?: WorkspaceProvider },
): Promise<QcWorkspaceContext> {
  const provider = options?.provider ?? "mock";
  const ctx =
    provider === "ollama"
      ? await createOllamaQcWorkspace(request, label)
      : await createMockQcWorkspace(request, label);
  await seedTenantStoreOnPage(page, ctx);
  await wireTenantApiProxy(page, ctx);
  await page.goto("/documents", { waitUntil: "domcontentloaded" });
  return ctx;
}
