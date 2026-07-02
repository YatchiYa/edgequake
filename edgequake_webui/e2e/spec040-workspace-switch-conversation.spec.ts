/**
 * SPEC-040 #259 — workspace switch clears stale conversation context.
 */
import { expect, test } from "@playwright/test";
import { waitForAppReady } from "./helpers/app-ready";
import { skipUnlessLiveStack } from "./helpers/live-stack";

const FAKE_CONVERSATION_ID = "00000000-0000-0000-0000-000000000001";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

test.describe("@audit SPEC-040 workspace switch conversation", () => {
  test("clears activeConversationId when workspace changes", async ({ page }) => {
    await page.goto("/");
    await page.evaluate((conversationId) => {
      localStorage.setItem(
        "edgequake-query-ui",
        JSON.stringify({
          state: {
            historyPanelOpen: true,
            activeConversationId: conversationId,
            filters: {
              mode: null,
              archived: false,
              pinned: null,
              folderId: null,
              search: "",
              dateFrom: null,
              dateTo: null,
            },
            sort: { field: "updated_at", order: "desc" },
          },
          version: 0,
        }),
      );
    }, FAKE_CONVERSATION_ID);

    await page.goto("/query");
    await waitForAppReady(page);

    const beforeSwitch = await page.evaluate(() => {
      const stored = localStorage.getItem("edgequake-query-ui");
      return stored ? JSON.parse(stored)?.state?.activeConversationId : null;
    });
    expect(beforeSwitch).toBe(FAKE_CONVERSATION_ID);

    const workspaceTrigger = page
      .getByRole("button", { name: /workspace/i })
      .first();
    if (await workspaceTrigger.isVisible()) {
      await workspaceTrigger.click();
      const altWorkspace = page.getByRole("menuitem").nth(1);
      if (await altWorkspace.isVisible()) {
        await altWorkspace.click();
        await page.waitForTimeout(500);
      }
    }

    const afterSwitch = await page.evaluate(() => {
      const stored = localStorage.getItem("edgequake-query-ui");
      return stored ? JSON.parse(stored)?.state?.activeConversationId ?? null : null;
    });

    expect(afterSwitch).toBeNull();
  });
});
