/**
 * GH-350 / SPEC-098: WebUI multi-file upload (N× concurrent single-file admits).
 *
 * Proves the Documents dropzone accepts ≥2 text files and both rows appear.
 * Does not require full KG Completed (AGE) — admission + list presence is the gate.
 */

import { expect, test } from "@playwright/test";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { bootstrapDeterministicUiContext } from "./helpers/bootstrap-ui";
import { uploadFilesOnDocumentsPage } from "./helpers/upload";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

test.describe("GH-350: WebUI multi-file upload", () => {
  test("dropzone uploads two text files and both appear in the table", async ({
    page,
    request,
  }) => {
    test.setTimeout(120_000);
    await bootstrapDeterministicUiContext(page, request, "spec350-bulk-up");

    const stamp = Date.now();
    const fileA = `gh350-bulk-a-${stamp}.md`;
    const fileB = `gh350-bulk-b-${stamp}.md`;

    await uploadFilesOnDocumentsPage(page, [
      {
        name: fileA,
        mimeType: "text/markdown",
        buffer: Buffer.from(
          `# GH-350 bulk A\n\nAlice works at EdgeQuake on graph RAG.`,
        ),
      },
      {
        name: fileB,
        mimeType: "text/markdown",
        buffer: Buffer.from(
          `# GH-350 bulk B\n\nBob leads embeddings for the knowledge graph.`,
        ),
      },
    ]);

    await expect(
      page
        .getByText(
          /Processing Files|Transferring files|Transfer complete|Upload Complete|Uploading|files|admitted|queued/i,
        )
        .first(),
    ).toBeVisible({ timeout: 20_000 });

    await expect(page.getByText(fileA).first()).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByText(fileB).first()).toBeVisible({
      timeout: 60_000,
    });

    // Must not surface the #350 persist fingerprint in the UI feedback.
    const bodyText = await page.locator("body").innerText();
    expect(bodyText.toLowerCase()).not.toContain('type "agtype" does not exist');
  });
});
