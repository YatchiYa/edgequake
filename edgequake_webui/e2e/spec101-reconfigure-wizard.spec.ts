/**
 * SPEC-101 LAW-101-12 — Reconfigure Workspace wizard.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';
import { GOTO_OPTS } from './helpers/app-ready';

async function openReconfigureWizard(page: import('@playwright/test').Page) {
  await page.goto('/workspace', GOTO_OPTS);
  await expect(page.getByTestId('workspace-edit-config')).toBeVisible({
    timeout: 30_000,
  });
  await page.getByTestId('workspace-edit-config').click();
  await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeVisible({
    timeout: 15_000,
  });
}

test.describe('SPEC-101 reconfigure workspace wizard', () => {
  test('opens from Edit Configuration and walks all steps', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-reconfig');

    await openReconfigureWizard(page);
    await expect(page.getByTestId('wizard-step-models')).toBeVisible();
    await expect(page.getByTestId('server-defaults-card')).toBeVisible();

    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-step-document-parsing')).toBeVisible();

    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-step-extraction')).toBeVisible();
    await expect(page.getByTestId('create-workspace-extraction-language')).toBeVisible();

    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-step-review')).toBeVisible();
    // No-op Apply disabled when nothing changed (EC-101-27)
    await expect(page.getByTestId('wizard-finish')).toBeDisabled();
  });

  test('applies extraction language change and updates card', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-reconfig-lang');

    await openReconfigureWizard(page);
    await wizardGoNext(page); // document-parsing
    await wizardGoNext(page); // extraction

    const lang = page.getByTestId('create-workspace-extraction-language');
    await lang.click();
    await page.getByRole('option', { name: 'Chinese' }).click();

    await wizardGoNext(page); // review
    await expect(page.getByTestId('wizard-reconfigure-impact')).toBeVisible();
    await expect(page.getByTestId('wizard-finish')).toBeEnabled();
    await page.getByTestId('wizard-finish').click();

    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeHidden({
      timeout: 30_000,
    });
    await expect(page.getByTestId('ws-extraction-language-value')).toContainText(
      'Chinese',
      { timeout: 15_000 },
    );
  });

  test('PDF parser step shows never-silent server default (Vision)', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-reconfig-pdf');

    await openReconfigureWizard(page);
    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-step-document-parsing')).toBeVisible();
    const select = page.getByTestId('pdf-parser-backend-select');
    await expect(select).toBeVisible();
    // Never silent: must disclose resolved backend (Vision by default).
    await expect(select).toContainText(/Server Default\s*\(\s*Vision\s*\)/i);
  });
});
