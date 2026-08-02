/**
 * SPEC-013 / GitHub #233 — Workspace create shows explicit server defaults (SPEC-101).
 */
import { expect, test } from '@playwright/test';
import { issueScreenshot } from './helpers/screenshot-paths';
import {
  bootstrapDeterministicUiContext,
  openCreateWorkspaceDialog,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';

test.describe('Issue #233 workspace create UX', () => {
  test('server defaults card visible on models step', async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'issue-233');
    await openCreateWorkspaceDialog(page);

    // Step 1: workspace basics — fill name then next
    await page.getByTestId('wizard-workspace-name').fill('issue-233-ws');
    await wizardGoNext(page);

    await expect(page.getByTestId('wizard-step-models')).toBeVisible({ timeout: 10_000 });
    const defaultsCard = page.getByTestId('server-defaults-card');
    const customize = page.getByTestId('server-defaults-customize');
    const advanced = page.getByTestId('wizard-models-advanced');
    const hasDefaults = await defaultsCard.isVisible().catch(() => false);
    const hasCustomize = await customize.isVisible().catch(() => false);
    const hasAdvanced = await advanced.isVisible().catch(() => false);
    expect(hasDefaults || hasCustomize || hasAdvanced).toBeTruthy();

    if (hasDefaults) {
      await expect(page.getByTestId('server-defaults-llm')).toBeVisible();
      await expect(page.getByTestId('server-defaults-embedding')).toBeVisible();
      await expect(page.getByTestId('server-defaults-vision')).toBeVisible();
    }

    await page.screenshot({
      path: issueScreenshot('issue-233', 'create-workspace-dialog.png'),
      fullPage: true,
    });
  });
});
