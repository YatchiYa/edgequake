/**
 * SPEC-099 F-099-02 / LAW-099-3 — Composite StatusCell (no peer Ready pill).
 */
import { expect, test, type Page } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";
import {
  makeSpec086ListDoc,
  mockSpec086DocumentList,
} from "./helpers/spec086-ingestion-mocks";

async function gotoCompletedFence(page: Page) {
  const ready = makeSpec086ListDoc({
    id: "doc-099-ready",
    file_name: "ready-099.pdf",
    status: "completed",
    current_stage: "completed",
    stage_message: "Completed",
    stage_progress: 1,
    source_type: "pdf",
    track_id: null,
    admission_staging: false,
    query_ready: true,
  });
  const indexed = makeSpec086ListDoc({
    id: "doc-099-indexed",
    file_name: "indexed-099.pdf",
    status: "completed",
    current_stage: "completed",
    stage_message: "Completed",
    stage_progress: 1,
    source_type: "pdf",
    track_id: null,
    admission_staging: false,
    query_ready: false,
  });
  await mockSpec038AdmissionRoutes(page);
  await seedSpec038TenantContext(page);
  await mockSpec086DocumentList(page, [ready, indexed]);
  await page.goto("/documents", GOTO_OPTS);
  await expect(page.getByText("ready-099.pdf").first()).toBeVisible({
    timeout: 20_000,
  });
}

test.describe("SPEC-099 StatusCell fence presentation", () => {
  test("one Status cell; Ready is not a peer success pill; data-query-ready retained", async ({
    page,
  }) => {
    await gotoCompletedFence(page);

    const readyRow = page.getByTestId("document-row-doc-099-ready");
    const indexedRow = page.getByTestId("document-row-doc-099-indexed");

    await expect(readyRow.getByTestId("status-cell")).toBeVisible();
    await expect(indexedRow.getByTestId("status-cell")).toBeVisible();

    // Fence attributes preserved for SPEC-091
    await expect(
      readyRow.locator(
        '[data-testid="spec091-serving-fence-badge"][data-query-ready="true"]',
      ),
    ).toBeVisible();
    await expect(
      indexedRow.locator(
        '[data-testid="spec091-serving-fence-badge"][data-query-ready="false"]',
      ),
    ).toContainText("not queryable");

    // No peer dual emerald Badge pills: only one status-cell per row
    await expect(readyRow.getByTestId("status-cell")).toHaveCount(1);
    await expect(indexedRow.getByTestId("status-cell")).toHaveCount(1);

    // A11y name includes fence
    await expect(readyRow.getByTestId("status-cell")).toHaveAttribute(
      "aria-label",
      /Ready/i,
    );
    await expect(indexedRow.getByTestId("status-cell")).toHaveAttribute(
      "aria-label",
      /not yet queryable/i,
    );

    // Indexed fence must not use emerald peer "Ready" success paint
    await expect(indexedRow.getByText("Ready", { exact: true })).toHaveCount(0);
  });
});
