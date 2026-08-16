/**
 * SPEC-128 — layout overlay on the real PDFViewer (not a cloned HTML harness).
 *
 * Always-on: Next.js + mocked GET layout + fixture PDF bytes (G-overlay CSS IoU).
 * Live persisted: SPEC128_LIVE_* (R01–R05).
 * Live mistral: pdf_data + mistral-small-latest (M01–M05), skip without MISTRAL_API_KEY.
 *
 * Screenshots: specs/128-improve-pdf-parsing/e2e/screenshots/
 *   S01–S05 = mocked fixture (coordinate unit)
 *   R01–R05 = live document already persisted
 *   M01–M05 = pdf_data + mistral-small-latest
 */

import { expect, test, type Page, type Route } from "@playwright/test";
import * as fs from "node:fs";
import * as path from "node:path";
import { spec128Screenshot } from "./helpers/screenshot-paths";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { seedTenantStoreOnPage } from "./helpers/bootstrap-ui";
import { GOTO_OPTS } from "./helpers/app-ready";
import { createTenantWorkspaceViaApi } from "./helpers/spec013-api";
import {
  admitPdfViaApi,
  listSpec128PdfData,
  pollDocumentPageLayout,
  type PageLayoutBody,
} from "./helpers/qc-documents";

const DOC_ID = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const TENANT_ID = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const WORKSPACE_ID = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const TEXT_DOC_ID = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
const EMPTY_DOC_ID = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
const FAILED_DOC_ID = "ffffffff-ffff-4fff-8fff-ffffffffffff";

const TINY_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
  "base64",
);

/** Letter page, PDF user space 612×792. Matches rust golden bbox_norm. */
const FIGURE_NORM = { x: 0.1, y: 0.25, w: 0.4, h: 0.25 };

const FIXTURE_PDF = path.resolve(
  __dirname,
  "../../specs/128-improve-pdf-parsing/fixtures/overlay-letter.pdf",
);

const LAYOUT = {
  document_id: DOC_ID,
  page_number: 1,
  width_pt: 612,
  height_pt: 792,
  rotation: 0,
  layout_model: "l0-l1",
  layout_status: "extracted",
  regions: [
    {
      region_id: "r-figure",
      class: "figure",
      source: "l1_paint",
      bbox_pdf: { x0: 61.2, y0: 396, x1: 306, y1: 594 },
      bbox_norm: FIGURE_NORM,
      confidence: 0.9,
      reading_order: 1,
      asset_path: "assets/page-0001-fig-01.png",
      extra: {},
    },
    {
      region_id: "r-para",
      class: "paragraph",
      source: "l1_paint",
      bbox_pdf: { x0: 61.2, y0: 72, x1: 550.8, y1: 200 },
      bbox_norm: { x: 0.1, y: 0.75, w: 0.8, h: 0.16 },
      confidence: 0.7,
      reading_order: 2,
      asset_path: null,
      extra: {},
    },
    {
      region_id: "r-noise",
      class: "abandon",
      source: "l3_filter",
      bbox_pdf: { x0: 24, y0: 720, x1: 90, y1: 780 },
      bbox_norm: { x: 0.04, y: 0.015, w: 0.11, h: 0.076 },
      confidence: 0.8,
      reading_order: 0,
      asset_path: "assets/logo.png",
      extra: { figure_kind: "logo" },
    },
  ],
};

const DOCUMENT = {
  id: DOC_ID,
  title: "SPEC-128 overlay fixture",
  status: "completed",
  source_type: "pdf",
  pdf_id: DOC_ID,
  content: "# Fixture\n\n![fig](assets/page-0001-fig-01.png)\n",
  workspace_id: WORKSPACE_ID,
  tenant_id: TENANT_ID,
  file_name: "overlay-letter.pdf",
};

const TEXT_DOCUMENT = {
  id: TEXT_DOC_ID,
  title: "SPEC-128 markdown only",
  status: "completed",
  source_type: "markdown",
  content: "# Not a PDF",
  workspace_id: WORKSPACE_ID,
  tenant_id: TENANT_ID,
};

const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET,HEAD,OPTIONS,POST,PUT,PATCH,DELETE",
  "Access-Control-Allow-Headers": "*",
};

function json(route: Route, body: unknown, status = 200): Promise<void> {
  return route.fulfill({
    status,
    contentType: "application/json",
    headers: CORS,
    body: JSON.stringify(body),
  });
}

function cssIou(
  a: { x: number; y: number; width: number; height: number },
  b: { x: number; y: number; width: number; height: number },
): number {
  const ax1 = a.x + a.width;
  const ay1 = a.y + a.height;
  const bx1 = b.x + b.width;
  const by1 = b.y + b.height;
  const ix0 = Math.max(a.x, b.x);
  const iy0 = Math.max(a.y, b.y);
  const ix1 = Math.min(ax1, bx1);
  const iy1 = Math.min(ay1, by1);
  const inter = Math.max(0, ix1 - ix0) * Math.max(0, iy1 - iy0);
  const union = a.width * a.height + b.width * b.height - inter;
  return union <= 0 ? 0 : inter / union;
}

async function mockSpec128Api(page: Page): Promise<void> {
  const pdfBytes = fs.readFileSync(FIXTURE_PDF);
  const pdfB64 = pdfBytes.toString("base64");
  // Main-thread HEAD/GET (pdf-viewer probe). pdf.js worker still needs context.route.
  await page.addInitScript((b64: string) => {
    const origFetch = window.fetch.bind(window);
    window.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      if (url.includes("/documents/pdf/") && url.includes("/download")) {
        const bin = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
        return new Response(bin, {
          status: 200,
          headers: { "Content-Type": "application/pdf", "Content-Length": String(bin.length) },
        });
      }
      return origFetch(input, init);
    };
  }, pdfB64);
  await page.context().route(/\/api\/v1\/documents\/pdf\/[^/]+\/download/, async (route) => {
    if (route.request().method() === "OPTIONS") {
      return route.fulfill({ status: 204, headers: CORS });
    }
    return route.fulfill({
      status: 200,
      contentType: "application/pdf",
      headers: {
        ...CORS,
        "Content-Length": String(pdfBytes.length),
        "Accept-Ranges": "bytes",
      },
      body: pdfBytes,
    });
  });
  await page.route("**/pdf.worker.min.mjs*", async (route) => {
    const workerPath = path.resolve(
      __dirname,
      "../node_modules/pdfjs-dist/build/pdf.worker.min.mjs",
    );
    return route.fulfill({
      status: 200,
      path: workerPath,
      contentType: "text/javascript",
      headers: CORS,
    });
  });
  await page.route("**/health", (route) =>
    json(route, { status: "healthy" }),
  );
  await page.route("**/api/health", (route) =>
    json(route, { status: "healthy" }),
  );
  await page.route("**/live", (route) =>
    route.fulfill({ status: 200, body: "OK" }),
  );
  await page.route("**/ws/**", (route) =>
    route.fulfill({ status: 200, body: "" }),
  );
  await page.route("**/api/v1/**", async (route) => {
    if (route.request().method() === "OPTIONS") {
      return route.fulfill({ status: 204, headers: CORS });
    }
    const url = new URL(route.request().url());
    const p = url.pathname;
    if (p.includes(`/documents/${DOC_ID}/pages/`) && p.endsWith("/layout")) {
      return json(route, LAYOUT);
    }
    if (p.includes(`/documents/${EMPTY_DOC_ID}/pages/`) && p.endsWith("/layout")) {
      return json(route, {
        ...LAYOUT,
        document_id: EMPTY_DOC_ID,
        regions: [],
      });
    }
    if (p.includes(`/documents/${FAILED_DOC_ID}/pages/`) && p.endsWith("/layout")) {
      return json(route, {
        ...LAYOUT,
        document_id: FAILED_DOC_ID,
        layout_status: "failed",
        regions: [],
      });
    }
    if (p.endsWith(`/documents/${DOC_ID}/pages`)) {
      return json(route, {
        document_id: DOC_ID,
        pages: [
          {
            page_number: 1,
            width_pt: 612,
            height_pt: 792,
            rotation: 0,
            layout_status: "extracted",
            region_count: 3,
          },
        ],
      });
    }
    if (p.endsWith(`/documents/${EMPTY_DOC_ID}/pages`)) {
      return json(route, {
        document_id: EMPTY_DOC_ID,
        pages: [
          {
            page_number: 1,
            width_pt: 612,
            height_pt: 792,
            rotation: 0,
            layout_status: "extracted",
            region_count: 0,
          },
        ],
      });
    }
    if (p.endsWith(`/documents/${FAILED_DOC_ID}/pages`)) {
      return json(route, {
        document_id: FAILED_DOC_ID,
        pages: [
          {
            page_number: 1,
            width_pt: 612,
            height_pt: 792,
            rotation: 0,
            layout_status: "failed",
            region_count: 0,
          },
        ],
      });
    }
    if (
      p.includes(`/documents/${DOC_ID}/assets/`) ||
      p.includes(`/documents/${DOC_ID}/mm-assets/`) ||
      p.includes(`/documents/${EMPTY_DOC_ID}/assets/`)
    ) {
      return route.fulfill({
        status: 200,
        contentType: "image/png",
        headers: CORS,
        body: TINY_PNG,
      });
    }
    if (
      p.includes(`/documents/pdf/${DOC_ID}/download`) ||
      p.includes(`/documents/pdf/${EMPTY_DOC_ID}/download`) ||
      p.includes(`/documents/pdf/${FAILED_DOC_ID}/download`)
    ) {
      return route.fulfill({
        status: 200,
        contentType: "application/pdf",
        headers: {
          ...CORS,
          "Access-Control-Expose-Headers": "Content-Type,Content-Length",
          "Content-Length": String(pdfBytes.length),
        },
        body: pdfBytes,
      });
    }
    if (p.includes(`/documents/pdf/${DOC_ID}/content`)) {
      return json(route, {
        pdf_id: DOC_ID,
        document_id: DOC_ID,
        filename: "overlay-letter.pdf",
        file_size_bytes: pdfBytes.length,
        content_type: "application/pdf",
        markdown_content: "# Fixture page\n\n![fig](assets/page-0001-fig-01.png)\n",
        is_processed: true,
      });
    }
    if (p.includes(`/documents/pdf/${EMPTY_DOC_ID}/content`) || p.includes(`/documents/pdf/${FAILED_DOC_ID}/content`)) {
      const id = p.includes(EMPTY_DOC_ID) ? EMPTY_DOC_ID : FAILED_DOC_ID;
      return json(route, {
        pdf_id: id,
        document_id: id,
        filename: "overlay-letter.pdf",
        file_size_bytes: pdfBytes.length,
        content_type: "application/pdf",
        markdown_content: "# Empty layout fixture",
        is_processed: true,
      });
    }
    if (p.endsWith(`/documents/${DOC_ID}`) || p.endsWith(`/documents/${DOC_ID}/`)) {
      return json(route, DOCUMENT);
    }
    if (p.endsWith(`/documents/${EMPTY_DOC_ID}`)) {
      return json(route, {
        ...DOCUMENT,
        id: EMPTY_DOC_ID,
        pdf_id: EMPTY_DOC_ID,
        title: "SPEC-128 empty layout",
        content: "# Empty",
      });
    }
    if (p.endsWith(`/documents/${FAILED_DOC_ID}`)) {
      return json(route, {
        ...DOCUMENT,
        id: FAILED_DOC_ID,
        pdf_id: FAILED_DOC_ID,
        title: "SPEC-128 failed layout",
        content: "# Failed",
      });
    }
    if (p.endsWith(`/documents/${TEXT_DOC_ID}`)) {
      return json(route, TEXT_DOCUMENT);
    }
    if (p.includes("/tenants/") && p.includes("/workspaces")) {
      return json(route, {
        items: [
          {
            id: WORKSPACE_ID,
            tenant_id: TENANT_ID,
            name: "SPEC-128 WS",
            slug: "spec128-ws",
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
          },
        ],
        total: 1,
        offset: 0,
        limit: 20,
      });
    }
    if (p.includes("/tenants")) {
      return json(route, {
        items: [
          {
            id: TENANT_ID,
            name: "SPEC-128",
            slug: "spec128",
            plan: "free",
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
          },
        ],
        total: 1,
        offset: 0,
        limit: 20,
      });
    }
    if (p.includes("/workspaces/") && p.includes("/stats")) {
      return json(route, {
        workspace_id: WORKSPACE_ID,
        document_count: 1,
        entity_count: 0,
        relationship_count: 0,
        entity_type_count: 0,
        chunk_count: 0,
        embedding_count: 0,
        storage_bytes: 0,
        stale: false,
      });
    }
    if (p.endsWith("/documents") || p.includes("/documents?")) {
      return json(route, {
        items: [DOCUMENT],
        documents: [DOCUMENT],
        total: 1,
        offset: 0,
        limit: 10,
      });
    }
    return json(route, {});
  });
}

async function expectedFigureCss(page: Page) {
  const overlay = page.getByTestId("pdf-layout-overlay");
  const box = await overlay.boundingBox();
  if (!box) throw new Error("overlay has no CSS box");
  return {
    x: box.x + FIGURE_NORM.x * box.width,
    y: box.y + FIGURE_NORM.y * box.height,
    width: FIGURE_NORM.w * box.width,
    height: FIGURE_NORM.h * box.height,
  };
}

const runNotes: string[] = [];

async function captureLiveOverlay(
  page: Page,
  documentId: string,
  prefix: "R" | "I" | "M",
  meta: { title: string; documentId: string; source: string },
): Promise<void> {
  await page.setViewportSize({ width: 1400, height: 900 });
  const netFails: string[] = [];
  page.on("requestfailed", (r) => {
    netFails.push(`${r.method()} ${r.url()} ${r.failure()?.errorText ?? ""}`);
  });
  await page.goto(`/documents/${documentId}`, GOTO_OPTS);
  const loadErr = page.getByText("Failed to Load PDF");
  const pageCanvas = page.locator(".react-pdf__Page canvas, .react-pdf__Page");
  await expect(page.getByTestId("side-by-side-viewer")).toBeVisible({ timeout: 60_000 });
  await Promise.race([
    pageCanvas.first().waitFor({ state: "visible", timeout: 90_000 }),
    loadErr.waitFor({ state: "visible", timeout: 90_000 }),
  ]);
  if (await loadErr.isVisible().catch(() => false)) {
    throw new Error(`PDF failed to load. requestfailed=${JSON.stringify(netFails.slice(-20))}`);
  }
  await expect(page.getByTestId("pdf-viewer")).toBeVisible({ timeout: 15_000 });
  await expect(page.getByTestId("pdf-layout-toggle")).toBeVisible();
  await expect(page.getByTestId("pdf-layout-overlay")).toHaveCount(0);
  await page.screenshot({ path: spec128Screenshot(`${prefix}01-overlay-off.png`), fullPage: true });

  await page.getByTestId("pdf-layout-toggle").click();
  await expect(page.getByTestId("pdf-layout-overlay")).toBeVisible({ timeout: 30_000 });
  const boxes = page.getByTestId("pdf-layout-box");
  let paragraphsAlreadyOn = false;
  if ((await boxes.count()) === 0) {
    await page.getByTestId("pdf-layout-chip-paragraphs").click();
    paragraphsAlreadyOn = true;
  }
  await expect(boxes.first()).toBeVisible({ timeout: 20_000 });
  await page.screenshot({ path: spec128Screenshot(`${prefix}02-overlay-on.png`), fullPage: true });

  if (!paragraphsAlreadyOn) {
    await page.getByTestId("pdf-layout-chip-paragraphs").click();
  }
  await page.screenshot({
    path: spec128Screenshot(`${prefix}03-chips-paragraphs.png`),
    fullPage: true,
  });

  await page.getByTitle(/Zoom in/i).click();
  await page.getByTitle(/Zoom in/i).click();
  await expect(page.locator("text=150%")).toBeVisible({ timeout: 10_000 });
  await expect(page.getByTestId("pdf-layout-overlay")).toBeVisible({ timeout: 15_000 });
  await expect(boxes.first()).toBeVisible();
  await page.screenshot({ path: spec128Screenshot(`${prefix}04-zoom-150.png`), fullPage: true });

  await page.getByTestId("pdf-layout-chip-noise").click();
  await page.screenshot({ path: spec128Screenshot(`${prefix}05-noise-chip.png`), fullPage: true });

  const notesPath = spec128Screenshot("RUN_NOTES.md");
  const prev = fs.existsSync(notesPath)
    ? fs.readFileSync(notesPath, "utf8")
    : "# SPEC-128 overlay run notes\n\n";
  const liveBlock = `## ${prefix} live ${meta.title}\n- document_id: \`${meta.documentId}\`\n- ${meta.source}\n- Unmocked GET layout + ingested PDF bytes\n`;
  fs.writeFileSync(notesPath, `${prev}\n${liveBlock}`);
  const analysisPath = spec128Screenshot("ANALYSIS.md");
  const analysisPrev = fs.existsSync(analysisPath)
    ? fs.readFileSync(analysisPath, "utf8")
    : "";
  if (!analysisPrev.includes(meta.documentId)) {
    fs.writeFileSync(
      analysisPath,
      `${analysisPrev.trimEnd()}\n\n${liveBlock}`,
    );
  }
}

test.describe("SPEC-128 layout overlay (real PDFViewer)", () => {
  test.setTimeout(120_000);

  test("overlay off / figures / paragraphs / zoom / noise on PDFViewer", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockSpec128Api(page);
    const netFails: string[] = [];
    page.on("requestfailed", (r) => {
      netFails.push(`${r.method()} ${r.url()} ${r.failure()?.errorText ?? ""}`);
    });
    await seedTenantStoreOnPage(page, {
      tenantId: TENANT_ID,
      workspaceId: WORKSPACE_ID,
      tenantName: "SPEC-128",
      workspaceName: "SPEC-128 WS",
      workspaceSlug: "spec128-ws",
    });
    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);
    const loadErr = page.getByText("Failed to Load PDF");
    const pageCanvas = page.locator(".react-pdf__Page canvas, .react-pdf__Page");
    await expect(page.getByTestId("side-by-side-viewer")).toBeVisible({ timeout: 30_000 });
    await Promise.race([
      pageCanvas.first().waitFor({ state: "visible", timeout: 45_000 }),
      loadErr.waitFor({ state: "visible", timeout: 45_000 }),
    ]);
    if (await loadErr.isVisible().catch(() => false)) {
      throw new Error(`PDF failed to load. requestfailed=${JSON.stringify(netFails.slice(-20))}`);
    }
    await expect(page.getByTestId("pdf-viewer")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("pdf-layout-toggle")).toBeVisible();
    await expect(page.getByTestId("pdf-layout-toggle")).toBeEnabled();
    await expect(page.getByTestId("pdf-layout-toggle")).toHaveAttribute("aria-pressed", "false");
    await expect(page.getByTestId("pdf-layout-toggle")).toHaveText(/Layout/i);
    await expect(page.getByTestId("pdf-layout-overlay")).toHaveCount(0);
    await page.screenshot({ path: spec128Screenshot("S01-overlay-off.png"), fullPage: true });
    runNotes.push("### S01", "- Overlay off; real PDFViewer chrome; 0 pdf-layout-box");

    await page.getByTestId("pdf-layout-toggle").click();
    await expect(page.getByTestId("pdf-layout-toggle")).toHaveAttribute("aria-pressed", "true");
    await expect(page.getByTestId("pdf-layout-overlay")).toBeVisible({ timeout: 15_000 });
    const figure = page.locator('[data-testid=pdf-layout-box][data-layout-class=figure]');
    await expect(figure).toBeVisible();
    await expect(figure.getByTestId("pdf-layout-label")).toHaveText("figure");
    await expect(page.locator('[data-layout-class=paragraph]')).toHaveCount(0);
    await expect(page.locator('[data-layout-class=abandon]')).toHaveCount(0);
    const figBox = await figure.boundingBox();
    const expected = await expectedFigureCss(page);
    expect(figBox).toBeTruthy();
    const iou = cssIou(figBox!, expected);
    expect(iou).toBeGreaterThanOrEqual(0.8);
    await page.screenshot({ path: spec128Screenshot("S02-overlay-figures.png"), fullPage: true });
    runNotes.push("### S02", `- Figures chip default on; CSS IoU vs bbox_norm=${iou.toFixed(3)}`);

    await page.getByTestId("pdf-layout-chip-paragraphs").click();
    await expect(page.locator('[data-layout-class=paragraph]')).toBeVisible();
    await page.screenshot({ path: spec128Screenshot("S03-chips-paragraphs.png"), fullPage: true });
    runNotes.push("### S03", "- Paragraphs chip reveals paragraph box from GET layout");

    await page.getByTitle(/Zoom in/i).click();
    await page.getByTitle(/Zoom in/i).click();
    await expect(page.locator("text=150%")).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId("pdf-layout-overlay")).toBeVisible({ timeout: 15_000 });
    await expect(figure).toBeVisible();
    const figZoom = await figure.boundingBox();
    const expectedZoom = await expectedFigureCss(page);
    expect(figZoom).toBeTruthy();
    const iouZoom = cssIou(figZoom!, expectedZoom);
    expect(iouZoom).toBeGreaterThanOrEqual(0.8);
    await page.screenshot({ path: spec128Screenshot("S04-zoom-150.png"), fullPage: true });
    runNotes.push("### S04", `- Zoom 150%; IoU=${iouZoom.toFixed(3)}`);

    await page.getByTestId("pdf-layout-chip-noise").click();
    await expect(page.locator('[data-layout-class=abandon]')).toBeVisible();
    await page.screenshot({ path: spec128Screenshot("S05-noise-chip.png"), fullPage: true });
    runNotes.push("### S05", "- Noise chip shows abandon (not RAG-indexed)");

    fs.writeFileSync(
      spec128Screenshot("RUN_NOTES.md"),
      `# SPEC-128 overlay run notes\n\n${runNotes.join("\n")}\n`,
    );
  });

  test("non-PDF document has no overlay toggle", async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockSpec128Api(page);
    await seedTenantStoreOnPage(page, {
      tenantId: TENANT_ID,
      workspaceId: WORKSPACE_ID,
      tenantName: "SPEC-128",
      workspaceName: "SPEC-128 WS",
      workspaceSlug: "spec128-ws",
    });
    await page.goto(`/documents/${TEXT_DOC_ID}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-layout-toggle")).toHaveCount(0);
    await expect(page.getByTestId("pdf-viewer")).toHaveCount(0);
  });

  test("empty extracted layout shows empty copy", async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockSpec128Api(page);
    await seedTenantStoreOnPage(page, {
      tenantId: TENANT_ID,
      workspaceId: WORKSPACE_ID,
      tenantName: "SPEC-128",
      workspaceName: "SPEC-128 WS",
      workspaceSlug: "spec128-ws",
    });
    await page.goto(`/documents/${EMPTY_DOC_ID}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-viewer")).toBeVisible({ timeout: 30_000 });
    await expect(page.locator(".react-pdf__Page canvas, .react-pdf__Page").first()).toBeVisible({
      timeout: 45_000,
    });
    await expect(page.getByTestId("pdf-layout-toggle")).toBeEnabled();
    await page.getByTestId("pdf-layout-toggle").click();
    await expect(page.getByTestId("pdf-layout-empty")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("pdf-layout-empty")).toContainText(/No regions on this page/i);
  });

  test("failed layout disables Layout toggle", async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockSpec128Api(page);
    await seedTenantStoreOnPage(page, {
      tenantId: TENANT_ID,
      workspaceId: WORKSPACE_ID,
      tenantName: "SPEC-128",
      workspaceName: "SPEC-128 WS",
      workspaceSlug: "spec128-ws",
    });
    await page.goto(`/documents/${FAILED_DOC_ID}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-layout-toggle")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("pdf-layout-toggle")).toBeDisabled();
    await expect(page.getByTestId("pdf-layout-overlay")).toHaveCount(0);
  });

  test("click layout box with asset_path focuses markdown image", async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    await mockSpec128Api(page);
    await seedTenantStoreOnPage(page, {
      tenantId: TENANT_ID,
      workspaceId: WORKSPACE_ID,
      tenantName: "SPEC-128",
      workspaceName: "SPEC-128 WS",
      workspaceSlug: "spec128-ws",
    });
    await page.goto(`/documents/${DOC_ID}`, GOTO_OPTS);
    await expect(page.getByTestId("pdf-viewer")).toBeVisible({ timeout: 30_000 });
    await expect(page.locator(".react-pdf__Page canvas, .react-pdf__Page").first()).toBeVisible({
      timeout: 45_000,
    });
    await page.getByTestId("pdf-layout-toggle").click();
    const figure = page.locator('[data-testid=pdf-layout-box][data-layout-class=figure]');
    await expect(figure).toBeVisible({ timeout: 15_000 });
    await figure.click();
    await expect(page.locator('[data-layout-asset-focused="true"]')).toBeVisible({
      timeout: 10_000,
    });
  });

function skipUnlessMistralLive(): void {
  skipUnlessLiveStack();
  test.skip(
    !process.env.MISTRAL_API_KEY?.trim(),
    "Requires MISTRAL_API_KEY for mistral-small-latest vision",
  );
}

function pickFigureOrChart(layout: PageLayoutBody) {
  const regions = layout.regions ?? [];
  return (
    regions.find((r) => r.class === "figure" && r.bbox_norm) ??
    regions.find((r) => r.class === "chart" && r.bbox_norm)
  );
}

async function assertLiveCssIou(
  page: Page,
  bbox: { x: number; y: number; w: number; h: number },
  layoutClass: string,
): Promise<number> {
  const overlay = page.getByTestId("pdf-layout-overlay");
  const box = await overlay.boundingBox();
  if (!box) throw new Error("overlay has no CSS box");
  const expected = {
    x: box.x + bbox.x * box.width,
    y: box.y + bbox.y * box.height,
    width: bbox.w * box.width,
    height: bbox.h * box.height,
  };
  const target = page
    .locator(`[data-testid=pdf-layout-box][data-layout-class=${layoutClass}]`)
    .first();
  await expect(target).toBeVisible();
  const figBox = await target.boundingBox();
  expect(figBox).toBeTruthy();
  const iou = cssIou(figBox!, expected);
  expect(iou).toBeGreaterThanOrEqual(0.8);
  return iou;
}

  test("live overlay on persisted real PDF", async ({ page, request }) => {
    skipUnlessLiveStack();
    test.setTimeout(180_000);
    const envDoc = process.env.SPEC128_LIVE_DOCUMENT_ID?.trim();
    const envTenant = process.env.SPEC128_LIVE_TENANT_ID?.trim();
    const envWs = process.env.SPEC128_LIVE_WORKSPACE_ID?.trim();
    if (!envDoc || !envTenant || !envWs) {
      test.skip(
        true,
        "Set SPEC128_LIVE_DOCUMENT_ID / TENANT_ID / WORKSPACE_ID for persisted-doc overlay",
      );
      return;
    }
    const layout = await pollDocumentPageLayout(request, envDoc, envTenant, envWs, 1, 15_000);
    expect(layout.regions!.length).toBeGreaterThan(0);
    await seedTenantStoreOnPage(page, {
      tenantId: envTenant,
      workspaceId: envWs,
      tenantName: "SPEC-128 live",
      workspaceName: "SPEC-128 live",
      workspaceSlug: "spec128-live",
    });
    await captureLiveOverlay(page, envDoc, "R", {
      title: "persisted live PDF",
      documentId: envDoc,
      source: `GET layout page 1 classes=${layout.regions!.map((r) => r.class).join(",")}`,
    });
  });

  test("live mistral-small overlay on pdf_data primary", async ({ page, request }) => {
    skipUnlessMistralLive();
    test.setTimeout(600_000);
    const pdfs = listSpec128PdfData();
    if (pdfs.length === 0) {
      throw new Error(
        `No PDFs in specs/128-improve-pdf-parsing/pdf_data/ (listSpec128PdfData empty)`,
      );
    }
    const primary = pdfs[0]!;
    const ctx = await createTenantWorkspaceViaApi(request, "spec128-mistral");
    await seedTenantStoreOnPage(page, {
      tenantId: ctx.tenantId,
      workspaceId: ctx.workspaceId,
      tenantName: ctx.tenantName,
      workspaceName: ctx.workspaceName,
      workspaceSlug: ctx.workspaceSlug,
    });
    const admitted = await admitPdfViaApi(request, ctx, {
      title: path.basename(primary),
      filePath: primary,
      enableVision: true,
      parserBackend: "vision",
      visionProvider: "mistral",
      visionModel: "mistral-small-latest",
    });
    const layout = await pollDocumentPageLayout(
      request,
      admitted.documentId,
      ctx.tenantId,
      ctx.workspaceId,
      1,
      480_000,
    );
    expect(layout.regions!.length).toBeGreaterThan(0);
    const target = pickFigureOrChart(layout);
    if (!target?.bbox_norm || !target.class) {
      throw new Error(
        `primary ${path.basename(primary)} has no figure/chart bbox_norm (classes=${layout.regions!.map((r) => r.class).join(",")})`,
      );
    }
    await captureLiveOverlay(page, admitted.documentId, "M", {
      title: `${path.basename(primary)} mistral-small-latest`,
      documentId: admitted.documentId,
      source: `vision=mistral/mistral-small-latest; classes=${[...new Set(layout.regions!.map((r) => r.class))].join(",")}`,
    });
    const iou = await assertLiveCssIou(page, target.bbox_norm, target.class);
    const abandon = (layout.regions ?? []).filter((r) => r.class === "abandon");
    if (abandon.length > 0) {
      await expect(page.locator("[data-layout-class=abandon]").first()).toBeVisible();
    } else {
      expect(
        (layout.regions ?? []).some((r) => r.class === "figure" || r.class === "chart"),
      ).toBeTruthy();
    }
    const notesPath = spec128Screenshot("RUN_NOTES.md");
    const prev = fs.existsSync(notesPath) ? fs.readFileSync(notesPath, "utf8") : "";
    fs.writeFileSync(
      notesPath,
      `${prev}\n- live CSS IoU vs GET bbox_norm (${target.class})=${iou.toFixed(3)}\n`,
    );
  });

  test("live mistral corpus layout persist on remaining pdf_data", async ({ request }) => {
    skipUnlessMistralLive();
    test.setTimeout(1_200_000);
    const pdfs = listSpec128PdfData();
    if (pdfs.length < 2) {
      test.skip(true, "Need ≥2 pdf_data PDFs for corpus smoke");
      return;
    }
    const ctx = await createTenantWorkspaceViaApi(request, "spec128-corpus");
    for (const filePath of pdfs.slice(1)) {
      const admitted = await admitPdfViaApi(request, ctx, {
        title: path.basename(filePath),
        filePath,
        enableVision: true,
        parserBackend: "vision",
        visionProvider: "mistral",
        visionModel: "mistral-small-latest",
      });
      const layout = await pollDocumentPageLayout(
        request,
        admitted.documentId,
        ctx.tenantId,
        ctx.workspaceId,
        1,
        480_000,
      );
      expect(
        layout.regions!.length,
        `${path.basename(filePath)} layout empty (status=${layout.layout_status} error=${layout.error_message ?? "none"})`,
      ).toBeGreaterThan(0);
    }
  });
});
