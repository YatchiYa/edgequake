/**
 * SPEC-123: Documents page honesty — Workspace Default (Vision) never
 * implies Auto, and upload selector exposes Auto explicitly.
 *
 * UI-only gate: mock + seed (same as SPEC-038) so `/documents` renders
 * without a live backend. `make test-e2e-ui` uses a 30s test timeout.
 *
 * Run: `PLAYWRIGHT_SKIP_STACK_CHECK=1 pnpm exec playwright test e2e/spec123-parser-priority.spec.ts --project=chromium`
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";

test.describe("SPEC-123 parser priority UI", () => {
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await page.goto("/documents", GOTO_OPTS);
    await page.getByRole("heading", { name: "Documents" }).waitFor({
      state: "visible",
      timeout: 20_000,
    });
  });

  test("upload parser select includes Vision, EdgeParse, and Auto", async ({
    page,
  }) => {
    const select = page.getByTestId("spec038-upload-parser-select");
    await expect(select).toBeVisible({ timeout: 15_000 });
    await select.click();
    await expect(page.getByRole("option", { name: /Vision/i })).toBeVisible();
    await expect(
      page.getByRole("option", { name: /EdgeParse/i }),
    ).toBeVisible();
    await expect(page.getByRole("option", { name: /Auto/i })).toBeVisible();
    // Inherit option must not silently say Auto when resolving Vision.
    const inherit = page.getByRole("option", { name: /Workspace Default/i });
    await expect(inherit).toBeVisible();
    const inheritText = (await inherit.textContent()) ?? "";
    if (inheritText.includes("Vision")) {
      expect(inheritText).not.toMatch(/Auto/i);
    }
  });
});
