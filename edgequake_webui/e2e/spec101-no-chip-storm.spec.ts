/**
 * SPEC-101 LAW-101-3 — Happy-path model step has zero external chip bars;
 * Advanced uses two-step provider → model selects.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  openCreateWorkspaceDialog,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';

test.describe('SPEC-101 no chip storm', () => {
  test('two-step provider→model on Customize; no external chip bars', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-chips');
    await openCreateWorkspaceDialog(page);
    await page.getByTestId('wizard-workspace-name').fill('spec101-chips');
    await wizardGoNext(page);

    await expect(page.getByTestId('wizard-step-models')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId('model-picker-provider-bar')).toHaveCount(0);
    await expect(page.getByTestId('model-picker-provider-in-popover')).toHaveCount(0);

    const customize = page.getByTestId('server-defaults-customize');
    if (await customize.isVisible().catch(() => false)) {
      await customize.click();
      await expect(page.getByTestId('wizard-models-advanced')).toBeVisible();
      await expect(page.getByTestId('model-picker-provider-bar')).toHaveCount(0);

      const llm = page.getByTestId('llm-model-selector').first();
      const providerTrigger = llm.getByTestId('model-picker-provider-trigger');
      await expect(providerTrigger).toBeVisible();
      await providerTrigger.click();

      const providerList = page.getByTestId('model-picker-provider-list');
      await expect(providerList).toBeVisible({ timeout: 10_000 });
      const providerOptions = providerList.locator('[cmdk-item]');
      expect(await providerOptions.count()).toBeGreaterThan(0);

      await providerOptions.first().click();

      const modelTrigger = llm.getByTestId('model-picker-panel-trigger');
      await expect(modelTrigger).toBeEnabled({ timeout: 5_000 });
      // Selecting a provider auto-opens the model popover.
      await expect(page.getByTestId('model-picker-panel-list')).toBeVisible({
        timeout: 10_000,
      });
    }
  });
});
