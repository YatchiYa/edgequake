/**
 * SPEC-050 — Delete progress in unified feedback zone (not toast-only).
 *
 * After Confirm on delete, the session panel must show Deleting / phase copy
 * (or Deleting badge). Loading toast alone is not sufficient.
 */
import { expect, test, type Page } from "@playwright/test";
import { GOTO_OPTS, waitForAppReady } from "./helpers/app-ready";
import { API_V1_URL } from "./helpers/backend-url";

const SPEC050_TENANT =
  process.env.E2E_TENANT_ID ?? "79d034a7-9b01-401b-b3c0-d898b5497766";
const SPEC050_WORKSPACE =
  process.env.E2E_WORKSPACE_ID ?? "940fadab-2390-4b29-af7e-ff27fd6d7755";

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
    { tenantId: SPEC050_TENANT, workspaceId: SPEC050_WORKSPACE },
  );
}

async function listWorkspaceDocs(
  request: import("@playwright/test").APIRequestContext,
): Promise<Array<{ id: string; file_name?: string; title?: string }>> {
  const docsRes = await request.get(`${API_V1_URL}/documents?limit=20`, {
    headers: {
      "X-Tenant-ID": SPEC050_TENANT,
      "X-Workspace-ID": SPEC050_WORKSPACE,
    },
  });
  if (!docsRes.ok()) return [];
  const body = (await docsRes.json()) as {
    documents?: Array<{ id?: string; file_name?: string; title?: string }>;
  };
  return (body.documents ?? [])
    .filter((d): d is { id: string; file_name?: string; title?: string } =>
      Boolean(d.id),
    )
    .map((d) => ({
      id: d.id as string,
      file_name: d.file_name,
      title: d.title,
    }));
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
    await row.waitFor({ state: "visible", timeout: 30_000 });
    return true;
  } catch {
    test.skip(true, "No document rows rendered after tenant seed");
    return false;
  }
}

test.describe("SPEC-050 Delete feedback zone", () => {
  test.setTimeout(120_000);

  test("confirm delete shows Deleting in feedback zone — not toast-only", async ({
    page,
    request,
  }) => {
    const docs = await listWorkspaceDocs(request);
    if (docs.length === 0) {
      test.skip(true, "No documents in seeded workspace");
      return;
    }
    if (!(await openDocumentsWithSeed(page))) return;

    // Prefer a completed doc that is not currently cleaning/reprocessing.
    const target =
      docs.find((d) => (d.file_name || d.title || "").includes("latent")) ??
      docs[0];

    const targetRow = page.locator(`[data-testid="document-row-${target.id}"]`);
    if (!(await targetRow.isVisible().catch(() => false))) {
      // Fall back to first visible row
      await page.locator('[data-testid^="document-row-"]').first().waitFor({
        state: "visible",
        timeout: 10_000,
      });
    }
    const row = (await targetRow.isVisible().catch(() => false))
      ? targetRow
      : page.locator('[data-testid^="document-row-"]').first();

    // Open delete via actions menu or bulk toolbar.
    const moreBtn = row
      .locator(
        'button[aria-label*="More"], button[aria-label*="Actions"], button[aria-haspopup="menu"]',
      )
      .first();
    if (await moreBtn.isVisible().catch(() => false)) {
      await moreBtn.click();
      const deleteItem = page.getByRole("menuitem", { name: /Delete/i });
      await expect(deleteItem).toBeVisible({ timeout: 3_000 });
      await deleteItem.click();
    } else {
      const checkbox = row
        .locator('[role="checkbox"], input[type="checkbox"]')
        .first();
      await checkbox.click();
      await page.locator('button:has-text("Delete")').first().click();
    }

    const dialog = page.locator('[role="dialog"]');
    await expect(dialog).toBeVisible({ timeout: 5_000 });
    await dialog
      .locator('button:has-text("Delete"), button:has-text("Confirm")')
      .last()
      .click();

    const feedbackZone = page.locator('[data-testid="spec051-feedback-zone"]');
    const deletePanel = page.locator('[data-testid="spec050-delete-panel"]');
    const deleteRow = page.locator('[data-testid="delete-progress-row"]');

    await expect
      .poll(
        async () => {
          const zone = await feedbackZone.isVisible().catch(() => false);
          const panel = await deletePanel.isVisible().catch(() => false);
          const rowVisible = await deleteRow.isVisible().catch(() => false);
          const deletingCopy = await page
            .getByText(
              /Deleting|Removing document data|Removing graph|Removing vector/i,
            )
            .first()
            .isVisible()
            .catch(() => false);
          return zone || panel || rowVisible || deletingCopy;
        },
        { timeout: 8_000, intervals: [50, 100, 200, 400] },
      )
      .toBe(true);

    const zoneOrPanel =
      (await deletePanel.isVisible().catch(() => false)) ||
      (await deleteRow.isVisible().catch(() => false)) ||
      (await feedbackZone.isVisible().catch(() => false));

    expect(
      zoneOrPanel,
      "Delete progress must appear in the unified feedback zone",
    ).toBe(true);

    // Toast-only story is insufficient if zone is open.
    const loadingToastAlone =
      (await page.getByText(/^Deleting document/i).isVisible().catch(() => false)) &&
      !zoneOrPanel;
    expect(loadingToastAlone).toBe(false);
  });
});
