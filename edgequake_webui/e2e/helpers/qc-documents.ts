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

/** Smallest SPEC-049 corpus paper (real ICLR PDF, 12 pages). */
export const SPEC049_HIPO_PDF = path.resolve(
  __dirname,
  "../../../specs/049-improve-figure-extraction/data/hipo_2607.02303v1.pdf",
);

/** LightRAG arXiv PDF used across EdgeQuake demos. */
export const SPEC049_LIGHTRAG_PDF = path.resolve(
  __dirname,
  "../../../specs/049-improve-figure-extraction/data/lighrad_2410.05779v3.pdf",
);

export const SPEC128_PDF_DATA_DIR = path.resolve(
  __dirname,
  "../../../specs/128-improve-pdf-parsing/pdf_data",
);

/** All `*.pdf` under spec 128 `pdf_data/`, smallest first (primary live paper). */
export function listSpec128PdfData(): string[] {
  if (!fs.existsSync(SPEC128_PDF_DATA_DIR)) {
    return [];
  }
  return fs
    .readdirSync(SPEC128_PDF_DATA_DIR)
    .filter((name) => name.toLowerCase().endsWith(".pdf"))
    .map((name) => path.join(SPEC128_PDF_DATA_DIR, name))
    .sort((a, b) => fs.statSync(a).size - fs.statSync(b).size);
}

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
  filePath?: string;
  fileName?: string;
  /** Form field `vision_provider` (e.g. mock) so ingest does not inherit a geo-blocked cloud VLM. */
  visionProvider?: string;
  /** Form field `vision_model` (upload wins SPEC-123). */
  visionModel?: string;
  /** Wall clock for PDF job + document terminal status. */
  timeoutMs?: number;
};

export type PageLayoutBody = {
  document_id?: string;
  page_number?: number;
  layout_status?: string;
  error_message?: string;
  regions?: Array<{
    class?: string;
    asset_path?: string | null;
    extra?: Record<string, unknown>;
    bbox_norm?: { x: number; y: number; w: number; h: number };
  }>;
};

/** Poll GET .../pages/{n}/layout until at least one region exists (convert-time persist). */
export async function pollDocumentPageLayout(
  request: import("@playwright/test").APIRequestContext,
  docId: string,
  tenantId: string,
  workspaceId: string,
  pageNumber = 1,
  maxMs = 300_000,
): Promise<PageLayoutBody> {
  const deadline = Date.now() + maxMs;
  let lastStatus = 0;
  while (Date.now() < deadline) {
    const res = await request.get(
      `${BACKEND_URL}/api/v1/documents/${docId}/pages/${pageNumber}/layout`,
      { headers: tenantHeaders(tenantId, workspaceId) },
    );
    lastStatus = res.status();
    if (res.ok()) {
      const body = (await res.json()) as PageLayoutBody;
      if ((body.regions?.length ?? 0) >= 1) {
        return body;
      }
    }
    const docRes = await request.get(`${BACKEND_URL}/api/v1/documents/${docId}`, {
      headers: tenantHeaders(tenantId, workspaceId),
    });
    if (docRes.ok()) {
      const doc = (await docRes.json()) as { status?: string; error_message?: string };
      const st = (doc.status ?? "").toLowerCase();
      if (st.includes("fail")) {
        throw new Error(
          `document ${docId} failed before layout persist: ${doc.error_message ?? st}`,
        );
      }
    }
    await new Promise((r) => setTimeout(r, 3000));
  }
  throw new Error(
    `layout for ${docId} page ${pageNumber} not persisted within ${maxMs}ms (last HTTP ${lastStatus})`,
  );
}

/** Admit a PDF (queued). Does not wait for convert or KG extract. */
export async function admitPdfViaApi(
  request: import("@playwright/test").APIRequestContext,
  ctx: QcWorkspaceContext,
  options: PdfUploadOptions = {},
): Promise<{ pdfId: string; documentId: string; enableVision: boolean }> {
  const filePath = options.filePath ?? SIMPLE_PDF_FIXTURE;
  if (!fs.existsSync(filePath)) {
    throw new Error(`PDF fixture missing: ${filePath}`);
  }
  const pdfBytes = fs.readFileSync(filePath);
  const fileName = options.fileName ?? path.basename(filePath);
  const docTitle = options.title ?? `spec020-pdf-${Date.now()}`;
  const trackId = `spec020-pdf-${Date.now()}`;
  const enableVision = options.enableVision ?? false;
  const fields: Record<string, string> = {
    title: docTitle,
    enable_vision: enableVision ? "true" : "false",
    pdf_parser_backend: options.parserBackend ?? "text",
    force_reindex: "true",
    track_id: trackId,
  };
  if (options.visionProvider) {
    fields.vision_provider = options.visionProvider;
  }
  if (options.visionModel) {
    fields.vision_model = options.visionModel;
  }
  const { boundary, body } = pdfMultipartBody(fileName, pdfBytes, fields);

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
  const uploadBody = (await upload.json()) as {
    pdf_id?: string;
    document_id?: string | null;
  };
  expect(uploadBody.pdf_id).toBeTruthy();
  expect(uploadBody.document_id).toBeTruthy();
  return {
    pdfId: uploadBody.pdf_id!,
    documentId: uploadBody.document_id!,
    enableVision,
  };
}

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
  const timeoutMs = options.timeoutMs ?? 300_000;
  const admitted = await admitPdfViaApi(request, ctx, options);

  const completed = await pollPdfCompleted(
    request,
    admitted.pdfId,
    ctx.tenantId,
    ctx.workspaceId,
    timeoutMs,
  );
  const documentId = completed.document_id ?? admitted.documentId;
  expect(documentId).toBeTruthy();

  const meta = await pollDocumentStatus(
    request,
    documentId!,
    ctx.tenantId,
    ctx.workspaceId,
    timeoutMs,
  );
  return {
    pdfId: admitted.pdfId,
    documentId: documentId!,
    chunkCount: meta.chunk_count ?? 0,
    status: meta.status ?? "unknown",
    enableVision: admitted.enableVision,
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
