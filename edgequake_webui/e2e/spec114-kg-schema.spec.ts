/**
 * SPEC-114 / 114b — KG schema (entity + relation + typed edges) in reconfigure wizard.
 * G-114-18: custom chip, strict, clear, lens, review impact; optional live ingest smoke.
 */
import { expect, test } from '@playwright/test';
import {
  bootstrapDeterministicUiContext,
  wizardGoNext,
  wizardGoToReconfigureExtraction,
} from './helpers/spec013-bootstrap';
import {
  mistralSpec114ExtractWorkspacePayload,
  tenantHeaders,
  SPEC013_BACKEND,
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

async function goToExtractionStep(page: import('@playwright/test').Page) {
  await openReconfigureWizard(page);
  await wizardGoToReconfigureExtraction(page);
  await expect(page.getByTestId('wizard-step-extraction')).toBeVisible({
    timeout: 15_000,
  });
}

test.describe('SPEC-114 KG schema configuration', () => {
  test('dual panels + typed edges; manufacturing domain persists', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-kg-schema');

    await goToExtractionStep(page);
    await expect(page.getByTestId('wizard-extraction-entity-types')).toBeVisible();
    await expect(page.getByTestId('wizard-extraction-relation-types')).toBeVisible();
    await expect(page.getByTestId('typed-edge-editor')).toBeVisible();

    // Domain presets are always visible (Blank + domains with relation + edge defaults).
    await expect(page.getByTestId('kg-schema-domain-grid')).toBeVisible();
    await expect(page.getByTestId('kg-schema-preset-blank')).toBeVisible();
    await page.getByTestId('kg-schema-preset-blank').click();
    await expect(page.getByTestId('kg-schema-preset-blank')).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await expect(page.getByTestId('typed-edge-list')).toContainText(/No typed edges/i);

    await page.getByTestId('kg-schema-preset-manufacturing').click();

    await expect(
      page.getByTestId('kg-schema-preset-manufacturing'),
    ).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByTestId('relation-types-chips')).toContainText('PART_OF');
    await expect(page.getByTestId('typed-edge-list')).toContainText('HAS_DEFECT');
    await expect(page.getByTestId('typed-edge-lens')).toBeVisible();

    // Entity lens filters edges
    await page.getByTestId('typed-edge-lens-MACHINE').click();
    await expect(page.getByTestId('typed-edge-list')).toContainText('MACHINE');

    await wizardGoNext(page);
    await expect(page.getByTestId('wizard-review-extraction')).toBeVisible();
    await page.getByTestId('wizard-finish').click();

    await expect(page.getByTestId('workspace-relation-types-card')).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId('ws-relation-type-PART_OF')).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByTestId('workspace-relation-edges')).toBeVisible({
      timeout: 15_000,
    });
    await expect(
      page.getByTestId('ws-relation-edge-MACHINE-HAS_DEFECT-DEFECT'),
    ).toBeVisible();
  });

  test('empty relations show free-form copy on workspace card', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-free-form');
    await page.goto('/workspace', GOTO_OPTS);
    await expect(page.getByTestId('workspace-relation-types-card')).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId('relation-types-free-form')).toBeVisible();
  });

  test('custom relation chip add persists after Apply', async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-custom-chip');
    await goToExtractionStep(page);

    await page.getByTestId('relation-type-input').fill('MENTIONS');
    await page.getByTestId('relation-type-add-btn').click();
    await expect(page.getByTestId('relation-type-chip-MENTIONS')).toBeVisible();

    await wizardGoNext(page);
    await page.getByTestId('wizard-finish').click();

    await expect(page.getByTestId('ws-relation-type-MENTIONS')).toBeVisible({
      timeout: 30_000,
    });
    await page.reload(GOTO_OPTS);
    await expect(page.getByTestId('ws-relation-type-MENTIONS')).toBeVisible({
      timeout: 15_000,
    });
  });

  test('clear relations → free-form card copy (G-114-09)', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-clear-rels');
    await goToExtractionStep(page);

    await page.getByTestId('relation-tab-advanced').click();
    await page.getByTestId('relation-advanced-clear-all').click();
    await expect(page.getByTestId('typed-edge-list')).toContainText(/No typed edges/i);

    await wizardGoNext(page);
    await page.getByTestId('wizard-finish').click();

    await expect(page.getByTestId('relation-types-free-form')).toBeVisible({
      timeout: 30_000,
    });
  });

  test('relation strict toggle persists', async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-strict-toggle');
    await goToExtractionStep(page);

    const strict = page.getByTestId('relation-types-strict-checkbox');
    await expect(strict).toBeVisible();
    // Bootstrap SPEC-114 schema defaults strict=true — flip off then Apply.
    if (await strict.isChecked()) {
      await strict.click();
    }
    await expect(strict).not.toBeChecked();

    await wizardGoNext(page);
    await page.getByTestId('wizard-finish').click();

    await expect(page.getByTestId('relation-types-strict-status')).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId('relation-types-strict-status')).toContainText(
      /Strict limit:\s*off/i,
    );

    await goToExtractionStep(page);
    await expect(page.getByTestId('relation-types-strict-checkbox')).not.toBeChecked();
  });

  test('typed-edge add + remove entity chip drops edges (EC-114-20/21)', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-edge-drop');
    await goToExtractionStep(page);

    // Bootstrap SPEC-114 schema already includes PERSON—WORKS_AT→ORGANIZATION.
    // edgeKey uses `|` separators (see kg-schema-presets.edgeKey).
    await expect(
      page.getByTestId('typed-edge-row-PERSON|WORKS_AT|ORGANIZATION'),
    ).toBeVisible();

    await page.getByTestId('typed-edge-lens-PERSON').click();
    await expect(page.getByTestId('typed-edge-list')).toContainText('WORKS_AT');

    // Removing ORGANIZATION entity chip should drop edges that reference it.
    await page.getByTestId('remove-type-ORGANIZATION').click();
    await expect(
      page.getByTestId('typed-edge-row-PERSON|WORKS_AT|ORGANIZATION'),
    ).toHaveCount(0);
  });

  test('review step shows relation/edge diff + rebuild hint (EC-114-09)', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, 'spec114-review-diff');
    await goToExtractionStep(page);

    await page.getByTestId('kg-schema-preset-manufacturing').click();
    await wizardGoNext(page);

    await expect(page.getByTestId('wizard-step-review')).toBeVisible();
    await expect(page.getByTestId('wizard-review-extraction')).toBeVisible();
    await expect(page.getByTestId('wizard-review-extraction')).toContainText(
      /PART_OF|HAS_DEFECT|Relation types|Typed edges/i,
    );
    // Impact panel may show rebuild hint when schema changed from bootstrap.
    const impact = page.getByTestId('wizard-reconfigure-impact');
    if (await impact.isVisible().catch(() => false)) {
      await expect(impact).toContainText(/Relation types|Typed edges|KG schema/i);
      const hint = page.getByTestId('wizard-reconfigure-rebuild-hint');
      const zero = page.getByTestId('wizard-reconfigure-zero-docs');
      await expect(hint.or(zero)).toBeVisible();
    }
  });

  test('optional live Mistral ingest smoke (skip without key)', async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.skip(
      !process.env.MISTRAL_API_KEY,
      'MISTRAL_API_KEY required for live extract smoke',
    );

    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      'spec114-live-smoke',
    );

    // Ensure WORKS_AT schema is applied (bootstrap already pins SPEC-114 schema).
    const put = await request.put(
      `${SPEC013_BACKEND}/api/v1/workspaces/${ctx.workspaceId}`,
      {
        headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
        data: mistralSpec114ExtractWorkspacePayload(ctx.workspaceName),
      },
    );
    expect(put.ok()).toBeTruthy();

    const upload = await request.post(`${SPEC013_BACKEND}/api/v1/documents`, {
      headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
      data: {
        title: `spec114-live-${Date.now()}`,
        content: 'Alice works at Acme in Paris.',
        async_processing: true,
      },
    });
    expect([200, 201, 202]).toContain(upload.status());
    const body = (await upload.json()) as {
      document_id?: string;
      id?: string;
      track_id?: string;
    };
    const docId = body.document_id ?? body.id;
    expect(docId).toBeTruthy();

    const deadline = Date.now() + 180_000;
    let status = 'pending';
    while (Date.now() < deadline) {
      const docRes = await request.get(
        `${SPEC013_BACKEND}/api/v1/documents/${docId}`,
        { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
      );
      if (docRes.ok()) {
        const doc = (await docRes.json()) as { status?: string };
        status = doc.status ?? status;
        if (
          ['completed', 'processed', 'indexed'].includes(status.toLowerCase())
        ) {
          break;
        }
        if (status.toLowerCase() === 'failed') {
          throw new Error(`live smoke document failed: ${JSON.stringify(doc)}`);
        }
      }
      await page.waitForTimeout(2000);
    }
    expect(['completed', 'processed', 'indexed']).toContain(status.toLowerCase());

    const graphRes = await request.get(
      `${SPEC013_BACKEND}/api/v1/graph?max_nodes=200&depth=3`,
      { headers: tenantHeaders(ctx.tenantId, ctx.workspaceId) },
    );
    expect(graphRes.ok()).toBeTruthy();
    const graph = (await graphRes.json()) as {
      edges?: Array<{ relationship_type?: string }>;
    };
    const relTypes = (graph.edges ?? [])
      .map((e) => (e.relationship_type ?? '').toUpperCase())
      .filter(Boolean);
    expect(relTypes.length).toBeGreaterThan(0);
    expect(
      relTypes.some((t) =>
        ['WORKS_AT', 'LOCATED_IN', 'RELATED_TO'].includes(t),
      ),
    ).toBeTruthy();

    await page.goto('/knowledge', GOTO_OPTS);
    // Soft UI assert — graph page should render without error.
    await expect(
      page.getByRole('heading', { name: /knowledge graph/i }).or(
        page.getByTestId('knowledge-graph'),
      ),
    ).toBeVisible({ timeout: 30_000 });
  });
});
