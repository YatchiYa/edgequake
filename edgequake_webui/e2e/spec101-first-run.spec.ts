/**
 * SPEC-101 LAW-101-4 / LAW-101-10 — Setup status API + first-run UI (non-dismissible).
 */
import { expect, test } from '@playwright/test';
import { BACKEND_URL } from './helpers/backend-url';
import { skipUnlessLiveStack } from './helpers/live-stack';

test.describe('SPEC-101 first-run status', () => {
  test('GET /api/v1/setup/status returns setup fields', async ({ request }) => {
    skipUnlessLiveStack();
    const res = await request.get(`${BACKEND_URL}/api/v1/setup/status`);
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body).toHaveProperty('needs_setup');
    expect(body).toHaveProperty('has_login_users');
    expect(body).toHaveProperty('tenant_count');
    expect(body).toHaveProperty('auth_enabled');
    expect(body).toHaveProperty('bootstrap_admin_configured');
  });

  test('mocked needs_setup shows first-run wizard without Cancel/X', async ({ page }) => {
    skipUnlessLiveStack();

    await page.route('**/api/v1/setup/status', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          needs_setup: true,
          has_login_users: false,
          tenant_count: 0,
          workspace_count: 0,
          auth_enabled: true,
          bootstrap_admin_configured: false,
        }),
      });
    });

    await page.goto('/login', { waitUntil: 'domcontentloaded' });

    const wizard = page.getByTestId('first-run-wizard');
    const loading = page.getByTestId('first-run-wizard-loading');
    await expect(wizard.or(loading)).toBeVisible({ timeout: 15_000 });
    await expect(wizard).toBeVisible({ timeout: 15_000 });

    await expect(page.getByTestId('wizard-cancel')).toHaveCount(0);
    await expect(
      page.locator('[data-testid="first-run-wizard"] [data-slot="dialog-close"]'),
    ).toHaveCount(0);
    await expect(page.getByTestId('wizard-step-live')).toBeAttached();
  });
});
