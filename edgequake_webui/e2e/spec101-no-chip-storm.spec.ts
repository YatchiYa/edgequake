/**
 * SPEC-101 LAW-101-3 — Happy-path model step has zero provider chip bars.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  openCreateWorkspaceDialog,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';

test.describe('SPEC-101 no chip storm', () => {
  test('provider filter bar absent until Customize models', async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-chips');
    await openCreateWorkspaceDialog(page);
    await page.getByTestId('wizard-workspace-name').fill('spec101-chips');
    await wizardGoNext(page);

    await expect(page.getByTestId('wizard-step-models')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId('model-picker-provider-bar')).toHaveCount(0);

    const customize = page.getByTestId('server-defaults-customize');
    if (await customize.isVisible().catch(() => false)) {
      await customize.click();
      // Advanced may show pickers without provider chip bars (showProviderFilters=false)
      await expect(page.getByTestId('wizard-models-advanced')).toBeVisible();
      await expect(page.getByTestId('model-picker-provider-bar')).toHaveCount(0);
    }
  });
});
