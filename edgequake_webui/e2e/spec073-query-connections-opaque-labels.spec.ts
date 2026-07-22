/**
 * 073 — Query Connections must show soft-labels, not bare UUID endpoints.
 *
 * Mocks a streamed query context with opaque source/target + soft labels.
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import { seedSpec038TenantContext } from "./helpers/spec038-admission-mocks";

const OPAQUE_ID = "84b69e27-e38b-444a-83dd-5e6a537c6f12";
const SOFT_SRC = "Future of work theme from the agenda";
const SOFT_TGT = "AI Next Conference";

test.describe("073 query connections opaque labels", () => {
  test.setTimeout(90_000);

  test("Connections show soft-labels not raw UUID endpoints", async ({
    page,
  }) => {
    await seedSpec038TenantContext(page);

    await page.route("**/api/v1/query/stream**", async (route) => {
      const body = [
        `event: context`,
        `data: ${JSON.stringify({
          sources: [],
          subgraph: {
            entities: [
              {
                id: OPAQUE_ID,
                name: SOFT_SRC,
                entity_type: "CONCEPT",
                description: SOFT_SRC,
                score: 0.9,
                degree: 2,
              },
              {
                id: "AI_NEXT_CONFERENCE",
                name: "AI_NEXT_CONFERENCE",
                entity_type: "EVENT",
                description: "",
                score: 0.8,
                degree: 3,
              },
            ],
            relationships: [
              {
                id: `rel:${OPAQUE_ID}:HAS_THEME:AI_NEXT_CONFERENCE`,
                source: OPAQUE_ID,
                target: "AI_NEXT_CONFERENCE",
                source_label: SOFT_SRC,
                target_label: SOFT_TGT,
                relation_type: "HAS_THEME",
                description: "",
                score: 0.7,
              },
            ],
          },
        })}`,
        ``,
        `event: token`,
        `data: ${JSON.stringify({ content: "Answer about the conference." })}`,
        ``,
        `event: done`,
        `data: {}`,
        ``,
      ].join("\n");

      await route.fulfill({
        status: 200,
        contentType: "text/event-stream",
        body,
      });
    });

    await page.goto("/query", GOTO_OPTS);

    const input = page.getByPlaceholder(/ask|question|knowledge/i).first();
    await expect(input).toBeVisible({ timeout: 30_000 });
    await input.fill("What themes does the conference cover?");
    await page.keyboard.press("Enter");

    const soft = page.getByText(SOFT_SRC, { exact: false });
    await expect(soft.first()).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(OPAQUE_ID, { exact: true })).toHaveCount(0);
  });
});
