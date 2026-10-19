/**
 * SPEC-169 round-00: the BEFORE state — user administration buried in Settings.
 * Captured once, before `UserManagementCard` was retired. Skipped when the card is gone.
 */
import { expect, test } from "@playwright/test";
import { prepareSpec168Page } from "../spec168/helpers/rich-mock";
import { applyTheme, setViewport, shot } from "./helpers/shots";

const USERS = [
  ["ada", "ada@acme.io", "admin"],
  ["bob", "bob@acme.io", "user"],
  ["carol", "carol@acme.io", "readonly"],
].map(([username, email, role], i) => ({
  user_id: `00000000-0000-0000-0000-00000000000${i + 1}`,
  username,
  email,
  role,
  is_active: true,
  created_at: "2026-09-01T10:00:00Z",
  last_login_at: "2026-10-18T09:00:00Z",
}));

test.describe("SPEC-169 baseline @spec169", () => {
  test("settings user card (before)", async ({ page }) => {
    test.setTimeout(120_000);
    await prepareSpec168Page(page);
    await page.route("**/api/v1/users**", (r) =>
      r.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ users: USERS, total: 3, page: 1, page_size: 20, total_pages: 1 }),
      }),
    );
    await setViewport(page, 1440);
    await page.goto("/settings", { waitUntil: "domcontentloaded" });
    await applyTheme(page, "light");
    await page.getByTestId("settings-nav").getByRole("link", { name: "Access" }).click();
    const card = page.getByText("User Management", { exact: true }).first();
    const present = await card.waitFor({ state: "visible", timeout: 20_000 }).then(() => true, () => false);
    test.skip(!present, "legacy card already retired");
    await card.scrollIntoViewIfNeeded();
    await page.waitForTimeout(800);
    await shot(page, "settings-user-card-before");
    await expect(page.getByText("ada@acme.io").first()).toBeVisible();
  });
});
