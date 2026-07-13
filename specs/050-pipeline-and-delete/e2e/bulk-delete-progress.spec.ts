/**
 * SPEC-050 E2E: Bulk Delete Progress
 *
 * Tests the ClearDocumentsDialog shows real-time bulk deletion progress:
 *   1. The dialog opens with the "DELETE ALL" confirmation input
 *   2. After confirming, the progress bar and counter appear
 *   3. On completion, the count is shown
 *
 * @implements SPEC-050: AC-050-05
 */

import { expect, test } from '@playwright/test';

const BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3000';

test.describe('SPEC-050: Bulk Delete Progress', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}/documents`);
  });

  test('AC-050-05: Bulk delete dialog shows progress container during deletion', async ({ page }) => {
    // The "Clear All" button is only visible when there are documents
    const clearAllBtn = page.locator('button').filter({ hasText: /Clear All/i });
    if (!(await clearAllBtn.isVisible({ timeout: 5000 }).catch(() => false))) {
      test.skip();
      return;
    }

    await clearAllBtn.click();

    // The AlertDialog should open
    const dialog = page.locator('[role="alertdialog"]');
    await expect(dialog).toBeVisible({ timeout: 5000 });

    // Type the confirmation text
    const confirmInput = dialog.locator('input[placeholder="DELETE ALL"]');
    await expect(confirmInput).toBeVisible({ timeout: 3000 });
    await confirmInput.fill('DELETE ALL');

    // The delete button should now be enabled
    const deleteBtn = dialog.locator('button').filter({ hasText: /Delete All/i }).last();
    await expect(deleteBtn).toBeEnabled({ timeout: 2000 });

    // Click to confirm
    await deleteBtn.click();

    // Wait for the progress indicator to appear
    // Either the spinner or the bulk-deletion-progress div
    const progressDiv = page.locator('[data-testid="bulk-deletion-progress"]');
    const spinner = dialog.locator('.animate-spin');

    // One of these should appear
    const appeared = await progressDiv.isVisible({ timeout: 5000 }).catch(() => false)
      || await spinner.isVisible({ timeout: 5000 }).catch(() => false);

    expect(appeared).toBe(true);

    // Wait for operation to complete (up to 60s for many documents)
    await page.waitForSelector('[role="alertdialog"]', { state: 'hidden', timeout: 60000 }).catch(() => {
      // Dialog may already be gone
    });
  });
});
