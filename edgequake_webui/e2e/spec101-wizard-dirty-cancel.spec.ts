/**
 * SPEC-101 — Dirty cancel prompts before discarding wizard progress.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';
import { GOTO_OPTS } from './helpers/app-ready';

test.describe('SPEC-101 wizard dirty cancel', () => {
  test('reconfigure cancel after Next shows discard dialog; Keep stays open', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-dirty-cancel');

    await page.goto('/workspace', GOTO_OPTS);
    await expect(page.getByTestId('workspace-edit-config')).toBeVisible({
      timeout: 30_000,
    });
    await page.getByTestId('workspace-edit-config').click();
    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeVisible({
      timeout: 15_000,
    });

    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-step-document-parsing')).toBeVisible();

    await page.getByTestId('wizard-cancel').click();
    await expect(page.getByTestId('wizard-discard-confirm')).toBeVisible();
    await page.getByTestId('wizard-discard-keep').click();
    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeVisible();
    await expect(page.getByTestId('wizard-step-document-parsing')).toBeVisible();

    await page.getByTestId('wizard-cancel').click();
    await expect(page.getByTestId('wizard-discard-confirm')).toBeVisible();
    await page.getByTestId('wizard-discard-confirm-action').click();
    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeHidden({
      timeout: 15_000,
    });
  });
});
