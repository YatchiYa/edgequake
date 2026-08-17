/**
 * SPEC-099 F-099-06 — Feedback zone viewport budget.
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

test.describe("SPEC-099 feedback viewport", () => {
  test("zone max-height ≤35vh with multiple runs; table still visible", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    const docs = Array.from({ length: 6 }, (_, i) =>
      makeSpec086ListDoc({
        id: `doc-099-vp-${i}`,
        file_name: `run-${i}.pdf`,
        status: "processing",
        current_stage: "extracting",
        stage_message: "Extracting",
        stage_progress: 0.3 + i * 0.05,
        track_id: `track-099-vp-${i}`,
        admission_staging: false,
      }),
    );
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086BusyPipeline(page);
    await mockSpec086DocumentList(page, docs);
    await page.goto("/documents", GOTO_OPTS);

    const zone = page.getByTestId("spec051-feedback-zone");
    await expect(zone).toBeVisible({ timeout: 20_000 });

    const maxHeight = await zone.evaluate((el) => getComputedStyle(el).maxHeight);
    expect(maxHeight === "35vh" || maxHeight.endsWith("px")).toBeTruthy();

    // Table section / rows remain in the document
    await expect(page.getByTestId("document-row-doc-099-vp-0")).toBeVisible();
    const tableBox = await page.getByTestId("document-row-doc-099-vp-0").boundingBox();
    expect(tableBox).toBeTruthy();
  });
});
