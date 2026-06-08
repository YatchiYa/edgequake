/**
 * SPEC-020 UI upload helpers — file input + progress observation (SRP).
 */
import path from "node:path";
import { expect, type Page } from "@playwright/test";
import { GOTO_OPTS, waitForAppReady } from "./app-ready";
import { uploadFilesOnDocumentsPage } from "./upload";
import type { QcWorkspaceContext } from "./qc-workspace";

const TERMINAL_STATUS = /Completed|Processed|Partial|Failed|Upload Complete/i;

/** Navigate to documents (API proxy must be wired via bootstrapQcUiContext). */
export async function bootstrapForUiUpload(page: Page): Promise<void> {
  await page.goto("/documents", GOTO_OPTS);
  await waitForAppReady(page);
}

/** Upload markdown via documents page file input (reliable UI path in dev). */
export async function uploadMarkdownViaUi(
  page: Page,
  ctx: QcWorkspaceContext,
  content: string,
  filename: string,
  options?: { timeoutMs?: number },
): Promise<{ observedStatus: string; title: string }> {
  await bootstrapForUiUpload(page);
  const timeoutMs = options?.timeoutMs ?? 120_000;
  const input = page.locator('input[type="file"]').first();
  await input.setInputFiles({
    name: filename,
    mimeType: "text/markdown",
    buffer: Buffer.from(content),
  });

  await expect(
    page.getByText(/Processing Files|Upload Complete|Uploading/i).first(),
  ).toBeVisible({ timeout: 15_000 });

  await expect(page.getByText(filename).first()).toBeVisible({ timeout: 30_000 });

  const row = page.locator("table tbody tr").filter({ hasText: filename }).first();
  await expect(row).toBeVisible({ timeout: timeoutMs });

  const badge = row.locator('[data-testid="status-badge"]').first();
  await expect(badge).toHaveText(TERMINAL_STATUS, { timeout: timeoutMs });
  return {
    observedStatus: (await badge.textContent()) ?? "unknown",
    title: filename,
  };
}

/** Upload PDF via documents page file input; wait for progress panel completion. */
export async function uploadPdfViaUi(
  page: Page,
  ctx: QcWorkspaceContext,
  pdfPath: string,
  options?: { filenamePattern?: RegExp; timeoutMs?: number },
): Promise<{ observedStatus: string }> {
  await bootstrapForUiUpload(page);
  const filenamePattern =
    options?.filenamePattern ?? /001_simple_text|\.pdf/i;
  const timeoutMs = options?.timeoutMs ?? 300_000;

  const basename = path.basename(pdfPath);
  await uploadFilesOnDocumentsPage(page, pdfPath);

  await expect(
    page.getByText(/Processing Files|Upload Complete|Uploading/i).first(),
  ).toBeVisible({ timeout: 15_000 });

  await expect(page.getByText(basename).first()).toBeVisible({ timeout: 30_000 });

  await expect(
    page.getByText(/Upload Complete|files complete/i).first(),
  ).toBeVisible({ timeout: timeoutMs });

  const row = page
    .locator("table tbody tr")
    .filter({ hasText: filenamePattern })
    .first();
  const inTable = await row.isVisible({ timeout: 90_000 }).catch(() => false);
  if (inTable) {
    const badge = row.locator('[data-testid="status-badge"]').first();
    await expect(badge).toHaveText(TERMINAL_STATUS, { timeout: 60_000 });
    return { observedStatus: (await badge.textContent()) ?? "table-row" };
  }
  return { observedStatus: "upload-complete-progress-panel" };
}

/** Open document detail by known API id (reliable vs View link). */
export async function openDocumentDetailById(
  page: Page,
  documentId: string,
): Promise<void> {
  await page.goto(`/documents/${documentId}`, GOTO_OPTS);
  await expect(page.locator("main").first()).toBeVisible({ timeout: 20_000 });
}
