/**
 * SPEC-123: Documents page honesty — Workspace Default (Vision) never
 * implies Auto, and upload selector exposes Auto explicitly.
 *
 * Run (with WebUI up): `pnpm exec playwright test e2e/spec123-parser-priority.spec.ts`
 */
import { expect, test } from "@playwright/test";

test.describe("SPEC-123 parser priority UI", () => {
  test("upload parser select includes Vision, EdgeParse, and Auto", async ({
    page,
  }) => {
    await page.goto("/documents");
    const select = page.getByTestId("spec038-upload-parser-select");
    await expect(select).toBeVisible({ timeout: 60_000 });
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
