/**
 * SPEC-101 LAW-101-2 — Server defaults show LLM · Embedding · Vision explicitly.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  openCreateWorkspaceDialog,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';

const MODEL_LINE = /\S+\/\S+|not configured/i;

test.describe('SPEC-101 server defaults explicit', () => {
  test('models step shows three provider/model lines', async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-defaults');
    await openCreateWorkspaceDialog(page);
    await page.getByTestId('wizard-workspace-name').fill('spec101-defaults');
    await wizardGoNext(page);

    const card = page.getByTestId('server-defaults-card');
    await expect(card).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId('server-defaults-llm')).toContainText(MODEL_LINE);
    await expect(page.getByTestId('server-defaults-embedding')).toContainText(MODEL_LINE);
    await expect(page.getByTestId('server-defaults-vision')).toContainText(MODEL_LINE);
  });
});
