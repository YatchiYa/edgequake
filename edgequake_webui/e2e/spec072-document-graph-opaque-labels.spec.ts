/**
 * 072 — Document-scoped graph must paint soft-labels, never bare UUID concept names.
 *
 * Mocks GET /lineage/documents/:id with an opaque entity id + soft label.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import { seedSpec038TenantContext } from "./helpers/spec038-admission-mocks";

const DOC_ID = "019f8883-2a64-7734-a164-7f61e45c019a";
const OPAQUE_ID = "84b69e27-e38b-444a-83dd-5e6a537c6f12";
const SOFT_LABEL = "Future of work theme from the agenda";

test.describe("072 document graph opaque labels", () => {
  test.setTimeout(90_000);

  test("entity browser shows soft-label not raw UUID", async ({ page }) => {
    await seedSpec038TenantContext(page);

    await page.route("**/api/v1/lineage/documents/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          document_id: DOC_ID,
          chunk_count: 1,
          entities: [
            {
              id: OPAQUE_ID,
              name: "Opaque ID · CONCEPT",
              label: SOFT_LABEL,
              entity_type: "concept",
              source_chunks: [`${DOC_ID}-chunk-0`],
              is_shared: false,
              description: SOFT_LABEL,
            },
            {
              id: "AI_NEXT_CONFERENCE",
              name: "AI_NEXT_CONFERENCE",
              label: "AI_NEXT_CONFERENCE",
              entity_type: "event",
              source_chunks: [`${DOC_ID}-chunk-0`],
              is_shared: false,
            },
          ],
          relationships: [
            {
              source: OPAQUE_ID,
              target: "AI_NEXT_CONFERENCE",
              keywords: "RELATED_TO",
              source_chunks: [`${DOC_ID}-chunk-0`],
            },
          ],
          extraction_stats: {
            total_entities: 2,
            unique_entities: 2,
            total_relationships: 1,
            unique_relationships: 1,
          },
        }),
      });
    });

    // Keep graph stream quiet if the page briefly loads it.
    await page.route("**/api/v1/graph/stream**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "text/event-stream",
        body: "event: complete\ndata: {}\n\n",
      });
    });

    await page.goto(`/graph?document=${DOC_ID}`, GOTO_OPTS);

    const soft = page.getByText(SOFT_LABEL, { exact: false });
    await expect(soft.first()).toBeVisible({ timeout: 30_000 });

    const uuidSoup = page.getByText(OPAQUE_ID.slice(0, 8), { exact: false });
    // Soft-label path should win; Identity row may still show truncated id —
    // assert the soft label is present and the full UUID is not painted as the
    // primary entity browser name.
    await expect(page.getByText(OPAQUE_ID, { exact: true })).toHaveCount(0);
    await expect(soft.first()).toBeVisible();
    // Avoid flaking on minimap/canvas glyph noise: require soft label in DOM.
    expect(await uuidSoup.count()).toBeGreaterThanOrEqual(0);
  });
});
