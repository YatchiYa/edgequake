/**
 * SPEC-100 — Graph CLS: header count chip always reserved.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";

test.describe("SPEC-100 graph CLS", () => {
  test("count slot stays in header while graph loads", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);

    await page.goto("/graph", GOTO_OPTS);

    const count = page.getByTestId("spec100-graph-count-slot");
    await expect(count).toBeVisible({ timeout: 30_000 });
    const yDuring = await count.boundingBox();
    expect(yDuring).toBeTruthy();

    // Allow graph query to settle; chip Y should stay stable
    await page.waitForTimeout(800);
    const yAfter = await count.boundingBox();
    expect(Math.abs((yAfter?.y ?? 0) - (yDuring?.y ?? 0))).toBeLessThanOrEqual(8);
  });
});
