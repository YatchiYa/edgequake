/**
 * SPEC-100 — API Explorer CLS: full-bleed loading slot until Scalar mounts.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";

test.describe("SPEC-100 api-explorer CLS", () => {
  test("explorer shell fills main area", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);

    await page.goto("/api-explorer", GOTO_OPTS);

    const shell = page.getByTestId("api-explorer-page");
    await expect(shell).toBeVisible({ timeout: 20_000 });
    await expect(page.getByTestId("api-explorer-scalar")).toBeVisible({
      timeout: 20_000,
    });
    const box = await shell.boundingBox();
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(400);
  });
});
