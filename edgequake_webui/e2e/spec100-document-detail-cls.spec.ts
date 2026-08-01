/**
 * SPEC-100 — Document detail CLS / layout stability.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";

const DOC_ID = "doc-100-detail-cls";

function makeDetailDoc(overrides: Record<string, unknown> = {}) {
  return {
    id: DOC_ID,
    title: "CLS Detail Doc",
    file_name: "cls-detail.md",
    status: "completed",
    source_type: "text",
    mime_type: "text/markdown",
    content: "# Hello\n\nStable body for CLS gate.\n",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    track_id: null,
    ...overrides,
  };
}

test.describe("SPEC-100 document detail CLS", () => {
  test("progress slot reserved; main shell Y stable after delayed load", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });

    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);

    await page.addInitScript(() => {
      (window as unknown as { __eqClsScore?: number }).__eqClsScore = 0;
      const obs = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          const ls = entry as PerformanceEntry & {
            value?: number;
            hadRecentInput?: boolean;
          };
          if (!ls.hadRecentInput) {
            (window as unknown as { __eqClsScore: number }).__eqClsScore +=
              ls.value ?? 0;
          }
        }
      });
      obs.observe({ type: "layout-shift", buffered: true });
    });

    await page.route(`**/api/v1/documents/${DOC_ID}**`, async (route) => {
      if (route.request().method() !== "GET") {
        await route.fallback();
        return;
      }
      await new Promise((r) => setTimeout(r, 500));
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(makeDetailDoc()),
      });
    });

    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);

    const progressSlot = page.getByTestId("detail-page-reprocess-progress-slot");
    // Cold skeleton or live page both mount the progress slot
    await expect(
      page
        .getByTestId("spec100-document-detail-skeleton")
        .or(page.getByRole("heading", { name: /CLS Detail Doc/i })),
    ).toBeVisible({ timeout: 20_000 });

    await expect(progressSlot).toBeVisible({ timeout: 20_000 });
    const yDuring = await progressSlot.boundingBox();
    expect(yDuring).toBeTruthy();

    await expect(page.getByRole("heading", { name: /CLS Detail Doc/i })).toBeVisible({
      timeout: 20_000,
    });

    const yAfter = await progressSlot.boundingBox();
    expect(yAfter).toBeTruthy();
    // Allow modest header chrome settle; slot should remain near-stable
    expect(Math.abs((yAfter?.y ?? 0) - (yDuring?.y ?? 0))).toBeLessThanOrEqual(48);

    const cls = await page.evaluate(
      () => (window as unknown as { __eqClsScore?: number }).__eqClsScore ?? 0,
    );
    expect(cls).toBeLessThan(0.35);
  });
});
