/**
 * SPEC-015V — Vision settings panel (live stack e2e + screenshots).
 *
 * Artifacts: specs/015-vision-parser/e2e/screenshots/
 */
import { expect, test } from '@playwright/test';
import path from 'node:path';
import {
  bootstrapDeterministicUiContext,
  wizardGoNext,
} from './helpers/spec013-bootstrap';
import { skipUnlessLiveStack } from './helpers/live-stack';
import { GOTO_OPTS } from './helpers/app-ready';
import { API_V1_URL } from './helpers/spec013-api';

const SCREENSHOT_DIR = path.resolve(
  __dirname,
  '../../specs/015-vision-parser/e2e/screenshots',
);

async function shot(
  page: import('@playwright/test').Page,
  name: string,
) {
  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, name),
    fullPage: true,
  });
}

test.describe('SPEC-015V vision extract e2e', () => {
  test('documents upload Vision settings panel + captures screenshots', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec015v-docs');

    await page.goto('/documents', GOTO_OPTS);
    await expect(page.getByTestId('document-dropzone')).toBeVisible({
      timeout: 30_000,
    });
    await shot(page, '01-documents-default.png');

    const parser = page.getByTestId('spec038-upload-parser-select');
    await expect(parser).toBeVisible({ timeout: 15_000 });
    await parser.click();
    await page.getByRole('option', { name: /^Vision$/i }).click();
    await shot(page, '02-documents-vision-selected.png');

    const trigger = page.getByTestId('vision-settings-panel-trigger');
    await expect(trigger).toBeVisible({ timeout: 10_000 });
    await trigger.click();
    await expect(page.getByTestId('vision-settings-panel')).toBeVisible();
    await expect(page.getByTestId('vision-extract-controls')).toBeVisible();
    await expect(page.getByTestId('vision-extract-extractImages')).toBeVisible();
    await expect(page.getByTestId('vision-extract-extractCharts')).toBeVisible();
    await expect(page.getByTestId('vision-extract-extractFigures')).toBeVisible();

    const charts = page.getByTestId('vision-extract-extractCharts');
    await expect(charts).toHaveAttribute('aria-checked', 'true');
    await charts.click();
    await expect(charts).toHaveAttribute('aria-checked', 'false');
    await shot(page, '03-documents-charts-off.png');

    await page.getByTestId('vision-extract-advanced-toggle').click();
    await expect(page.getByTestId('vision-extract-prompts')).toBeVisible();
    // Built-in default must be visible (not a useless placeholder)
    await expect(page.getByTestId('vision-extract-prompt-chartSystemPrompt')).toHaveValue(
      /expert chart\/data-visualization analyzer/,
    );
    await expect(page.getByTestId('vision-extract-prompt-mode-chartSystemPrompt')).toHaveText(
      /Built-in/i,
    );
    await page
      .getByTestId('vision-extract-prompt-chartSystemPrompt')
      .fill('SPEC015V chart override — extract every axis label verbatim.');
    await expect(
      page.getByTestId('vision-extract-prompt-chartSystemPrompt'),
    ).toHaveValue('SPEC015V chart override — extract every axis label verbatim.');
    await expect(page.getByTestId('vision-extract-prompt-mode-chartSystemPrompt')).toHaveText(
      /Custom/i,
    );
    await shot(page, '04-documents-prompt-override.png');
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('vision-settings-panel')).toBeHidden();

    await parser.click();
    await page.getByRole('option', { name: /EdgeParse/i }).click();
    await expect(trigger).toBeHidden();
    await shot(page, '05-documents-edgeparse-hidden.png');
  });

  test('reconfigure wizard Document parsing shows Vision extract + PUT persists', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      'spec015v-wizard',
    );

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
    await shot(page, '06-wizard-document-parsing.png');

    const parserField = page.getByTestId('wizard-step-document-parsing');
    const visionOption = parserField.getByRole('combobox').first();
    if (await visionOption.isVisible().catch(() => false)) {
      await visionOption.click();
      const visionItem = page.getByRole('option', { name: /^Vision$/i });
      if (await visionItem.isVisible().catch(() => false)) {
        await visionItem.click();
      }
    }

    const controls = page.getByTestId('vision-extract-controls');
    await expect(controls).toBeVisible({ timeout: 15_000 });
    const figures = page.getByTestId('vision-extract-extractFigures');
    await expect(figures).toHaveAttribute('aria-checked', 'true');
    await figures.click();
    await expect(figures).toHaveAttribute('aria-checked', 'false');
    await shot(page, '07-wizard-figures-toggled.png');

    const put = await request.put(`${API_V1_URL}/workspaces/${ctx.workspaceId}`, {
      data: {
        vision_extract_images: true,
        vision_extract_charts: false,
        vision_extract_figures: true,
        vision_chart_system_prompt: 'E2E chart prompt',
        pdf_parser_backend: 'vision',
      },
      headers: {
        'X-Tenant-ID': ctx.tenantId,
        'X-Workspace-ID': ctx.workspaceId,
      },
    });
    expect(put.ok(), await put.text()).toBeTruthy();
    const body = await put.json();
    expect(body.vision_extract_charts).toBe(false);
    expect(body.vision_extract_figures).toBe(true);
    expect(body.vision_chart_system_prompt).toBe('E2E chart prompt');
    await shot(page, '08-wizard-after-api-put.png');
  });
});
