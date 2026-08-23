/**
 * SPEC-101 LAW-101-8/12 — Multi-viewport reconfigure wizard evidence captures.
 */
import { expect, test, type Page } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  wizardGoNext,
  wizardGoUntilStep,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';
import { GOTO_OPTS } from './helpers/app-ready';
import { spec101E2eScreenshot, spec101Screenshot } from './helpers/screenshot-paths';

const VIEWPORTS = [
  { name: '1440', width: 1440, height: 900 },
  { name: '768', width: 768, height: 1024 },
  { name: '375', width: 375, height: 812 },
] as const;

async function captureWizard(page: Page, fileName: string) {
  const evidence = spec101Screenshot(fileName);
  const mirror = spec101E2eScreenshot(fileName);
  await page.screenshot({ path: evidence, fullPage: true });
  await page.screenshot({ path: mirror, fullPage: true });
}

async function assertDialogWithinViewport(page: Page, testId: string) {
  const box = await page.getByTestId(testId).boundingBox();
  expect(box).not.toBeNull();
  const vp = page.viewportSize();
  expect(vp).not.toBeNull();
  expect(box!.height).toBeLessThanOrEqual(vp!.height + 1);
}

test.describe('SPEC-101 reconfigure UX capture', () => {
  for (const vp of VIEWPORTS) {
    test(`reconfigure models+doc+review @ ${vp.name}`, async ({ page, request }) => {
      skipUnlessLiveStack();
      await page.setViewportSize({ width: vp.width, height: vp.height });
      await bootstrapDeterministicUiContext(
        page,
        request,
        `spec101-reconfig-cap-${vp.name}`,
      );

      await page.goto('/workspace', GOTO_OPTS);
      await expect(page.getByTestId('workspace-edit-config')).toBeVisible({
        timeout: 30_000,
      });
      await page.getByTestId('workspace-edit-config').click();
      await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeVisible();

      await expect(page.getByTestId('wizard-step-models')).toBeVisible();
      await assertDialogWithinViewport(page, 'reconfigure-workspace-wizard');
      await captureWizard(page, `after-reconfigure-models-${vp.name}.png`);

      await wizardGoNext(page);
      await expect(page.getByTestId('wizard-step-document-parsing')).toBeVisible();
      await assertDialogWithinViewport(page, 'reconfigure-workspace-wizard');
      await captureWizard(page, `after-reconfigure-document-parsing-${vp.name}.png`);

      await wizardGoUntilStep(page, 'wizard-step-extraction');
      // Force a change so Impact appears on review
      const lang = page.getByTestId('create-workspace-extraction-language');
      await lang.click();
      await page.getByRole('option', { name: 'French' }).click();

      await wizardGoNext(page); // review
      await expect(page.getByTestId('wizard-step-review')).toBeVisible();
      await expect(page.getByTestId('wizard-reconfigure-impact')).toBeVisible();
      await assertDialogWithinViewport(page, 'reconfigure-workspace-wizard');
      await captureWizard(page, `after-reconfigure-review-${vp.name}.png`);
    });
  }
});
