/**
 * SPEC-101 LAW-101-11 — One-line Tenant — Workspace; Organization → Workspace.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  openWorkspaceSelectorMenu,
} from './helpers/spec013-bootstrap';
import { waitForAppReady } from './helpers/app-ready';
import {
  API_V1_URL,
  createTenantWorkspaceViaApi,
  mistralWorkspacePayload,
} from './helpers/spec013-api';
import { skipUnlessLiveStack } from './helpers/live-stack';

async function clickCommandItemByTitle(page: import('@playwright/test').Page, title: string) {
  // cmdk items expose title=; role=option name matching is flaky with long labels.
  await page.locator(`[cmdk-item][title="${title}"]`).click();
}

test.describe('SPEC-101 context selector', () => {
  test('trigger shows one-line Tenant — Workspace', async ({ page, request }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(page, request, 'spec101-ctx');

    const trigger = page.locator('[data-testid="workspace-selector"]:visible').first();
    await expect(trigger).toBeVisible({ timeout: 15_000 });

    const line = page.getByTestId('context-line').first();
    await expect(line).toBeVisible();
    await expect(line).toHaveCSS('flex-direction', 'row');

    const tenantLabel = page.getByTestId('context-tenant-label').first();
    const workspaceLabel = page.getByTestId('context-workspace-label').first();
    await expect(tenantLabel).toHaveAttribute('data-full-name', ctx.tenantName, {
      timeout: 15_000,
    });
    await expect(workspaceLabel).toHaveAttribute('data-full-name', ctx.workspaceName, {
      timeout: 15_000,
    });

    await expect(trigger).toHaveAttribute('title', `${ctx.tenantName} — ${ctx.workspaceName}`);
    const aria = await trigger.getAttribute('aria-label');
    expect(aria ?? '').toContain(ctx.tenantName);
    expect(aria ?? '').toContain(ctx.workspaceName);
  });

  test('select Organization then Workspace; popover stays open after tenant', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const primary = await bootstrapDeterministicUiContext(page, request, 'spec101-ctx-a');
    const other = await createTenantWorkspaceViaApi(request, 'spec101-ctx-b');

    // Refresh so React Query picks up the API-created organization.
    await page.reload({ waitUntil: 'domcontentloaded' });
    await waitForAppReady(page);

    await openWorkspaceSelectorMenu(page);

    const tenantsGroup = page.getByTestId('context-selector-tenants');
    const workspacesGroup = page.getByTestId('context-selector-workspaces');
    await expect(tenantsGroup).toBeVisible();
    await expect(workspacesGroup).toBeVisible();

    // Organizations appear above Workspaces (tenant → workspace).
    const tnBox = await tenantsGroup.boundingBox();
    const wsBox = await workspacesGroup.boundingBox();
    expect(tnBox && wsBox).toBeTruthy();
    if (tnBox && wsBox) {
      expect(tnBox.y).toBeLessThan(wsBox.y);
    }

    await expect(workspacesGroup).toContainText(primary.workspaceName);
    await expect(page.getByTestId('context-selector-current')).toHaveCount(0);

    // Filter list so the new org is visible without scrolling forever.
    await page.getByPlaceholder(/Search organizations/i).fill(other.tenantName);
    await expect(page.locator(`[cmdk-item][title="${other.tenantName}"]`)).toBeVisible({
      timeout: 10_000,
    });
    await clickCommandItemByTitle(page, other.tenantName);

    // Clear search so workspace items for the new tenant are listed.
    await page.getByPlaceholder(/Search organizations/i).fill('');
    await expect(page.getByTestId('context-selector-workspaces')).toContainText(other.workspaceName, {
      timeout: 15_000,
    });

    await clickCommandItemByTitle(page, other.workspaceName);

    await expect(page.getByTestId('context-tenant-label').first()).toHaveAttribute(
      'data-full-name',
      other.tenantName,
      { timeout: 10_000 },
    );
    await expect(page.getByTestId('context-workspace-label').first()).toHaveAttribute(
      'data-full-name',
      other.workspaceName,
      { timeout: 10_000 },
    );

    await expect
      .poll(() => new URL(page.url()).searchParams.get('workspace'), { timeout: 10_000 })
      .toMatch(new RegExp(other.workspaceSlug.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  });

  test('selecting a workspace updates context-workspace-label', async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec101-ctx-ws');

    const trigger = page.locator('[data-testid="workspace-selector"]:visible').first();
    await expect(trigger).toBeVisible();

    const tenantId = await page.evaluate(() => localStorage.getItem('tenantId'));
    expect(tenantId).toBeTruthy();

    const name = `ctx-ws-${Date.now()}`;
    const slug = `ctx-ws-${Date.now()}`;
    const createRes = await request.post(`${API_V1_URL}/tenants/${tenantId}/workspaces`, {
      data: { ...mistralWorkspacePayload(name), slug },
    });
    expect(createRes.ok(), await createRes.text()).toBeTruthy();

    await page.reload({ waitUntil: 'domcontentloaded' });
    await waitForAppReady(page);

    await openWorkspaceSelectorMenu(page);
    await page.getByPlaceholder(/Search organizations/i).fill(name);
    await expect(page.locator(`[cmdk-item][title="${name}"]`)).toBeVisible({ timeout: 10_000 });
    await clickCommandItemByTitle(page, name);

    await expect(page.getByTestId('context-workspace-label').first()).toHaveAttribute(
      'data-full-name',
      name,
      { timeout: 10_000 },
    );
  });
});
