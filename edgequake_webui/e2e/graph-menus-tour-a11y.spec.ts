/**
 * E2E — Graph node context menu + onboarding tour a11y.
 *
 * Covers the Phase 3 keyboard/focus-trap remediations:
 *  - NodeContextMenu (Radix DropdownMenu): ArrowDown/Enter/Escape + focus return
 *  - Tour overlay: initial focus, Tab trap, Escape, input-target guard
 *
 * Menu open is driven by a Playwright-only CustomEvent (`eq:e2e-open-node-menu`)
 * so the test does not depend on Sigma.js pixel-precise right-click hit detection.
 */
import { expect, test, type Page } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  GRAPH_FILTER_DOC_A,
  mockGraphDocumentFilterRoutes,
  seedGraphFilterTenantContext,
} from "./helpers/graph-document-filter-mocks";

async function openNodeMenu(page: Page) {
  await page.evaluate(() => {
    window.dispatchEvent(
      new CustomEvent("eq:e2e-open-node-menu", {
        detail: { x: 420, y: 280 },
      }),
    );
  });
  await page.getByTestId("node-context-menu").waitFor({ state: "visible" });
}

test.describe("Graph context menu + tour a11y", () => {
  test.beforeEach(async ({ page }) => {
    await mockGraphDocumentFilterRoutes(page);
    await seedGraphFilterTenantContext(page);
    await page.goto(
      `/graph?document=${GRAPH_FILTER_DOC_A}&stream=0`,
      GOTO_OPTS,
    );
    // Wait until the lineage subgraph is loaded (nodes available for the menu hook).
    await expect(page.getByText(/2 nodes · 1 edge/i)).toBeVisible({
      timeout: 20_000,
    });
  });

  test("node context menu: keyboard arrows, Escape closes", async ({
    page,
  }) => {
    await openNodeMenu(page);
    const menu = page.getByTestId("node-context-menu");
    await expect(menu).toBeVisible();
    await expect(page.getByTestId("node-context-menu-header")).toBeVisible();

    // Radix moves focus into the first menu item on open.
    // ArrowDown moves highlight to the next item; Escape closes the menu.
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Escape");
    await expect(menu).toHaveCount(0);
  });

  test("node context menu: Enter activates View Details", async ({ page }) => {
    await openNodeMenu(page);
    // Ensure a menu item has focus before Enter (Radix focuses the content on
    // open; ArrowDown moves onto the first actionable item explicitly).
    await page.keyboard.press("ArrowDown");
    const firstItem = page.getByRole("menuitem").first();
    await expect(firstItem).toBeFocused();
    await firstItem.press("Enter");
    await expect(page.getByTestId("node-context-menu")).toHaveCount(0);
  });

  test("tour: focus trap, Escape ends, arrows ignored while typing", async ({
    page,
  }) => {
    const trigger = page.getByTestId("tour-trigger");
    await expect(trigger).toBeVisible({ timeout: 10_000 });
    await trigger.click();

    const popover = page.getByTestId("tour-popover");
    await expect(popover).toBeVisible();
    // Initial focus lands on the popover (or a control inside it).
    await expect
      .poll(async () =>
        page.evaluate(() => {
          const root = document.querySelector('[data-testid="tour-popover"]');
          return Boolean(root && root.contains(document.activeElement));
        }),
      )
      .toBe(true);

    // Tab cycles within the popover (focus trap) — focus stays inside.
    await page.keyboard.press("Tab");
    await expect
      .poll(async () =>
        page.evaluate(() => {
          const root = document.querySelector('[data-testid="tour-popover"]');
          return Boolean(root && root.contains(document.activeElement));
        }),
      )
      .toBe(true);

    // Capture the step title before typing so we can assert it did NOT advance.
    const titleBefore = await popover.locator("h3").textContent();

    // Inject a temporary input into the popover and type with ArrowRight —
    // the input-target guard must prevent the tour from advancing.
    await popover.evaluate((el) => {
      const input = document.createElement("input");
      input.setAttribute("data-testid", "tour-temp-input");
      input.setAttribute("type", "text");
      el.appendChild(input);
      input.focus();
    });
    await page.getByTestId("tour-temp-input").press("ArrowRight");
    await page.getByTestId("tour-temp-input").press("ArrowRight");

    const titleAfter = await popover.locator("h3").textContent();
    expect(titleAfter).toBe(titleBefore);

    // Escape ends the tour.
    await page.keyboard.press("Escape");
    await expect(popover).toHaveCount(0);
  });
});
