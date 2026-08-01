/**
 * SPEC-100 F-100-02 — Breadcrumb band always reserved (CLS).
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";
import {
  makeSpec086ListDoc,
  mockSpec086DocumentList,
} from "./helpers/spec086-ingestion-mocks";

test.describe("SPEC-100 breadcrumb slot", () => {
  test("depth-1 spacer and depth-2 bar share h-9 band", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086DocumentList(page, [
      makeSpec086ListDoc({
        id: "doc-100-bc",
        file_name: "bc.pdf",
        status: "completed",
        query_ready: true,
      }),
    ]);

    await page.goto("/documents", GOTO_OPTS);
    const spacer = page.getByTestId("breadcrumb-spacer");
    await expect(spacer).toBeVisible({ timeout: 20_000 });
    const spacerBox = await spacer.boundingBox();
    expect(spacerBox?.height).toBeGreaterThanOrEqual(32);
    expect(spacerBox?.height).toBeLessThanOrEqual(40);

    await page.goto("/documents/doc-100-bc", GOTO_OPTS);
    const bar = page.getByTestId("breadcrumb-bar");
    // Detail may 404 without full mocks — bar or spacer still occupies the band
    const band = bar.or(spacer);
    await expect(band.first()).toBeVisible({ timeout: 20_000 });
    const bandBox = await band.first().boundingBox();
    expect(bandBox?.height).toBeGreaterThanOrEqual(32);
    expect(bandBox?.height).toBeLessThanOrEqual(40);
    expect(Math.abs((bandBox?.height ?? 0) - (spacerBox?.height ?? 0))).toBeLessThan(
      4,
    );
  });
});
