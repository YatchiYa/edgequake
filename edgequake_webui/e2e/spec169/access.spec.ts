/**
 * SPEC-169 — set password, magic sign-in links, reset links, change password and the public
 * invitation page (hermetic). Each test asserts behaviour AND writes a screenshot to
 * specs/169-perfect-user-management/e2e/screenshots/<SPEC169_ROUND>/.
 */
import { expect, test, type Page } from "@playwright/test";
import { CURRENT_PASSWORD, STRONG_PASSWORD } from "./helpers/access-mock";
import { prepareMembersPage } from "./helpers/members-mock";
import { applyTheme, setViewport, shot } from "./helpers/shots";

const DIEGO = "00000000-0000-4000-8000-000000000004";
const CAROL = "00000000-0000-4000-8000-000000000003";

const members = async (page: Page) => {
  const state = await prepareMembersPage(page);
  await setViewport(page, 1440);
  await page.goto("/members", { waitUntil: "domcontentloaded" });
  await expect(page.getByTestId("members-table")).toBeVisible({ timeout: 30_000 });
  await page.waitForTimeout(250);
  return state;
};
const publicPage = async (page: Page, path: string, width = 1440) => {
  const state = await prepareMembersPage(page);
  await setViewport(page, width);
  await page.goto(path, { waitUntil: "domcontentloaded" });
  return state;
};

test.describe("SPEC-169 access links & passwords @spec169", () => {
  test.beforeEach(({}, info) => info.setTimeout(120_000));

  test("create a sign-in link from the row menu", async ({ page }) => {
    const mock = await members(page);
    await page.getByTestId(`member-actions-${DIEGO}`).click();
    await shot(page, "access-row-menu");
    await page.getByTestId("action-magic-link").click();
    await expect(page.getByTestId("access-link-dialog")).toContainText("Sign-in link for Diego Alvarez");
    await expect(page.getByTestId("access-link-warning")).toContainText("signs in as Diego Alvarez");
    await shot(page, "access-magic-confirm");

    await page.getByTestId("access-link-create").click();
    await expect(page.getByTestId("access-link-panel")).toBeVisible();
    await expect(page.getByTestId("access-link-panel")).toContainText("expires in 15 minutes");
    await expect(page.getByTestId("access-link-input")).toHaveValue(/\/access\/acc_magic_login_/);
    expect(mock.access.issued).toEqual([{ userId: DIEGO, purpose: "magic_login" }]);
    await shot(page, "access-magic-issued");
    await applyTheme(page, "dark");
    await page.waitForTimeout(300);
    await shot(page, "access-magic-issued", { theme: "dark" });
    await page.getByTestId("access-done").click();
    await expect(page.getByTestId("access-link-dialog")).toBeHidden();
  });

  test("reset a password from the detail sheet", async ({ page }) => {
    const mock = await members(page);
    await page.getByRole("button", { name: "Diego Alvarez", exact: true }).click();
    await expect(page.getByTestId("detail-access")).toBeVisible();
    await shot(page, "access-detail-sheet");
    await page.getByTestId("detail-reset-link").click();
    await expect(page.getByTestId("access-link-dialog")).toContainText("Reset password for Diego Alvarez");
    await shot(page, "access-reset-confirm");
    await page.getByTestId("access-link-create").click();
    await expect(page.getByTestId("access-link-panel")).toContainText("expires in 24 hours");
    expect(mock.access.issued).toEqual([{ userId: DIEGO, purpose: "password_reset" }]);
    await shot(page, "access-reset-issued");
  });

  test("no link actions for single sign-on people, yourself or suspended accounts", async ({ page }) => {
    await members(page);
    await page.getByTestId(`member-actions-${CAROL}`).click();
    await expect(page.getByTestId("action-copy-email")).toBeVisible();
    await expect(page.getByTestId("action-magic-link")).toHaveCount(0);
    await page.keyboard.press("Escape");
    await page.getByRole("button", { name: "Carol Nguyen", exact: true }).click();
    await expect(page.getByTestId("detail-access-sso")).toContainText("single sign-on");
    await shot(page, "access-detail-sso");
    await page.keyboard.press("Escape");
    await page.getByTestId("member-actions-spec155-user-001").click();
    await expect(page.getByTestId("action-magic-link")).toHaveCount(0);
  });

  test("magic link page signs you in once", async ({ page }) => {
    const mock = await publicPage(page, "/access/magic-ok");
    await expect(page.getByTestId("access-summary-user")).toHaveText("Diego Alvarez");
    await expect(page.getByTestId("access-signin")).toBeVisible();
    await shot(page, "access-page-magic");
    await applyTheme(page, "dark");
    await page.waitForTimeout(300);
    await shot(page, "access-page-magic", { theme: "dark" });
    await applyTheme(page, "light");
    await setViewport(page, 390);
    await page.waitForTimeout(300);
    await shot(page, "access-page-magic", { width: 390 });
    await page.getByTestId("access-signin").click();
    await expect(page).not.toHaveURL(/\/access\//);
    expect(mock.access.redeemed).toEqual([{ token: "magic-ok" }]);
  });

  test("reset page guides a strong password, then signs in", async ({ page }) => {
    const mock = await publicPage(page, "/access/reset-ok");
    const submit = page.getByTestId("reset-submit");
    await expect(page.getByTestId("password-strength")).toHaveAttribute("data-strength", "0");
    await expect(submit).toBeDisabled();
    await shot(page, "access-page-reset-empty");

    await page.getByTestId("reset-password").fill("password1");
    await expect(page.getByTestId("password-strength")).toHaveAttribute("data-strength", "1");
    await expect(page.getByTestId("password-rule-common")).toHaveAttribute("data-ok", "false");
    await expect(page.getByTestId("password-rule-variety")).toHaveAttribute("data-ok", "false");
    await expect(submit).toBeDisabled();
    await shot(page, "access-page-reset-weak");

    await page.getByTestId("reset-password").fill(STRONG_PASSWORD);
    await expect(page.getByTestId("password-strength")).toHaveAttribute("data-strength", "4");
    await expect(page.getByTestId("password-strength-label")).toHaveText("Excellent");
    await page.getByTestId("reset-password-confirm").fill(STRONG_PASSWORD + "x");
    await expect(page.getByText("Passwords do not match.")).toBeVisible();
    await expect(submit).toBeDisabled();
    await shot(page, "access-page-reset-mismatch");

    await page.getByTestId("reset-password-toggle").click();
    await expect(page.getByTestId("reset-password")).toHaveAttribute("type", "text");
    await page.getByTestId("reset-password-confirm").fill(STRONG_PASSWORD);
    await expect(submit).toBeEnabled();
    await shot(page, "access-page-reset-ready");
    await applyTheme(page, "dark");
    await page.waitForTimeout(300);
    await shot(page, "access-page-reset-ready", { theme: "dark" });
    await applyTheme(page, "light");
    await setViewport(page, 390);
    await page.waitForTimeout(300);
    await shot(page, "access-page-reset-ready", { width: 390 });

    await submit.click();
    await expect(page).not.toHaveURL(/\/access\//);
    expect(mock.access.redeemed).toEqual([{ token: "reset-ok", password: STRONG_PASSWORD }]);
  });

  test("used, expired and unknown links explain themselves", async ({ page }) => {
    await publicPage(page, "/access/expired-1");
    await expect(page.getByTestId("access-card")).toContainText("This link has expired");
    await shot(page, "access-page-expired");
    await page.goto("/access/nope");
    await expect(page.getByTestId("access-card")).toContainText("no longer valid");
    await expect(page.getByTestId("access-go-login")).toBeVisible();
    await shot(page, "access-page-invalid");
  });

  test("change your own password from the user menu", async ({ page }) => {
    const mock = await members(page);
    await page.getByRole("button", { name: /user menu/i }).click();
    await page.getByTestId("user-menu-change-password").click();
    const dialog = page.getByTestId("change-password-dialog");
    await expect(dialog).toBeVisible();
    const submit = page.getByTestId("change-password-submit");
    await expect(submit).toBeDisabled();
    await shot(page, "change-password-empty");

    await page.getByTestId("change-current").fill("wrong-Password1!");
    await page.getByTestId("change-new").fill(STRONG_PASSWORD);
    await page.getByTestId("change-new-confirm").fill(STRONG_PASSWORD);
    await submit.click();
    await expect(page.getByTestId("change-password-error")).toContainText("current password is incorrect");
    await shot(page, "change-password-wrong-current");

    await page.getByTestId("change-current").fill(CURRENT_PASSWORD);
    await submit.click();
    await expect(dialog).toBeHidden();
    expect(mock.access.passwordChanges).toEqual([{ current_password: CURRENT_PASSWORD, new_password: STRONG_PASSWORD }]);
  });

  test("invitation page: new person sets a password", async ({ page }) => {
    await publicPage(page, "/invite/inv-new");
    await expect(page.getByTestId("invite-summary-email")).toHaveText("jules.verne@acme.io");
    await page.getByTestId("accept-password").fill("Weak1");
    await expect(page.getByTestId("accept-submit")).toBeDisabled();
    await shot(page, "invite-page-new-weak");
    await page.getByTestId("accept-password").fill(STRONG_PASSWORD);
    await page.getByTestId("accept-password-confirm").fill(STRONG_PASSWORD);
    await expect(page.getByTestId("accept-submit")).toBeEnabled();
    await shot(page, "invite-page-new-ready");
    await applyTheme(page, "dark");
    await page.waitForTimeout(300);
    await shot(page, "invite-page-new-ready", { theme: "dark" });
  });

  test("invitation page: existing account, expired, invalid", async ({ page }) => {
    await publicPage(page, "/invite/inv-existing");
    await expect(page.getByTestId("invite-existing")).toBeVisible();
    await shot(page, "invite-page-existing");
    await page.goto("/invite/inv-expired");
    await expect(page.getByTestId("invite-card")).toContainText("expired");
    await shot(page, "invite-page-expired");
    await page.goto("/invite/bogus");
    await expect(page.getByTestId("invite-card")).toContainText("no longer valid");
    await shot(page, "invite-page-invalid");
  });
});
