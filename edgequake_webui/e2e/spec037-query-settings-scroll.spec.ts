/**
 * SPEC-037 — Query Settings scroll E2E
 * @implements REQ-037-01
 */

import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/navigation";
import { mockBackendForUiOnly } from "./helpers/mock-backend";
import { expectScrollable, scrollToBottom } from "./helpers/scroll";
import { spec037Screenshot } from "./helpers/screenshot-paths";

test.describe("SPEC-037 Query Settings Scroll", () => {
  test.beforeEach(async ({ page }) => {
    await mockBackendForUiOnly(page);
    await page.goto("/query", GOTO_OPTS);
    await page.waitForLoadState("domcontentloaded");
    await page.locator("main").first().waitFor({ state: "visible", timeout: 20_000 });
  });

  test("settings sheet scrolls to system prompt", async ({ page }) => {
    await page.getByTestId("query-settings-trigger").click();
    await expect(page.getByTestId("query-settings-sheet")).toBeVisible();

    await page.screenshot({
      path: spec037Screenshot("01-settings-open-top.png"),
      fullPage: false,
    });

    const viewport = page.locator(
      '[data-testid="query-settings-sheet"] [data-slot="scroll-area-viewport"]',
    );
    await expectScrollable(viewport);
    await scrollToBottom(viewport);

    const systemPrompt = page.getByTestId("query-settings-system-prompt");
    await expect(systemPrompt).toBeVisible();

    await page.screenshot({
      path: spec037Screenshot("02-settings-scrolled-system-prompt.png"),
      fullPage: false,
    });
  });
});
