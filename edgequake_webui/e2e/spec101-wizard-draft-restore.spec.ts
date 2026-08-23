/**
 * SPEC-101 EC-101-22 — Reconfigure session draft survives reload.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  wizardGoToReconfigureExtraction,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';
import { GOTO_OPTS } from './helpers/app-ready';

test.describe('SPEC-101 wizard draft restore', () => {
  test('reconfigure language pick restores after reload', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      'spec101-draft-restore',
    );

    await page.goto('/workspace', GOTO_OPTS);
    await expect(page.getByTestId('workspace-edit-config')).toBeVisible({
      timeout: 30_000,
    });
    await page.getByTestId('workspace-edit-config').click();
    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeVisible({
      timeout: 15_000,
    });

    await wizardGoToReconfigureExtraction(page);
    const lang = page.getByTestId('create-workspace-extraction-language');
    await lang.click();
    await page.getByRole('option', { name: 'Chinese' }).click();
    await expect(lang).toContainText('Chinese');

    const stored = await page.evaluate((workspaceId) => {
      const key = `edgequake:wizard-draft:reconfigure-workspace:${workspaceId}`;
      return sessionStorage.getItem(key);
    }, ctx.workspaceId);
    expect(stored).toBeTruthy();
    expect(stored).toContain('Chinese');

    await page.reload(GOTO_OPTS);
    await expect(page.getByTestId('workspace-edit-config')).toBeVisible({
      timeout: 30_000,
    });
    await page.getByTestId('workspace-edit-config').click();
    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeVisible({
      timeout: 15_000,
    });

    await wizardGoToReconfigureExtraction(page);
    await expect(
      page.getByTestId('create-workspace-extraction-language'),
    ).toContainText('Chinese', { timeout: 15_000 });
  });
});
