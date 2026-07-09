/**
 * E2E — graph document filter (SPEC-045).
 *
 * First principle: `?document=` must load lineage subgraph only — never SSE stream.
 */
import { expect, test } from "@playwright/test";

import {
  GRAPH_FILTER_DOC_A,
  GRAPH_FILTER_DOC_B,
  mockGraphDocumentFilterRoutes,
  seedGraphFilterTenantContext,
} from "./helpers/graph-document-filter-mocks";

test.describe("Graph document filter (SPEC-045)", () => {
  test.beforeEach(async ({ page }) => {
    await mockGraphDocumentFilterRoutes(page);
    await seedGraphFilterTenantContext(page);
  });

  test("document query loads lineage subgraph and skips graph stream", async ({
    page,
  }) => {
    const streamRequests: string[] = [];
    const lineageRequests: string[] = [];

    page.on("request", (req) => {
      if (req.url().includes("/api/v1/graph/stream")) {
        streamRequests.push(req.url());
      }
      if (req.url().includes(`/api/v1/lineage/documents/${GRAPH_FILTER_DOC_A}`)) {
        lineageRequests.push(req.url());
      }
    });

    await page.goto(`/graph?document=${GRAPH_FILTER_DOC_A}&stream=0`);
    await page.waitForLoadState("networkidle");

    expect(lineageRequests.length).toBeGreaterThan(0);
    expect(streamRequests).toHaveLength(0);

    await expect(page.getByText(/2 nodes · 1 edge/i)).toBeVisible({
      timeout: 20000,
    });
  });

  test("switching document filter updates scoped counts", async ({ page }) => {
    await page.goto(`/graph?document=${GRAPH_FILTER_DOC_A}&stream=0`);
    await expect(page.getByText(/2 nodes · 1 edge/i)).toBeVisible({
      timeout: 20000,
    });

    await page.goto(`/graph?document=${GRAPH_FILTER_DOC_B}&stream=0`);
    await expect(page.getByText(/1 nodes · 0 edges/i)).toBeVisible({
      timeout: 20000,
    });
  });
});
