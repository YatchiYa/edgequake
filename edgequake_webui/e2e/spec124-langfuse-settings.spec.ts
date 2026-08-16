import { expect, test } from "@playwright/test";
import { waitForAppReady } from "./helpers/app-ready";
import {
  liveStackSkipReason,
  requiresLiveStack,
  skipUnlessLiveStack,
} from "./helpers/live-stack";

/**
 * SPEC-124: Langfuse Settings card visibility and Open-link gating.
 */

test.beforeEach(() => {
  skipUnlessLiveStack();
});

test.describe("SPEC-124 Langfuse settings", () => {
  test.describe.configure({
    skip: !requiresLiveStack,
    reason: liveStackSkipReason,
  });

  test("settings page shows Langfuse card; Open link absent without export", async ({
    page,
    request,
  }) => {
    const status = await request.get("/api/v1/settings/langfuse");
    expect(status.ok()).toBeTruthy();
    const body = await status.json();
    expect(body).toHaveProperty("enabled");
    expect(body).toHaveProperty("ui_url");
    expect(body).toHaveProperty("config_requirements");
    expect(JSON.stringify(body)).not.toMatch(/sk-lf-[A-Za-z0-9]/);

    await waitForAppReady(page);
    await page.goto("/settings");
    await expect(page.getByTestId("langfuse-settings-card")).toBeVisible({
      timeout: 30_000,
    });

    if (body.export_active) {
      await expect(page.getByTestId("langfuse-open-link")).toBeVisible();
      const href = await page.getByTestId("langfuse-open-link").getAttribute("href");
      const expected = body.project_ui_url || body.ui_url;
      expect(href).toBe(expected);
      const host = String(body.ui_url).replace(/\/$/, "");
      expect(href?.startsWith(host)).toBeTruthy();
      if (host.includes("localhost")) {
        expect(href).toContain("/project/");
        expect(href).not.toMatch(/cloud\.langfuse\.com/);
      }
    } else {
      await expect(page.getByTestId("langfuse-open-link")).toHaveCount(0);
    }
  });
});
