/**
 * Unfakable proof: HTML sub/sup inside markdown codespans render as real DOM,
 * not literal `c<sub>i</sub>` text (PDF extraction artifact).
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";

test.describe("HTML-in-codespan math display", () => {
  test("fixture renders real <sub>/<sup> from backtick HTML", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.goto("/e2e-fixtures/markdown-latex", {
      ...GOTO_OPTS,
      waitUntil: "load",
    });

    await page.waitForSelector('[data-testid="markdown-latex-fixture"]', {
      timeout: 60_000,
    });

    const htmlCodespans = page.getByTestId("md-html-codespan");
    await expect(htmlCodespans.first()).toBeVisible({ timeout: 15_000 });
    expect(await htmlCodespans.count()).toBeGreaterThanOrEqual(1);

    // Real DOM subscript — not escaped text
    const subI = page.locator('[data-testid="md-html-codespan"] sub').filter({
      hasText: "i",
    });
    await expect(subI.first()).toBeVisible();

    const bodyText = (await page.textContent("body")) ?? "";
    expect(bodyText).not.toContain("c<sub>i</sub>");
    expect(bodyText).not.toContain("r<sub>i</sub>");
  });
});
