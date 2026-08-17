/**
 * SPEC-099 F-099-16 — Selection mode replaces primary toolbar row.
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

test.describe("SPEC-099 selection toolbar", () => {
  test("selection actions replace (not stack under) primary toolbar", async ({
    page,
  }) => {
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086DocumentList(page, [
      makeSpec086ListDoc({
        id: "doc-099-sel-1",
        file_name: "a.pdf",
        status: "completed",
        current_stage: "completed",
        track_id: null,
        admission_staging: false,
      }),
      makeSpec086ListDoc({
        id: "doc-099-sel-2",
        file_name: "b.pdf",
        status: "completed",
        current_stage: "completed",
        track_id: null,
        admission_staging: false,
      }),
    ]);
    await page.goto("/documents", GOTO_OPTS);
    await expect(page.getByTestId("spec099-primary-toolbar")).toBeVisible({
      timeout: 20_000,
    });

    await page
      .getByTestId("document-row-doc-099-sel-1")
      .getByRole("checkbox")
      .click();

    await expect(page.getByTestId("spec099-selection-toolbar")).toBeVisible();
    await expect(page.getByTestId("batch-actions-bar")).toBeVisible();
    // Primary search/filter row replaced — not stacked
    await expect(page.getByTestId("spec099-primary-toolbar")).toHaveCount(0);
  });
});
