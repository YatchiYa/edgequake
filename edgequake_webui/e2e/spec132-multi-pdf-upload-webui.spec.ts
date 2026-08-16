/**
 * SPEC-132 / GH-378: WebUI multi-PDF upload (N× concurrent single-file admits).
 *
 * Proves Documents dropzone accepts ≥2 PDFs and both rows appear after admit.
 * Does not require full KG Completed — Plane A (admit + list presence) is the gate.
 */

import { expect, test } from "@playwright/test";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { bootstrapDeterministicUiContext } from "./helpers/bootstrap-ui";
import { uploadFilesOnDocumentsPage } from "./helpers/upload";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

/** Minimal distinct PDF bytes so content-hash duplicate detection does not collide. */
function minimalPdfBuffer(unique: string): Buffer {
  const body = `%PDF-1.4
1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj
2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj
3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] /Contents 4 0 R /Resources<< /Font<< /F1 5 0 R >> >> >>endobj
4 0 obj<< /Length 44 >>stream
BT /F1 24 Tf 40 80 Td (${unique}) Tj ET
endstream endobj
5 0 obj<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>endobj
trailer<< /Size 6 /Root 1 0 R >>
%%EOF
UNIQUE_${unique}_${"x".repeat(32)}
`;
  return Buffer.from(body, "latin1");
}

test.describe("SPEC-132: WebUI multi-PDF upload", () => {
  test("dropzone uploads two PDFs and both appear in the table", async ({
    page,
    request,
  }) => {
    test.setTimeout(180_000);
    await bootstrapDeterministicUiContext(page, request, "spec132-multi-pdf");

    const stamp = Date.now();
    const fileA = `gh378-pdf-a-${stamp}.pdf`;
    const fileB = `gh378-pdf-b-${stamp}.pdf`;

    await uploadFilesOnDocumentsPage(page, [
      {
        name: fileA,
        mimeType: "application/pdf",
        buffer: minimalPdfBuffer(`A${stamp}`),
      },
      {
        name: fileB,
        mimeType: "application/pdf",
        buffer: minimalPdfBuffer(`B${stamp}`),
      },
    ]);

    // Plane A gate: both filenames appear (admit + list). Do not require progress-chrome
    // copy — it can race past Transferring before the assertion, and "files" matches
    // unrelated hidden chrome.
    await expect(page.getByText(fileA).first()).toBeVisible({
      timeout: 90_000,
    });
    await expect(page.getByText(fileB).first()).toBeVisible({
      timeout: 90_000,
    });

    const bodyText = await page.locator("body").innerText();
    expect(bodyText.toLowerCase()).not.toContain("all 2 file(s) failed");
  });
});
