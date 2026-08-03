/**
 * SPEC-099 F-099-05 — Clear All demoted from Refresh peer.
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

test.describe("SPEC-099 Clear All demoted", () => {
  test("Clear All not adjacent peer to Refresh; overflow + typed confirm", async ({
    page,
  }) => {
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086DocumentList(page, [
      makeSpec086ListDoc({
        id: "doc-099-clear",
        file_name: "keep.pdf",
        status: "completed",
        current_stage: "completed",
        track_id: null,
        admission_staging: false,
      }),
    ]);
    await page.goto("/documents", GOTO_OPTS);
    await expect(page.getByTestId("documents-refresh-button")).toBeVisible({
      timeout: 20_000,
    });

    // Not a primary peer button next to Refresh
    await expect(page.getByTestId("spec099-clear-all-button")).toHaveCount(0);

    await page.getByTestId("spec099-documents-overflow").click();
    await expect(page.getByTestId("spec099-clear-all-menu-item")).toBeVisible();
    await page.getByTestId("spec099-clear-all-menu-item").click();

    // Typed confirm still required
    await expect(page.getByRole("alertdialog")).toBeVisible();
    await expect(page.getByLabel(/Type DELETE ALL/i)).toBeVisible();
  });
});
