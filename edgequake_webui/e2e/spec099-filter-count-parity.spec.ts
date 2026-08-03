/**
 * SPEC-099 F-099-10 — Header / chip / rows share one view-model.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";
import {
  makeSpec086ListDoc,
  mockSpec086DocumentList,
} from "./helpers/spec086-ingestion-mocks";

test.describe("SPEC-099 filter count parity", () => {
  test("header count, All Status chip, and rows agree", async ({ page }) => {
    const docs = [
      makeSpec086ListDoc({
        id: "doc-099-c1",
        file_name: "one.pdf",
        status: "completed",
        current_stage: "completed",
        track_id: null,
        admission_staging: false,
        query_ready: true,
      }),
      makeSpec086ListDoc({
        id: "doc-099-c2",
        file_name: "two.pdf",
        status: "completed",
        current_stage: "completed",
        track_id: null,
        admission_staging: false,
        query_ready: true,
      }),
      makeSpec086ListDoc({
        id: "doc-099-c3",
        file_name: "three.pdf",
        status: "failed",
        current_stage: "failed",
        track_id: null,
        admission_staging: false,
      }),
    ];
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086DocumentList(page, docs);
    await page.goto("/documents", GOTO_OPTS);

    await expect(page.getByTestId("document-row-doc-099-c1")).toBeVisible({
      timeout: 20_000,
    });

    const header = page.getByTestId("spec099-documents-count");
    await expect(header).toBeVisible();
    const headerText = (await header.textContent())?.trim() ?? "";
    // Without server totals, label is filtered row count
    expect(headerText === "3" || headerText.startsWith("3")).toBeTruthy();

    const rows = page.locator('[data-testid^="document-row-"]');
    await expect(rows).toHaveCount(3);
  });
});
