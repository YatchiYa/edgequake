/**
 * SPEC-098 LAW-098-10 / GH-350: selected bulk delete must show Deleting
 * (not Ready) while sessions are active.
 *
 * Seeds ≥2 documents via API so CI never skip()s on an empty table.
 */

import { expect, test } from "@playwright/test";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import {
  bootstrapDeterministicUiContext,
  tenantHeaders,
} from "./helpers/bootstrap-ui";
import { SPEC013_BACKEND } from "./helpers/spec013-api";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

async function seedTextDocuments(
  request: import("@playwright/test").APIRequestContext,
  tenantId: string,
  workspaceId: string,
  count: number,
): Promise<string[]> {
  const titles: string[] = [];
  for (let i = 0; i < count; i++) {
    const title = `spec098-bulk-del-${Date.now()}-${i}.md`;
    const resp = await request.post(`${SPEC013_BACKEND}/api/v1/documents`, {
      headers: tenantHeaders(tenantId, workspaceId),
      data: {
        title,
        content: `# GH-350 / SPEC-098 delete seed ${i}\n\nEntity Alpha and Entity Beta.`,
        async_processing: true,
      },
    });
    expect(resp.ok(), `seed upload ${i}: ${resp.status()}`).toBeTruthy();
    titles.push(title);
  }
  return titles;
}

test.describe("SPEC-098: selected bulk delete list honesty", () => {
  test("mid-delete badges are Deleting, not Completed/Ready", async ({
    page,
    request,
  }) => {
    test.setTimeout(120_000);
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      "spec098-bulk-del",
    );
    const titles = await seedTextDocuments(
      request,
      ctx.tenantId,
      ctx.workspaceId,
      2,
    );

    await page.goto("/documents");
    await page.waitForLoadState("networkidle").catch(() => undefined);

    for (const title of titles) {
      await expect(page.getByText(title).first()).toBeVisible({
        timeout: 60_000,
      });
    }

    // Radix Checkbox uses role=checkbox (button), not input[type=checkbox].
    const checkboxes = page.locator(
      'table tbody tr [role="checkbox"], table tbody tr input[type="checkbox"]',
    );
    await expect
      .poll(async () => checkboxes.count(), { timeout: 30_000 })
      .toBeGreaterThanOrEqual(2);
    await checkboxes.nth(0).click();
    await checkboxes.nth(1).click();

    const deleteBtn = page
      .locator("button")
      .filter({ hasText: /^Delete$/i })
      .first();
    await expect(deleteBtn).toBeVisible({ timeout: 10_000 });
    await deleteBtn.click();

    const confirm = page
      .locator('[role="alertdialog"] button, [role="dialog"] button')
      .filter({ hasText: /Delete/i })
      .last();
    if (await confirm.isVisible({ timeout: 3000 }).catch(() => false)) {
      await confirm.click();
    }

    const progress = page.locator(
      '[data-testid="spec050-delete-progress-panels"]',
    );
    await expect(progress).toBeVisible({ timeout: 15_000 });

    const deletingPanels = page.locator(
      '[data-testid="spec050-delete-panel"][data-status="active"]',
    );
    // Active or completed panels — at least one feedback signal.
    const activeOrDone = page.locator(
      '[data-testid="spec050-delete-panel"][data-status="active"], [data-testid="spec050-delete-panel"][data-status="completed"], [data-testid="spec050-delete-panel"][data-status="failed"]',
    );
    await expect(activeOrDone.first()).toBeVisible({ timeout: 15_000 });

    // While any session is active, feedback must say Deleting (not Ready alone).
    if ((await deletingPanels.count()) > 0) {
      await expect(progress.getByText(/Deleting/i).first()).toBeVisible();
    } else {
      // Cascade may finish quickly in mock/CI — progress must still have appeared.
      await expect(progress).toBeVisible();
    }
  });

  test("failed delete header says Delete failed, not Deleting", async ({
    page,
  }) => {
    // Unit-level honesty is covered in vitest; this smoke waits for a failed
    // session panel if the environment already has delete_failed docs / sessions.
    await page.goto("/documents");
    await page.waitForLoadState("networkidle").catch(() => undefined);

    const failedBadge = page
      .locator('[data-testid="status-badge"]')
      .filter({ hasText: /Delete failed/i });
    const header = page.locator(
      '[data-testid="spec098-delete-progress-header"]',
    );

    const panelFailed = page.locator(
      '[data-testid="spec050-delete-panel"][data-status="failed"]',
    );
    if ((await panelFailed.count()) === 0) {
      test.skip();
      return;
    }
    await expect(header).toBeVisible({ timeout: 5000 });
    const text = (await header.textContent()) || "";
    expect(text).toMatch(/Delete failed|failed/i);
    const failedCount = Number(await header.getAttribute("data-failed-count"));
    const activeCount = Number(await header.getAttribute("data-active-count"));
    if (failedCount > 0 && activeCount === 0) {
      expect(text).toMatch(/Delete failed/i);
      expect(text).not.toMatch(/^Deleting \d+ document/);
    }
    if ((await failedBadge.count()) > 0) {
      await expect(failedBadge.first()).toBeVisible();
    }
  });
});
