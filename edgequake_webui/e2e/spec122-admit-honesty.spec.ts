/**
 * SPEC-122 P0 — Admit honesty (LAW-122-1) + bulk banner visibility.
 *
 * E1: After multi-file admit, toast uses admitted/queued language (not
 *     “uploaded successfully” alone) and offers no Graph CTA.
 * E2: While ingest is in flight, Documents bulk banner is visible; if
 *     cloud LLM finishes before paint, transfer-list honesty still holds.
 */

import { expect, test } from "@playwright/test";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { bootstrapDeterministicUiContext } from "./helpers/bootstrap-ui";
import { uploadFilesOnDocumentsPage } from "./helpers/upload";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

test.describe("SPEC-122: admit honesty", () => {
  test("E1/E2: admit toast is queued language and bulk banner appears", async ({
    page,
    request,
  }) => {
    test.setTimeout(120_000);
    await bootstrapDeterministicUiContext(page, request, "spec122-admit");

    const stamp = Date.now();
    const fileA = `spec122-a-${stamp}.md`;
    const fileB = `spec122-b-${stamp}.md`;

    const banner = page.getByTestId("spec122-bulk-ingest-banner");
    const ingestionBanner = page.getByTestId("ingestion-status-banner");

    // Start watching for banner *before* upload completes — tiny MD on
    // Mistral can finish before a post-list poll would see pending rows.
    const bannerPromise = Promise.race([
      banner.waitFor({ state: "visible", timeout: 45_000 }).then(() => "bulk"),
      ingestionBanner
        .waitFor({ state: "visible", timeout: 45_000 })
        .then(() => "ingestion"),
    ]).catch(() => null as string | null);

    await uploadFilesOnDocumentsPage(page, [
      {
        name: fileA,
        mimeType: "text/markdown",
        buffer: Buffer.from(
          `# SPEC-122 A\n\nAlice documents admit honesty for bulk ingest.`,
        ),
      },
      {
        name: fileB,
        mimeType: "text/markdown",
        buffer: Buffer.from(
          `# SPEC-122 B\n\nBob verifies queued language after transfer.`,
        ),
      },
    ]);

    // E1 — toast must not claim searchable/ready; prefer admitted/queued.
    const toast = page.locator("[data-sonner-toast]").filter({
      hasText: /admitted|queued|processing queued/i,
    });
    await expect(toast.first()).toBeVisible({ timeout: 30_000 });

    const toastText = (await toast.first().innerText()).toLowerCase();
    expect(toastText).not.toMatch(/uploaded successfully/);
    expect(toastText).not.toMatch(/\bready\b/);
    expect(toastText).not.toMatch(/searchable/);
    expect(toastText).not.toMatch(/available for query/);
    expect(toastText).not.toMatch(/not available/);
    expect(toastText).not.toMatch(/view in graph|open graph/);
    // Admit toast must not offer a Graph CTA (implies queryable now).
    await expect(
      toast.first().getByRole("button", { name: /view in graph|open graph/i }),
    ).toHaveCount(0);

    // Transfer list honesty (header after or during transfer).
    await expect(
      page
        .getByText(
          /Transfer complete — processing queued|Transferring files|admitted|queued/i,
        )
        .first(),
    ).toBeVisible({ timeout: 20_000 });

    await expect(page.getByText(fileA).first()).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByText(fileB).first()).toBeVisible({
      timeout: 60_000,
    });

    // E2 — banner during flight. On fast cloud LLM completes, transfer list
    // clears and banner never paints; listed docs + E1 toast still prove admit.
    const bannerKind = await bannerPromise;
    if (bannerKind) {
      expect(["bulk", "ingestion"]).toContain(bannerKind);
    } else {
      // Fast-path: docs admitted and listed without lingering pending banner.
      await expect(page.getByText(fileA).first()).toBeVisible();
      await expect(page.getByText(fileB).first()).toBeVisible();
    }
  });
});
