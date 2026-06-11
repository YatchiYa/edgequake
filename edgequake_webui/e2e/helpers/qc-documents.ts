/**
 * SPEC-020 document + PDF API helpers (DRY extraction from spec017).
 */
import fs from "node:fs";
import path from "node:path";
import { expect } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";
import { tenantHeaders } from "./spec013-api";
import type { QcWorkspaceContext } from "./qc-workspace";

export const SIMPLE_PDF_FIXTURE = path.resolve(
  __dirname,
  "../../../legacy/edgequake-pdf/test-data/001_simple_text.pdf",
);

export async function pollDocumentStatus(
  request: import("@playwright/test").APIRequestContext,
  docId: string,
  tenantId: string,
  workspaceId: string,
  maxMs = 300_000,
): Promise<{
  status?: string;
  entity_count?: number;
  chunk_count?: number;
}> {
  const deadline = Date.now() + maxMs;
  while (Date.now() < deadline) {
    const res = await request.get(`${BACKEND_URL}/api/v1/documents/${docId}`, {
      headers: tenantHeaders(tenantId, workspaceId),
    });
    if (res.ok()) {
      const body = (await res.json()) as {
        status?: string;
        entity_count?: number;
        chunk_count?: number;
      };
      const status = (body.status ?? "").toLowerCase();
      if (
        ["processed", "completed", "failed", "partial", "partial_failure"].some(
          (s) => status.includes(s),
        )
      ) {
        return body;
      }
      if (status === "pending" || status === "processing") {
        await new Promise((r) => setTimeout(r, 3000));
        continue;
      }
    }
    await new Promise((r) => setTimeout(r, 2000));
  }
  throw new Error(`document ${docId} did not reach terminal status within ${maxMs}ms`);
}

function pdfMultipartBody(
  filename: string,
  pdfBytes: Buffer,
  fields: Record<string, string>,
): { boundary: string; body: Buffer } {
  const boundary = `spec020-pdf-${Date.now()}`;
  const chunks: Buffer[] = [];
  for (const [k, v] of Object.entries(fields)) {
    chunks.push(Buffer.from(`--${boundary}\r\n`));
    chunks.push(
      Buffer.from(`Content-Disposition: form-data; name="${k}"\r\n\r\n${v}\r\n`),
    );
  }
  chunks.push(Buffer.from(`--${boundary}\r\n`));
  chunks.push(
    Buffer.from(
      `Content-Disposition: form-data; name="file"; filename="${filename}"\r\nContent-Type: application/pdf\r\n\r\n`,
    ),
  );
  chunks.push(pdfBytes);
  chunks.push(Buffer.from(`\r\n--${boundary}--\r\n`));
  return { boundary, body: Buffer.concat(chunks) };
}

async function pollPdfCompleted(
  request: import("@playwright/test").APIRequestContext,
  pdfId: string,
  tenantId: string,
  workspaceId: string,
  maxMs = 600_000,
): Promise<{ status?: string; document_id?: string | null }> {
  const deadline = Date.now() + maxMs;
  while (Date.now() < deadline) {
    const res = await request.get(`${BACKEND_URL}/api/v1/documents/pdf/${pdfId}`, {
      headers: tenantHeaders(tenantId, workspaceId),
    });
    if (res.ok()) {
      const body = (await res.json()) as {
        status?: string;
        document_id?: string | null;
      };
      const status = (body.status ?? "").toLowerCase();
      if (status === "failed") {
        throw new Error(`PDF pipeline failed: ${JSON.stringify(body)}`);
      }
      if (status === "completed" && (body.document_id?.length ?? 0) > 10) {
        return body;
      }
    }
    await new Promise((r) => setTimeout(r, 3000));
  }
  throw new Error(`PDF ${pdfId} did not complete within ${maxMs}ms`);
}

export type PdfUploadOptions = {
  title?: string;
  enableVision?: boolean;
  parserBackend?: "text" | "vision";
};

/** Upload small PDF via API (text parser default; vision flag optional). */
export async function uploadPdfViaApi(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  options: PdfUploadOptions = {},
): Promise<{
  pdfId: string;
  documentId: string;
  chunkCount: number;
  status: string;
  enableVision: boolean;
}> {
  if (!fs.existsSync(SIMPLE_PDF_FIXTURE)) {
    throw new Error(`PDF fixture missing: ${SIMPLE_PDF_FIXTURE}`);
  }
  const pdfBytes = fs.readFileSync(SIMPLE_PDF_FIXTURE);
  const docTitle = options.title ?? `spec020-pdf-${Date.now()}`;
  const trackId = `spec020-pdf-${Date.now()}`;
  const enableVision = options.enableVision ?? false;
  const { boundary, body } = pdfMultipartBody("001_simple_text.pdf", pdfBytes, {
    title: docTitle,
    enable_vision: enableVision ? "true" : "false",
    pdf_parser_backend: options.parserBackend ?? "text",
    force_reindex: "true",
    track_id: trackId,
  });

  const upload = await request.fetch(`${BACKEND_URL}/api/v1/documents/pdf`, {
    method: "POST",
    headers: {
      ...tenantHeaders(ctx.tenantId, ctx.workspaceId),
      "Content-Type": `multipart/form-data; boundary=${boundary}`,
    },
    data: body,
    timeout: 120_000,
  });
  expect([200, 201, 202]).toContain(upload.status());
  const uploadBody = (await upload.json()) as { pdf_id?: string };
  expect(uploadBody.pdf_id).toBeTruthy();

  const completed = await pollPdfCompleted(
    request,
    uploadBody.pdf_id!,
    ctx.tenantId,
    ctx.workspaceId,
    300_000,
  );
  expect(completed.document_id).toBeTruthy();

  const meta = await pollDocumentStatus(
    request,
    completed.document_id!,
    ctx.tenantId,
    ctx.workspaceId,
    180_000,
  );
  return {
    pdfId: uploadBody.pdf_id!,
    documentId: completed.document_id!,
    chunkCount: meta.chunk_count ?? 0,
    status: meta.status ?? "unknown",
    enableVision,
  };
}

/** Upload small text-parser PDF via API (no vision). */
export async function uploadSimplePdfViaApi(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  title?: string,
) {
  return uploadPdfViaApi(request, ctx, { title });
}

/** Fetch document by ID — expect 404 for unknown UUID. */
export async function assertDocumentNotFound(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  fakeId = "00000000-0000-0000-0000-000000000099",
): Promise<number> {
  const res = await request.get(`${BACKEND_URL}/api/v1/documents/${fakeId}`, {
    headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
  });
  expect([404, 403]).toContain(res.status());
  return res.status();
}

/** List document titles for tenant/workspace. */
export async function listDocumentTitles(
  request: import("@playwright/test").APIRequestContext,
  tenantId: string,
  workspaceId: string,
): Promise<string[]> {
  const res = await request.get(`${BACKEND_URL}/api/v1/documents`, {
    headers: tenantHeaders(tenantId, workspaceId),
  });
  if (!res.ok()) return [];
  const body = (await res.json()) as {
    documents?: Array<{ title?: string }>;
    items?: Array<{ title?: string }>;
  };
  const docs = body.documents ?? body.items ?? (Array.isArray(body) ? body : []);
  return (docs as Array<{ title?: string }>).map((d) => d.title ?? "").filter(Boolean);
}

/** Delete document by ID — expect 200/204. */
export async function deleteDocumentViaApi(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  documentId: string,
): Promise<number> {
  const res = await request.delete(
    `${BACKEND_URL}/api/v1/documents/${documentId}`,
    { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
  );
  expect([200, 204, 202]).toContain(res.status());
  return res.status();
}

/** After delete, GET must 404 and title absent from list. */
export async function assertDocumentDeleted(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  documentId: string,
  title: string,
): Promise<{ getStatus: number; listed: boolean }> {
  const getRes = await request.get(
    `${BACKEND_URL}/api/v1/documents/${documentId}`,
    { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
  );
  expect([404, 410]).toContain(getRes.status());
  const titles = await listDocumentTitles(
    request,
    ctx.tenantId,
    ctx.workspaceId,
  );
  return { getStatus: getRes.status(), listed: titles.includes(title) };
}
