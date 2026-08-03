/**
 * SPEC-099 F-099-07 — Live rows hide stage subtitle (zone owns narrative).
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";
import {
  makeSpec086ListDoc,
  mockSpec086BusyPipeline,
  mockSpec086DocumentList,
} from "./helpers/spec086-ingestion-mocks";

test.describe("SPEC-099 live row no stage subtitle", () => {
  test("Active runs doc has no spec048-row-stage in table", async ({ page }) => {
    const doc = makeSpec086ListDoc({
      id: "doc-099-live-row",
      file_name: "live-row.pdf",
      status: "processing",
      current_stage: "converting",
      stage_message: "Converting",
      stage_progress: 0.5,
      track_id: "track-099-live-row",
      admission_staging: false,
      progress_counts: { unit: "pages", current: 2, total: 5 },
    });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086BusyPipeline(page);
    await mockSpec086DocumentList(page, [doc]);
    await page.goto("/documents", GOTO_OPTS);

    await expect(page.getByTestId("spec048-active-runs-panel")).toBeVisible({
      timeout: 20_000,
    });
    const row = page.getByTestId("document-row-doc-099-live-row");
    await expect(row).toBeVisible();
    await expect(row.getByTestId("spec048-row-stage")).toHaveCount(0);
  });
});
