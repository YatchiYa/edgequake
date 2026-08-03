/**
 * Document detail must render text body content (not a blank pane).
 *
 * Covers the finalize truncation regression: shell body must be full text,
 * and empty body must show an explicit empty state.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import { bootstrapDeterministicUiContext } from "./helpers/bootstrap-ui";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { gotoApp } from "./helpers/navigation";
import { API_V1_URL } from "./helpers/backend-url";
import {
  obtainAccessToken,
  tenantHeaders,
} from "./helpers/spec013-api";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";
import { uploadFilesOnDocumentsPage } from "./helpers/upload";

const SENTINEL = "EQ_TEXT_CONTENT_SENTINEL_TAIL_9f3c2a";
const MOCK_DOC_ID = "doc-text-content-sentinel";

function makeLongPlainText(): string {
  const prefix = "EdgeQuake text detail body padding. ".repeat(20);
  return `${prefix}\n\n${SENTINEL}\n`;
}

function makeDetailDoc(overrides: Record<string, unknown> = {}) {
  return {
    id: MOCK_DOC_ID,
    title: "text-content-sentinel.txt",
    file_name: "text-content-sentinel.txt",
    status: "completed",
    source_type: "text",
    mime_type: "text/plain",
    chunk_count: 2,
    entity_count: 1,
    relationship_count: 0,
    content: makeLongPlainText(),
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    track_id: null,
    ...overrides,
  };
}

test.describe("Document text detail content (mocked)", () => {
  test("renders sentinel body when API returns full text content", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);

    const body = makeLongPlainText();
    expect(body.length).toBeGreaterThan(500);

    await page.route(`**/api/v1/documents/${MOCK_DOC_ID}**`, async (route) => {
      if (route.request().method() !== "GET") {
        await route.fallback();
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(makeDetailDoc({ content: body })),
      });
    });

    await page.goto(`/documents/${MOCK_DOC_ID}`, GOTO_OPTS);
    await expect(
      page.getByRole("heading", { name: /text-content-sentinel/i }).first(),
    ).toBeVisible({ timeout: 20_000 });
    // Desktop + mobile panes can both mount; assert at least one visible body.
    await expect(page.getByText(SENTINEL).first()).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByTestId("document-content-empty")).toHaveCount(0);
  });

  test("shows empty state when API omits content", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);

    await page.route(`**/api/v1/documents/${MOCK_DOC_ID}**`, async (route) => {
      if (route.request().method() !== "GET") {
        await route.fallback();
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(
          makeDetailDoc({ content: undefined, content_summary: undefined }),
        ),
      });
    });

    await page.goto(`/documents/${MOCK_DOC_ID}`, GOTO_OPTS);
    await expect(
      page.getByRole("heading", { name: /text-content-sentinel/i }).first(),
    ).toBeVisible({ timeout: 20_000 });
    await expect(page.getByTestId("document-content-empty").first()).toBeVisible({
      timeout: 15_000,
    });
  });
});

test.describe("Document text detail content (live)", () => {
  test.beforeEach(() => {
    skipUnlessLiveStack();
  });

  test("upload long txt and show sentinel on detail page", async ({
    page,
    request,
  }) => {
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      "text-content",
    );
    const body = makeLongPlainText();
    expect(body.length).toBeGreaterThan(500);

    await uploadFilesOnDocumentsPage(page, {
      name: `eq-text-content-${Date.now()}.txt`,
      mimeType: "text/plain",
      buffer: Buffer.from(body, "utf-8"),
    });

    // Wait for a row / view link for this upload, then open detail.
    const viewLink = page.getByRole("link", { name: /view/i }).first();
    await expect(viewLink).toBeVisible({ timeout: 120_000 });
    await viewLink.click();
    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/, { timeout: 30_000 });

    // Prefer API assertion (heal/full body) then UI sentinel.
    const detailUrl = page.url();
    const docId = detailUrl.split("/documents/")[1]?.split(/[?#]/)[0];
    expect(docId).toBeTruthy();

    const token = await obtainAccessToken(request);
    const headers = tenantHeaders(
      ctx.tenantId,
      ctx.workspaceId,
      token ? { Authorization: `Bearer ${token}` } : {},
    );

    await expect
      .poll(
        async () => {
          const res = await request.get(`${API_V1_URL}/documents/${docId}`, {
            headers,
          });
          if (!res.ok()) return 0;
          const json = (await res.json()) as { content?: string };
          return json.content?.length ?? 0;
        },
        { timeout: 180_000 },
      )
      .toBeGreaterThan(500);

    await gotoApp(page, `/documents/${docId}`);
    await expect(page.getByText(SENTINEL).first()).toBeVisible({
      timeout: 60_000,
    });
  });
});
