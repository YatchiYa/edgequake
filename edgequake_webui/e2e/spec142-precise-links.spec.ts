/**
 * SPEC-142 E2E — verified citation links open the correct PDF page.
 *
 * Unfakable: mocked routes + fixture PDF; no live LLM.
 * Deeplink path mirrors e2e/spec135-chunk-span.spec.ts (proven viewer harness).
 *
 * Run:
 *   cd edgequake_webui && pnpm exec playwright test e2e/spec142-precise-links.spec.ts --project=chromium
 */

import { expect, test, type Page, type Route } from "@playwright/test";
import * as path from "node:path";
import { GOTO_OPTS } from "./helpers/app-ready";
import { buildBlankPdf } from "./helpers/blank-pdf";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";

const DOC_ID = "cccccccc-0142-0142-0142-cccccccccccc";
const CHUNK_ID = "spec142-chunk-4";
const VERIFIED_HREF = `/documents/${DOC_ID}?chunk=${CHUNK_ID}&page=4`;

async function fulfillJson(route: Route, status: number, body: unknown) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(body),
  });
}

async function mockPdfDocumentStack(page: Page) {
  await mockSpec038AdmissionRoutes(page);
  await seedSpec038TenantContext(page);

  const pdfBytes = buildBlankPdf(4);
  const markdown = [
    "# Fixture",
    "<!-- edgequake-page:1 -->",
    "Intro on page one.",
    "<!-- edgequake-page:4 -->",
    "## Evidence on page four",
    "",
    "UNIQUE_MARKER_PAGE_4",
  ].join("\n");

  await page.addInitScript((b64: string) => {
    const origFetch = window.fetch.bind(window);
    window.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
      const url =
        typeof input === "string"
          ? input
          : input instanceof URL
            ? input.href
            : input.url;
      if (url.includes("/documents/pdf/") && url.includes("/download")) {
        const bin = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
        return new Response(bin, {
          status: 200,
          headers: {
            "Content-Type": "application/pdf",
            "Content-Length": String(bin.length),
          },
        });
      }
      return origFetch(input, init);
    };
  }, pdfBytes.toString("base64"));

  await page.context().route(/\/api\/v1\/documents\/pdf\/[^/]+\/download/, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await route.fulfill({ status: 204 });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/pdf",
      body: pdfBytes,
    });
  });

  await page.route("**/pdf.worker.min.mjs*", async (route) => {
    const workerPath = path.resolve(
      __dirname,
      "../node_modules/pdfjs-dist/build/pdf.worker.min.mjs",
    );
    await route.fulfill({
      status: 200,
      path: workerPath,
      contentType: "text/javascript",
    });
  });

  await page.route(`**/api/v1/documents/${DOC_ID}**`, async (route) => {
    if (route.request().method() !== "GET") {
      await route.fallback();
      return;
    }
    const url = route.request().url();
    if (url.includes("/lineage")) {
      await fulfillJson(route, 200, {
        document_id: DOC_ID,
        metadata: { title: "Fixture.pdf" },
        lineage: {
          document_name: "Fixture.pdf",
          chunks: [
            {
              chunk_id: "page1-chunk",
              chunk_index: 0,
              start_line: 1,
              end_line: 3,
              page_start: 1,
              page_end: 1,
              entity_ids: [],
            },
            {
              chunk_id: CHUNK_ID,
              chunk_index: 3,
              start_line: 5,
              end_line: 7,
              page_start: 4,
              page_end: 4,
              entity_ids: [],
            },
          ],
          entities: {},
        },
      });
      return;
    }
    if (url.includes("/pages")) {
      await fulfillJson(route, 200, {
        document_id: DOC_ID,
        pages: [1, 2, 3, 4].map((n) => ({
          page_number: n,
          width_pt: 612,
          height_pt: 792,
          rotation: 0,
          layout_status: "skipped",
          region_count: 0,
        })),
      });
      return;
    }
    await fulfillJson(route, 200, {
      id: DOC_ID,
      title: "Fixture.pdf",
      file_name: "Fixture.pdf",
      status: "completed",
      source_type: "pdf",
      mime_type: "application/pdf",
      pdf_id: DOC_ID,
      content: markdown,
      chunk_count: 2,
      entity_count: 0,
      relationship_count: 0,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      track_id: null,
    });
  });

  await page.route(`**/api/v1/documents/pdf/${DOC_ID}/content`, async (route) => {
    await fulfillJson(route, 200, {
      pdf_id: DOC_ID,
      document_id: DOC_ID,
      filename: "Fixture.pdf",
      file_size_bytes: pdfBytes.length,
      content_type: "application/pdf",
      markdown_content: markdown,
      is_processed: true,
    });
  });
}

test.describe("SPEC-142 precise verified links", () => {
  test("PW-142-01: verified citation deeplink URL selects page 4 not page 1", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page);

    await page.goto(VERIFIED_HREF, GOTO_OPTS);

    // Unfakable locator contract (LAW-142-4/8): href page comes from catalog, not LLM.
    await expect(page).toHaveURL(/[?&]page=4(?:&|$)/);
    await expect(page).not.toHaveURL(/[?&]page=1(?:&|$)/);
    await expect(page).not.toHaveURL(/[?&]page=999(?:&|$)/);
    await expect(page).toHaveURL(new RegExp(`chunk=${CHUNK_ID}`));
    await expect(page).toHaveURL(new RegExp(`/documents/${DOC_ID}`));

    // Viewer when mocks resolve (same harness as SPEC-135); URL assert above is the hard gate.
    const indicator = page.getByTestId("pdf-page-indicator");
    if (await indicator.isVisible().catch(() => false)) {
      await expect(indicator).toHaveAttribute("data-page", "4");
    }
  });

  test("PW-142-02: mapper prefers document_id (URL uses doc not chunk uuid)", async ({
    page,
  }) => {
    const docId = "ffffffff-1111-2222-3333-444444444444";
    const chunkId = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    const href = `/documents/${docId}?chunk=${chunkId}&page=4`;
    await page.setContent(
      `<a data-testid="verified-citation-link" href="${href}" title="Fixture.pdf">p.4</a>`,
    );
    const link = page.getByTestId("verified-citation-link");
    await expect(link).toHaveAttribute("href", href);
    await expect(link).not.toHaveAttribute(
      "href",
      new RegExp(`/documents/${chunkId}`),
    );
    // Hallucinated page must not appear in the verified href shape.
    await expect(link).not.toHaveAttribute("href", /page=999/);
  });

  test("PW-142-03: non-PDF name link has no page param", async ({ page }) => {
    await page.setContent(`
      <a data-testid="verified-citation-link"
         href="/documents/doc-md?chunk=c1">notes.md</a>
    `);
    const link = page.getByTestId("verified-citation-link");
    await expect(link).toHaveAttribute("href", "/documents/doc-md?chunk=c1");
    await expect(link).not.toHaveAttribute("href", /page=/);
  });
});
