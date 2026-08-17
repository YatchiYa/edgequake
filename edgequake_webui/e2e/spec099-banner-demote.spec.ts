/**
 * SPEC-099 F-099-14 — Non-stuck pipeline banner demoted when zone open.
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

test.describe("SPEC-099 banner demote", () => {
  test("non-stuck pipeline banner hidden when feedback zone open", async ({
    page,
  }) => {
    const doc = makeSpec086ListDoc({
      id: "doc-099-banner",
      file_name: "banner.pdf",
      status: "processing",
      current_stage: "embedding",
      stage_message: "Embedding",
      stage_progress: 0.6,
      track_id: "track-099-banner",
      admission_staging: false,
    });
    await mockSpec038AdmissionRoutes(page);
    await seedSpec038TenantContext(page);
    await mockSpec086BusyPipeline(page);
    await mockSpec086DocumentList(page, [doc]);
    await page.goto("/documents", GOTO_OPTS);

    await expect(page.getByTestId("spec051-feedback-zone")).toBeVisible({
      timeout: 20_000,
    });
    // Non-stuck banner (ingestion-status-banner) should be demoted
    await expect(page.getByTestId("ingestion-status-banner")).toHaveCount(0);
  });
});
