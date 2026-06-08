/**
 * SPEC-019 — Option 1 live stack: UI query after API upload.
 * Artifacts: specs/019-0-12-7-control/e2e/screenshots/
 */
import fs from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const ARTIFACT_DIR = path.resolve(
  __dirname,
  "../../specs/019-0-12-7-control/e2e/screenshots",
);

const API_URL =
  process.env.EQ_BACKEND_URL ??
  `http://127.0.0.1:${process.env.EDGEQUAKE_PORT ?? "18080"}`;
const TENANT_ID = "00000000-0000-0000-0000-000000000002";
const WORKSPACE_ID = "00000000-0000-0000-0000-000000000003";

const UPLOAD_DOC = `
SPEC-019 UI query proof. Sarah Chen is a senior engineer at EDGEQUAKE.
Michael Torres leads LLM integration. GraphRAG platform in Rust v0.12.7.
`.trim();

test.describe("SPEC-019 Option 1 upload + query UI", () => {
  test("API upload then UI query returns Sarah Chen answer", async ({
    page,
    request,
  }) => {
    test.setTimeout(300_000);
    fs.mkdirSync(ARTIFACT_DIR, { recursive: true });

    const health = await request.get(`${API_URL}/health`);
    expect(health.ok()).toBeTruthy();
    const healthBody = await health.json();
    expect(healthBody.version).toBe("0.12.7");

    const title = `spec019-ui-${Date.now()}.md`;
    const upload = await request.post(`${API_URL}/api/v1/documents`, {
      headers: {
        "Content-Type": "application/json",
        "X-Tenant-ID": TENANT_ID,
        "X-Workspace-ID": WORKSPACE_ID,
      },
      data: {
        title,
        content: UPLOAD_DOC,
        async_processing: false,
      },
      timeout: 240_000,
    });
    expect([200, 201]).toContain(upload.status());
    const uploaded = (await upload.json()) as {
      entity_count?: number;
      chunk_count?: number;
    };
    expect((uploaded.chunk_count ?? 0) > 0).toBeTruthy();
    expect((uploaded.entity_count ?? 0) > 0).toBeTruthy();

    await page.goto("/documents");
    await expect(page.getByText(title).first()).toBeVisible({ timeout: 60_000 });
    await page.screenshot({
      path: path.join(ARTIFACT_DIR, "05-documents-after-upload.png"),
      fullPage: false,
    });

    await page.goto("/query");
    const input = page.locator("textarea.query-input").first();
    await expect(input).toBeVisible({ timeout: 20_000 });
    await input.fill("Who is Sarah Chen at EDGEQUAKE?");
    await input.press("Enter");

    await expect(page.getByText(/Processing your query/i)).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByText(/Processing your query/i)).toBeHidden({
      timeout: 180_000,
    });

    const assistantAnswer = page
      .locator(".message-assistant, [data-testid='assistant-message'], .prose")
      .filter({ hasText: /senior engineer|Sarah Chen/i })
      .first();
    await expect(assistantAnswer).toBeVisible({ timeout: 30_000 });

    await page.screenshot({
      path: path.join(ARTIFACT_DIR, "07-query-answer-ui.png"),
      fullPage: false,
    });
  });
});
