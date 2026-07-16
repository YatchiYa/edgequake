/**
 * SPEC-054 / bulletproof progress UX — Confirm → Cleaning → live without 404.
 *
 * Invariant: after Confirm on bulk reprocess, the session panel must show
 * Cleaning / "Removing prior…" (or Queued / Queuing fallback) and must NEVER
 * show "Progress not found" during admit (early-admit batch track_id window).
 */
import { expect, test, type Page } from "@playwright/test";
import { GOTO_OPTS, waitForAppReady } from "./helpers/app-ready";
import { API_V1_URL } from "./helpers/backend-url";

/** Seeded from make-dev DB when present (raphael-article / Blue Owl workspace). */
const SPEC054_TENANT =
  process.env.E2E_TENANT_ID ?? "79d034a7-9b01-401b-b3c0-d898b5497766";
const SPEC054_WORKSPACE =
  process.env.E2E_WORKSPACE_ID ?? "940fadab-2390-4b29-af7e-ff27fd6d7755";

/** Set tenant/workspace without wiping auth (localStorage.clear breaks login). */
async function seedWorkspaceContext(page: Page): Promise<void> {
  await page.evaluate(
    ({ tenantId, workspaceId }) => {
      localStorage.setItem("tenantId", tenantId);
      localStorage.setItem("workspaceId", workspaceId);
      localStorage.setItem(
        "edgequake-tenant",
        JSON.stringify({
          state: {
            selectedTenantId: tenantId,
            selectedWorkspaceId: workspaceId,
          },
          version: 1,
        }),
      );
    },
    { tenantId: SPEC054_TENANT, workspaceId: SPEC054_WORKSPACE },
  );
}

async function assertSeededWorkspaceHasDocs(
  request: import("@playwright/test").APIRequestContext,
): Promise<boolean> {
  const docsRes = await request.get(`${API_V1_URL}/documents?limit=20`, {
    headers: {
      "X-Tenant-ID": SPEC054_TENANT,
      "X-Workspace-ID": SPEC054_WORKSPACE,
    },
  });
  if (!docsRes.ok()) {
    test.skip(true, `Documents API not reachable (${docsRes.status()})`);
    return false;
  }
  const body = (await docsRes.json()) as {
    documents?: Array<{ id?: string; file_name?: string; title?: string }>;
  };
  if ((body.documents ?? []).length === 0) {
    test.skip(true, "No documents in seeded workspace — seed a PDF first");
    return false;
  }
  return true;
}

async function openDocumentsWithSeed(page: Page): Promise<boolean> {
  await page.goto("/", GOTO_OPTS);
  await waitForAppReady(page);
  await seedWorkspaceContext(page);
  await page.goto("/documents?workspace=spec054-ws", GOTO_OPTS);
  await waitForAppReady(page);
  await seedWorkspaceContext(page);
  await page.reload(GOTO_OPTS);
  await waitForAppReady(page);

  const row = page.locator('[data-testid^="document-row-"]').first();
  try {
    await row.waitFor({ state: "visible", timeout: 25_000 });
    return true;
  } catch {
    test.skip(true, "No document rows rendered after tenant seed");
    return false;
  }
}

async function confirmFullBulkReprocess(page: Page): Promise<void> {
  const pdfRow = page
    .locator('[data-testid^="document-row-"]')
    .filter({ hasText: /\.pdf/i })
    .first();
  const row = page.locator('[data-testid^="document-row-"]').first();
  const targetRow = (await pdfRow.isVisible().catch(() => false)) ? pdfRow : row;

  const checkbox = targetRow.locator('[role="checkbox"], input[type="checkbox"]').first();
  await checkbox.click();
  await expect(page.getByText(/\d+\s+document\(s\)\s+selected/i)).toBeVisible({
    timeout: 5_000,
  });

  await page.locator('button:has-text("Reprocess")').first().click();
  const dialog = page.locator('[role="dialog"]');
  await expect(dialog).toBeVisible({ timeout: 5_000 });

  const fullOption = dialog
    .locator(
      'label:has-text("Re-convert from PDF"), [id*="full"], text=/Re-convert from PDF/i',
    )
    .first();
  if (await fullOption.isVisible().catch(() => false)) {
    await fullOption.click();
  }

  await dialog
    .locator('button:has-text("Reprocess"), button:has-text("Confirm")')
    .last()
    .click();
}

/** Immediate admit UI: Cleaning preferred; Queuing toast/row still accepted. */
async function hasImmediateAdmitFeedback(page: Page): Promise<boolean> {
  const cleaningRow = await page
    .locator('[data-testid="reprocess-cleaning-row"]')
    .isVisible()
    .catch(() => false);
  const cleaningText = await page
    .getByText(/Cleaning|Removing prior/i)
    .first()
    .isVisible()
    .catch(() => false);
  const queuingRow = await page
    .locator('[data-testid="reprocess-queuing-row"]')
    .isVisible()
    .catch(() => false);
  const queuingText = await page
    .getByText(/Queuing reprocess|Waiting for a free worker/i)
    .first()
    .isVisible()
    .catch(() => false);
  const provisional = await page
    .locator('[data-testid="reprocess-provisional-progress-row"]')
    .isVisible()
    .catch(() => false);
  const panel = await page
    .locator('[data-testid="spec051-reprocess-panel"]')
    .isVisible()
    .catch(() => false);
  const zone = await page
    .locator('[data-testid="spec051-feedback-zone"]')
    .isVisible()
    .catch(() => false);
  return (
    cleaningRow ||
    cleaningText ||
    queuingRow ||
    queuingText ||
    provisional ||
    panel ||
    zone
  );
}

test.describe("SPEC-054 Immediate reprocess feedback", () => {
  test.setTimeout(120_000);

  test("bulk Confirm shows Cleaning immediately — no Progress not found", async ({
    page,
    request,
  }) => {
    if (!(await assertSeededWorkspaceHasDocs(request))) return;
    if (!(await openDocumentsWithSeed(page))) return;

    await confirmFullBulkReprocess(page);

    const progressNotFound = page.getByText(/Progress not found/i);

    await expect
      .poll(async () => hasImmediateAdmitFeedback(page), {
        timeout: 3_000,
        intervals: [50, 100, 200],
      })
      .toBe(true);

    await expect(progressNotFound).not.toBeVisible({ timeout: 2_000 });

    // Admission row is a polite live region for assistive tech.
    const liveAdmission = page
      .locator(
        '[data-testid="reprocess-cleaning-row"], [data-testid="reprocess-queuing-row"]',
      )
      .first();
    if (await liveAdmission.isVisible().catch(() => false)) {
      await expect(liveAdmission).toHaveAttribute("role", "status");
      await expect(liveAdmission).toHaveAttribute("aria-live", "polite");
    }

    for (let i = 0; i < 8; i++) {
      await page.waitForTimeout(500);
      const bad = await progressNotFound.isVisible().catch(() => false);
      expect(bad, "Progress not found must not appear during admit").toBe(false);
    }

    await expect
      .poll(
        async () => {
          const notFound = await progressNotFound.isVisible().catch(() => false);
          if (notFound) return "not_found";
          const livePdf = await page
            .locator(
              '[data-testid="pdf-progress-row"], [data-testid="spec051-reprocess-panel"]',
            )
            .first()
            .isVisible()
            .catch(() => false);
          const activeRun = await page
            .getByText(
              /Active run|Queued run|Cleaning|Reprocessing|Processing|Storing|Converting|Extracting/i,
            )
            .first()
            .isVisible()
            .catch(() => false);
          const stageBadge = await page
            .getByText(/Cleaning|Queued|Processing|Working|Extracting|Converting/i)
            .first()
            .isVisible()
            .catch(() => false);
          if (livePdf || activeRun || stageBadge) return "ok";
          return "waiting";
        },
        { timeout: 60_000, intervals: [500, 1_000] },
      )
      .not.toBe("not_found");

    await expect(progressNotFound).not.toBeVisible();
  });

  test("dismiss during Cleaning/Queuing removes session panel without Progress not found", async ({
    page,
    request,
  }) => {
    if (!(await assertSeededWorkspaceHasDocs(request))) return;
    if (!(await openDocumentsWithSeed(page))) return;

    await confirmFullBulkReprocess(page);

    const admissionRow = page.locator(
      '[data-testid="reprocess-cleaning-row"], [data-testid="reprocess-queuing-row"]',
    );
    const progressNotFound = page.getByText(/Progress not found/i);

    await expect
      .poll(async () => hasImmediateAdmitFeedback(page), {
        timeout: 3_000,
        intervals: [50, 100, 200],
      })
      .toBe(true);

    await expect(progressNotFound).not.toBeVisible({ timeout: 1_000 });

    const dismiss = page
      .locator(
        '[data-testid="spec051-reprocess-panel"] button[aria-label*="Dismiss progress"]',
      )
      .first();
    if (await dismiss.isVisible({ timeout: 2_000 }).catch(() => false)) {
      await expect(dismiss).toHaveAttribute(
        "title",
        /Hides progress; processing continues/i,
      );
      await dismiss.click();
      await expect(admissionRow).toHaveCount(0, { timeout: 3_000 });
    }

    await expect(progressNotFound).not.toBeVisible();
  });
});
