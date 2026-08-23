/**
 * SPEC-101 LAW-101-12 — Reconfigure Workspace wizard.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  seedTenantStoreOnPage,
  wizardGoNext,
  wizardGoToReconfigureReview,
} from './helpers/spec013-bootstrap';
import {
  API_V1_URL,
  MISTRAL_EMBEDDING_MODEL,
  MISTRAL_LLM_MODEL,
} from './helpers/spec013-api';
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

async function ensureAdvancedModels(page: import('@playwright/test').Page) {
  const customize = page.getByTestId('server-defaults-customize');
  if (await customize.isVisible().catch(() => false)) {
    await customize.click();
  }
  await expect(page.getByTestId('wizard-models-advanced')).toBeVisible({
    timeout: 15_000,
  });
}

/** Two-step: provider then first matching model option. */
async function pickProviderAndModel(
  page: import('@playwright/test').Page,
  picker: import('@playwright/test').Locator,
  providerId: string,
) {
  await picker.getByTestId('model-picker-provider-trigger').click();
  const providerOpt = page.getByTestId(`model-picker-provider-option-${providerId}`);
  await expect(providerOpt).toBeVisible({ timeout: 15_000 });
  await providerOpt.click();
  await expect(page.getByTestId('model-picker-panel-list')).toBeVisible({
    timeout: 15_000,
  });
  const firstModel = page.locator('[cmdk-item]').first();
  await expect(firstModel).toBeVisible({ timeout: 15_000 });
  await firstModel.click();
}

async function patchWorkspaceModels(
  request: import('@playwright/test').APIRequestContext,
  tenantId: string,
  workspaceId: string,
  body: Record<string, unknown>,
) {
  const res = await request.put(`${API_V1_URL}/workspaces/${workspaceId}`, {
    data: body,
    headers: {
      'X-Tenant-ID': tenantId,
      'X-Workspace-ID': workspaceId,
    },
  });
  if (!res.ok()) {
    throw new Error(`workspace update failed: ${res.status()} ${await res.text()}`);
  }
  return res.json();
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
    await expect(page.getByTestId('wizard-step-chunking')).toBeVisible();

    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-step-extract-budget')).toBeVisible();

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
    await wizardGoNext(page); // chunking
    await wizardGoNext(page); // extract-budget
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

  test('Customize → pick mistral LLM+embedding → Apply persists on cards', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      'spec101-reconfig-mistral',
    );

    // Start from explicit Ollama so Apply must write mistral (not no-op).
    await patchWorkspaceModels(request, ctx.tenantId, ctx.workspaceId, {
      llm_provider: 'ollama',
      llm_model: 'gemma4:latest',
      embedding_provider: 'ollama',
      embedding_model: 'embeddinggemma',
      embedding_dimension: 768,
    });
    await page.reload(GOTO_OPTS);

    await openReconfigureWizard(page);
    await ensureAdvancedModels(page);

    const llm = page.getByTestId('llm-model-selector').first();
    await pickProviderAndModel(page, llm, 'mistral');
    const emb = page.getByTestId('embedding-model-selector');
    await pickProviderAndModel(page, emb, 'mistral');

    const patchPromise = page.waitForRequest(
      (req) =>
        (req.method() === 'PUT' || req.method() === 'PATCH') &&
        req.url().includes(`/workspaces/${ctx.workspaceId}`),
      { timeout: 60_000 },
    );

    await wizardGoToReconfigureReview(page);
    await expect(page.getByTestId('wizard-finish')).toBeEnabled();
    await page.getByTestId('wizard-finish').click();

    const patchReq = await patchPromise;
    const payload = patchReq.postDataJSON() as {
      llm_provider?: string;
      llm_model?: string;
      embedding_provider?: string;
      embedding_model?: string;
      embedding_dimension?: number;
    };
    expect(payload.llm_provider).toBe('mistral');
    expect(payload.llm_model).toBeTruthy();
    expect(payload.embedding_provider).toBe('mistral');
    expect(payload.embedding_model).toBeTruthy();
    expect(payload.embedding_dimension).toBeGreaterThan(0);

    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeHidden({
      timeout: 30_000,
    });

    const ws = await request.get(`${API_V1_URL}/workspaces/${ctx.workspaceId}`, {
      headers: {
        'X-Tenant-ID': ctx.tenantId,
        'X-Workspace-ID': ctx.workspaceId,
      },
    });
    expect(ws.ok()).toBeTruthy();
    const body = (await ws.json()) as {
      llm_provider?: string;
      embedding_provider?: string;
      embedding_model?: string;
      embedding_resolution_source?: string;
      embedding_dimension?: number;
    };
    expect(body.llm_provider).toBe('mistral');
    expect(body.embedding_provider).toBe('mistral');
    expect(body.embedding_model).toBeTruthy();
    expect(body.embedding_resolution_source).toBe('workspace');
    expect(body.embedding_dimension).toBeGreaterThan(0);
  });

  test('Use tenant defaults clears overrides to tenant mistral', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    // Tenant must be created WITH mistral defaults (UpdateTenantRequest has no model fields).
    const suffix = Date.now();
    const tenantRes = await request.post(`${API_V1_URL}/tenants`, {
      data: {
        name: `spec101-reconfig-inherit ${suffix}`,
        default_llm_provider: 'mistral',
        default_llm_model: MISTRAL_LLM_MODEL,
        default_embedding_provider: 'mistral',
        default_embedding_model: MISTRAL_EMBEDDING_MODEL,
        default_embedding_dimension: 1024,
      },
    });
    expect(tenantRes.ok()).toBeTruthy();
    const tenant = (await tenantRes.json()) as { id: string };
    const wsRes = await request.post(
      `${API_V1_URL}/tenants/${tenant.id}/workspaces`,
      {
        data: {
          name: `inherit ws ${suffix}`,
          slug: `spec101-inherit-${suffix}`,
          llm_provider: 'ollama',
          llm_model: 'gemma4:latest',
          embedding_provider: 'ollama',
          embedding_model: 'embeddinggemma',
          embedding_dimension: 768,
        },
      },
    );
    expect(wsRes.ok()).toBeTruthy();
    const wsCreated = (await wsRes.json()) as { id: string };
    await seedTenantStoreOnPage(page, {
      tenantId: tenant.id,
      tenantName: `spec101-reconfig-inherit ${suffix}`,
      workspaceId: wsCreated.id,
      workspaceName: `inherit ws ${suffix}`,
      workspaceSlug: `spec101-inherit-${suffix}`,
    });

    await openReconfigureWizard(page);
    await ensureAdvancedModels(page);
    await page.getByTestId('wizard-models-use-defaults').click();
    await expect(page.getByTestId('wizard-models-advanced')).toBeHidden();

    await wizardGoToReconfigureReview(page);
    await expect(page.getByTestId('wizard-finish')).toBeEnabled();
    await page.getByTestId('wizard-finish').click();
    await expect(page.getByTestId('reconfigure-workspace-wizard')).toBeHidden({
      timeout: 30_000,
    });

    const ws = await request.get(`${API_V1_URL}/workspaces/${wsCreated.id}`, {
      headers: {
        'X-Tenant-ID': tenant.id,
        'X-Workspace-ID': wsCreated.id,
      },
    });
    expect(ws.ok()).toBeTruthy();
    const body = (await ws.json()) as {
      llm_provider?: string;
      llm_model?: string;
      embedding_provider?: string;
      embedding_model?: string;
    };
    expect(body.llm_provider).toBe('mistral');
    expect(body.llm_model).toBe(MISTRAL_LLM_MODEL);
    expect(body.embedding_provider).toBe('mistral');
    expect(body.embedding_model).toBe(MISTRAL_EMBEDDING_MODEL);
  });

  test('draft useServerDefaults=true + Advanced change sends override (not clear-all)', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      'spec101-reconfig-draft',
    );

    await patchWorkspaceModels(request, ctx.tenantId, ctx.workspaceId, {
      llm_provider: 'ollama',
      llm_model: 'gemma4:latest',
      embedding_provider: 'ollama',
      embedding_model: 'embeddinggemma',
      embedding_dimension: 768,
    });

    // Poison session draft: inherit flag true while workspace has overrides.
    await page.goto('/workspace', GOTO_OPTS);
    await page.evaluate(
      ({ workspaceId }) => {
        const key = `edgequake:wizard-draft:reconfigure-workspace:${workspaceId}`;
        sessionStorage.setItem(
          key,
          JSON.stringify({
            version: 1,
            stepIndex: 0,
            draft: {
              adminUsername: 'admin',
              adminEmail: '',
              tenantName: '',
              tenantDescription: '',
              workspaceName: 'x',
              workspaceSlug: 'x',
              workspaceDescription: '',
              useServerDefaults: true,
              extractionLanguage: null,
              entityTypes: [],
              pdfParserBackend: 'none',
              entityTypesStrict: true,
              entityTypeColors: {},
            },
          }),
        );
      },
      { workspaceId: ctx.workspaceId },
    );

    const patchPromise = page.waitForRequest(
      (req) =>
        (req.method() === 'PUT' || req.method() === 'PATCH') &&
        req.url().includes(`/workspaces/${ctx.workspaceId}`),
      { timeout: 60_000 },
    );

    await openReconfigureWizard(page);
    await ensureAdvancedModels(page);
    const llm = page.getByTestId('llm-model-selector').first();
    await pickProviderAndModel(page, llm, 'mistral');
    const emb = page.getByTestId('embedding-model-selector');
    await pickProviderAndModel(page, emb, 'mistral');

    await wizardGoToReconfigureReview(page);
    await page.getByTestId('wizard-finish').click();

    const patchReq = await patchPromise;
    const payload = patchReq.postDataJSON() as {
      llm_provider?: string;
      llm_model?: string;
      embedding_provider?: string;
    };
    expect(payload.llm_provider).toBe('mistral');
    expect(payload.llm_model).toBeTruthy();
    expect(payload.llm_model).not.toBe('');
    expect(payload.embedding_provider).toBe('mistral');
  });
});
