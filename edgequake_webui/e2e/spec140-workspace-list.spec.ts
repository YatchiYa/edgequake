/**
 * SPEC-140 — Newly created workspaces must appear in the header popover (#388).
 */
import { expect, test } from '@playwright/test';
import {
  openWorkspaceSelectorMenu,
  seedTenantStoreOnPage,
} from './helpers/spec013-bootstrap';
import { waitForAppReady } from './helpers/app-ready';
import { API_V1_URL } from './helpers/spec013-api';
import { skipUnlessLiveStack } from './helpers/live-stack';

type ListPayload = {
  items?: Array<{ name?: string }>;
  total?: number;
};

async function createProTenant(
  request: import('@playwright/test').APIRequestContext,
  name: string,
): Promise<{ id: string; name: string; slug: string }> {
  const res = await request.post(`${API_V1_URL}/tenants`, {
    data: { name, plan: 'pro' },
  });
  expect(res.ok(), await res.text()).toBeTruthy();
  const body = (await res.json()) as { id: string; slug?: string };
  return { id: body.id, name, slug: body.slug ?? name };
}

async function createNamedWorkspace(
  request: import('@playwright/test').APIRequestContext,
  tenantId: string,
  name: string,
  slug: string,
): Promise<{ id: string; name: string; slug: string }> {
  const res = await request.post(`${API_V1_URL}/tenants/${tenantId}/workspaces`, {
    data: { name, slug },
  });
  expect(res.ok(), await res.text()).toBeTruthy();
  const body = (await res.json()) as { id: string; slug?: string };
  return { id: body.id, name, slug: body.slug ?? slug };
}

async function listWorkspaces(
  request: import('@playwright/test').APIRequestContext,
  tenantId: string,
): Promise<ListPayload> {
  const res = await request.get(`${API_V1_URL}/tenants/${tenantId}/workspaces`);
  expect(res.ok(), await res.text()).toBeTruthy();
  return (await res.json()) as ListPayload;
}

test.describe('SPEC-140 workspace list completeness', () => {
  test('three named workspaces on one tenant all appear in the popover', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const suffix = `${Date.now()}`;
    const tenant = await createProTenant(request, `spec140-3ws ${suffix}`);
    const names = [`g99-71-${suffix}`, `g99-72-${suffix}`, `g99-73-${suffix}`];
    const created = [];
    for (const name of names) {
      const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-');
      created.push(await createNamedWorkspace(request, tenant.id, name, slug));
    }

    const list = await listWorkspaces(request, tenant.id);
    expect(list.total ?? 0).toBeGreaterThanOrEqual(3);

    const listFromUi = page.waitForResponse((res) => {
      const url = res.url();
      return (
        res.request().method() === 'GET' &&
        url.includes(`/tenants/${tenant.id}/workspaces`) &&
        res.ok()
      );
    });

    await seedTenantStoreOnPage(page, {
      tenantId: tenant.id,
      tenantName: tenant.name,
      workspaceId: created[2].id,
      workspaceName: created[2].name,
      workspaceSlug: created[2].slug,
    });
    const uiList = (await (await listFromUi).json()) as ListPayload;
    expect(uiList.total ?? 0).toBeGreaterThanOrEqual(3);
    const uiNames = (uiList.items ?? []).map((w) => w.name);
    for (const name of names) {
      expect(uiNames).toContain(name);
    }

    await openWorkspaceSelectorMenu(page);
    const search = page.getByTestId('context-selector-search');
    if (await search.isVisible().catch(() => false)) {
      await search.fill('');
    }
    for (const ws of created) {
      await expect(page.getByTestId(`workspace-option-${ws.slug}`)).toBeVisible({
        timeout: 15_000,
      });
    }
  });

  test('21st workspace remains reachable in the popover', async ({ page, request }) => {
    skipUnlessLiveStack();
    const suffix = `${Date.now()}`;
    const tenant = await createProTenant(request, `spec140-21 ${suffix}`);
    const oldestName = `spec140-oldest-${suffix}`;
    const oldest = await createNamedWorkspace(
      request,
      tenant.id,
      oldestName,
      `spec140-oldest-${suffix}`,
    );
    let lastId = oldest.id;
    let lastName = oldest.name;
    let lastSlug = oldest.slug;
    for (let i = 0; i < 20; i += 1) {
      const created = await createNamedWorkspace(
        request,
        tenant.id,
        `spec140-n${i}-${suffix}`,
        `spec140-n${i}-${suffix}`,
      );
      lastId = created.id;
      lastName = created.name;
      lastSlug = created.slug;
    }

    const list = await listWorkspaces(request, tenant.id);
    expect(list.total ?? 0).toBeGreaterThanOrEqual(21);

    const listFromUi = page.waitForResponse((res) => {
      const url = res.url();
      return (
        res.request().method() === 'GET' &&
        url.includes(`/tenants/${tenant.id}/workspaces`) &&
        res.ok()
      );
    });

    await seedTenantStoreOnPage(page, {
      tenantId: tenant.id,
      tenantName: tenant.name,
      workspaceId: lastId,
      workspaceName: lastName,
      workspaceSlug: lastSlug,
    });
    const uiList = (await (await listFromUi).json()) as ListPayload;
    expect(uiList.total ?? 0).toBeGreaterThanOrEqual(21);

    await openWorkspaceSelectorMenu(page);
    const search = page.getByTestId('context-selector-search');
    await search.fill('');
    const optionCount = await page.locator('[data-testid^="workspace-option-"]').count();
    if ((uiList.total ?? 0) >= 21) {
      expect(
        optionCount,
        `popover hid rows: DOM=${optionCount} API total=${uiList.total}`,
      ).toBeGreaterThanOrEqual(21);
    }

    await search.fill(oldestName);
    await expect(page.getByTestId(`workspace-option-${oldest.slug}`)).toBeVisible({
      timeout: 15_000,
    });
  });

  test('three organizations each expose their named workspace', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const suffix = `${Date.now()}`;
    const orgs = [];
    for (const n of ['g99-71', 'g99-72', 'g99-73']) {
      const tenantName = `${n}-org-${suffix}`;
      const tenant = await createProTenant(request, tenantName);
      const wsName = `${n}-ws-${suffix}`;
      const slug = `${n}-ws-${suffix}`.toLowerCase();
      const ws = await createNamedWorkspace(request, tenant.id, wsName, slug);
      orgs.push({ tenant, ws });
    }

    await seedTenantStoreOnPage(page, {
      tenantId: orgs[2].tenant.id,
      tenantName: orgs[2].tenant.name,
      workspaceId: orgs[2].ws.id,
      workspaceName: orgs[2].ws.name,
      workspaceSlug: orgs[2].ws.slug,
    });
    await waitForAppReady(page);

    await openWorkspaceSelectorMenu(page);
    for (const org of orgs) {
      const search = page.getByTestId('context-selector-search');
      await search.fill(org.tenant.name);
      const tenantOption = page.getByTestId(`tenant-option-${org.tenant.slug}`);
      await expect(tenantOption).toBeVisible({ timeout: 15_000 });
      await tenantOption.click();
      await search.fill('');
      await expect(page.getByTestId(`workspace-option-${org.ws.slug}`)).toBeVisible({
        timeout: 15_000,
      });
    }
  });
});
