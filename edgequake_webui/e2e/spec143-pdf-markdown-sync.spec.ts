/**
 * SPEC-143 E2E — PDF continuous scroll + Markdown page sync.
 *
 * Unfakable: mocked routes + fixture PDF/markdown with page markers.
 * Asserts data-page / data-eq-page / URL — not screenshots alone.
 *
 * Run:
 *   cd edgequake_webui && pnpm exec playwright test e2e/spec143-pdf-markdown-sync.spec.ts --project=chromium
 */

import { expect, test, type Page, type Route } from "@playwright/test";
import * as path from "node:path";
import { GOTO_OPTS } from "./helpers/app-ready";
import { buildBlankPdf } from "./helpers/blank-pdf";
import {
  mockSpec038AdmissionRoutes,
  seedSpec038TenantContext,
} from "./helpers/spec038-admission-mocks";

const DOC_ID = "dddddddd-0143-0143-0143-dddddddddddd";
const DOC_NO_MARKERS = "eeeeeeee-0143-0143-0143-eeeeeeeeeeee";

async function fulfillJson(route: Route, status: number, body: unknown) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(body),
  });
}

function fixtureMarkdown(withMarkers: boolean): string {
  if (!withMarkers) {
    return ["# Fixture", "No page markers here.", "Still readable."].join("\n");
  }
  return [
    "# Fixture",
    "<!-- edgequake-page:1 -->",
    "Intro on page one. UNIQUE_MARKER_PAGE_1",
    "",
    "<!-- edgequake-page:2 -->",
    "## Page two section UNIQUE_MARKER_PAGE_2",
    "",
    "<!-- edgequake-page:3 -->",
    "Middle of the doc UNIQUE_MARKER_PAGE_3",
    "",
    "<!-- edgequake-page:4 -->",
    "## Evidence on page four",
    "",
    "UNIQUE_MARKER_PAGE_4",
  ].join("\n");
}

async function mockPdfDocumentStack(
  page: Page,
  opts: { docId: string; withMarkers: boolean; pageCount?: number },
) {
  const { docId, withMarkers } = opts;
  const pageCount = opts.pageCount ?? 4;
  await mockSpec038AdmissionRoutes(page);
  await seedSpec038TenantContext(page);

  // TenantGuard calls GET /tenants?limit=&offset= — glob without ** misses query strings
  // and the SPEC-038 catch-all returns { items: [] } → "Create Tenant" gate.
  await page.route("**/api/v1/tenants**", async (route) => {
    if (route.request().method() !== "GET") {
      await route.fallback();
      return;
    }
    const url = route.request().url();
    if (url.includes("/workspaces")) {
      await fulfillJson(route, 200, {
        items: [
          {
            id: "ws-spec038-aaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            tenant_id: "tenant-spec038-aaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            name: "SPEC-038 Workspace",
            slug: "spec038-workspace",
            llm_provider: "ollama",
            llm_model: "gemma3:latest",
            embedding_provider: "ollama",
            embedding_model: "embeddinggemma:latest",
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
          },
        ],
        total: 1,
        offset: 0,
        limit: 50,
      });
      return;
    }
    if (/\/tenants\/[^/?]+(?:\?|$)/.test(url)) {
      await fulfillJson(route, 200, {
        id: "tenant-spec038-aaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
        name: "SPEC038Tenant",
        slug: "spec038-tenant",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      });
      return;
    }
    await fulfillJson(route, 200, {
      items: [
        {
          id: "tenant-spec038-aaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
          name: "SPEC038Tenant",
          slug: "spec038-tenant",
          created_at: "2026-01-01T00:00:00Z",
          updated_at: "2026-01-01T00:00:00Z",
        },
      ],
      total: 1,
      offset: 0,
      limit: 50,
    });
  });

  await page.reload(GOTO_OPTS);

  const pdfBytes = buildBlankPdf(pageCount);
  const markdown = fixtureMarkdown(withMarkers);

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

  await page.route(`**/api/v1/documents/${docId}**`, async (route) => {
    if (route.request().method() !== "GET") {
      await route.fallback();
      return;
    }
    const url = route.request().url();
    if (url.includes("/lineage")) {
      await fulfillJson(route, 200, {
        document_id: docId,
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
              chunk_id: "page4-chunk",
              chunk_index: 3,
              start_line: 10,
              end_line: 14,
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
        document_id: docId,
        pages: Array.from({ length: pageCount }, (_, i) => ({
          page_number: i + 1,
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
      id: docId,
      title: "Fixture.pdf",
      file_name: "Fixture.pdf",
      status: "completed",
      source_type: "pdf",
      mime_type: "application/pdf",
      pdf_id: docId,
      content: markdown,
      chunk_count: 2,
      entity_count: 0,
      relationship_count: 0,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      track_id: null,
    });
  });

  await page.route(`**/api/v1/documents/pdf/${docId}/content`, async (route) => {
    await fulfillJson(route, 200, {
      pdf_id: docId,
      document_id: docId,
      filename: "Fixture.pdf",
      file_size_bytes: pdfBytes.length,
      content_type: "application/pdf",
      markdown_content: markdown,
      is_processed: true,
    });
  });
}

test.describe("SPEC-143 PDF / Markdown sync", () => {
  test("E-143-01: side-by-side shows page indicator and MD anchors", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, { docId: DOC_ID, withMarkers: true });

    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);

    await expect(page.getByTestId("side-by-side-viewer")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "1",
      { timeout: 45_000 },
    );
    await expect(page.locator('[data-eq-page="1"]').first()).toBeAttached({
      timeout: 30_000,
    });
    await expect(page.getByTestId("pdf-md-sync-toggle")).toHaveAttribute(
      "data-sync",
      "on",
    );
  });

  test("E-143-02/06: toolbar next updates data-page and URL", async ({ page }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, { docId: DOC_ID, withMarkers: true });

    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "1",
      { timeout: 45_000 },
    );

    await page.getByTestId("pdf-next-page").click();
    await expect(page.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "2",
      { timeout: 15_000 },
    );
    await expect(page).toHaveURL(/[?&]page=2(?:&|$)/, { timeout: 10_000 });
  });

  test("E-143-03: sync ON PDF page scrolls MD toward matching anchor", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, { docId: DOC_ID, withMarkers: true });

    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);
    const viewer = page.getByTestId("side-by-side-viewer");
    await expect(viewer.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "1",
      { timeout: 45_000 },
    );

    await viewer.getByTestId("pdf-next-page").click();
    await viewer.getByTestId("pdf-next-page").click();
    await viewer.getByTestId("pdf-next-page").click();
    await expect(viewer.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "4",
      { timeout: 15_000 },
    );

    await expect(viewer.getByTestId("md-page-indicator")).toHaveAttribute(
      "data-page",
      "4",
      { timeout: 10_000 },
    );
    await expect(viewer.locator("#eq-md-page-4")).toBeAttached();
  });

  test("E-143-05: sync OFF keeps markdown scrollTop when PDF page changes", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, { docId: DOC_ID, withMarkers: true });

    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-md-sync-toggle")).toBeVisible({
      timeout: 45_000,
    });

    const mdScroll = page.locator(
      '[data-testid="side-by-side-viewer"] .flex-1.min-h-0.overflow-y-auto',
    ).last();
    await mdScroll.evaluate((el) => {
      el.scrollTop = 0;
    });
    const before = await mdScroll.evaluate((el) => el.scrollTop);

    await page.getByTestId("pdf-md-sync-toggle").click();
    await expect(page.getByTestId("pdf-md-sync-toggle")).toHaveAttribute(
      "data-sync",
      "off",
    );

    await page.getByTestId("pdf-next-page").click();
    await page.getByTestId("pdf-next-page").click();
    await expect(page.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "3",
      { timeout: 15_000 },
    );

    const after = await mdScroll.evaluate((el) => el.scrollTop);
    expect(Math.abs(after - before)).toBeLessThan(8);
  });

  test("E-143-07: deeplink ?page=4 lands PDF indicator on 4", async ({ page }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, { docId: DOC_ID, withMarkers: true });

    await page.goto(`/documents/${DOC_ID}?page=4`, GOTO_OPTS);
    await expect(page).toHaveURL(/[?&]page=4(?:&|$)/);
    const viewer = page.getByTestId("side-by-side-viewer");
    await expect(viewer.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "4",
      { timeout: 45_000 },
    );
  });

  test("E-143-08: no markers disables sync toggle; PDF still navigable", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, {
      docId: DOC_NO_MARKERS,
      withMarkers: false,
    });

    await page.goto(`/documents/${DOC_NO_MARKERS}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-md-sync-toggle")).toBeDisabled({
      timeout: 45_000,
    });
    await expect(page.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "1",
      { timeout: 45_000 },
    );
    await page.getByTestId("pdf-next-page").click();
    await expect(page.getByTestId("pdf-page-indicator")).toHaveAttribute(
      "data-page",
      "2",
      { timeout: 15_000 },
    );
  });

  test("E-143-stack: continuous stack mounts all page sheets", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockPdfDocumentStack(page, { docId: DOC_ID, withMarkers: true });

    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);
    const viewer = page.getByTestId("side-by-side-viewer");
    await expect(viewer.getByTestId("pdf-scroll-container")).toBeVisible({
      timeout: 45_000,
    });
    await expect(viewer.getByTestId("pdf-page-sheet")).toHaveCount(4, {
      timeout: 45_000,
    });
    for (const n of [1, 2, 3, 4]) {
      await expect(
        viewer.locator(`[data-testid="pdf-page-sheet"][data-page="${n}"]`),
      ).toBeAttached();
    }

    // When the stack overflows the viewport, native scroll must advance active page.
    // Blank fixture pages may fit entirely — then toolbar next (same emitPage path) is the contract.
    const scroll = viewer.getByTestId("pdf-scroll-container");
    const canScroll = await scroll.evaluate(
      (el) => el.scrollHeight > el.clientHeight + 40,
    );
    if (canScroll) {
      await scroll.evaluate((el) => {
        el.scrollTop = el.scrollHeight;
      });
      await expect
        .poll(
          async () =>
            Number(
              await viewer.getByTestId("pdf-page-indicator").getAttribute("data-page"),
            ),
          { timeout: 15_000 },
        )
        .toBeGreaterThanOrEqual(2);
    } else {
      await viewer.getByTestId("pdf-next-page").click();
      await expect(viewer.getByTestId("pdf-page-indicator")).toHaveAttribute(
        "data-page",
        "2",
        { timeout: 10_000 },
      );
    }
  });
});
