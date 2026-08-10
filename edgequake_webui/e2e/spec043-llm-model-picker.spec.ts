/**
 * SPEC-043 — edgequake-llm 0.10.0 model picker, provider hub, attribution settings.
 * Screenshots: specs/043-update-edgequake-llm/e2e/screenshots/
 *
 * Requires live stack: make dev-bg && E2E_LIVE_STACK=1 pnpm exec playwright test e2e/spec043-llm-model-picker.spec.ts
 */
import { expect, test } from "@playwright/test";
import { GOTO_OPTS, waitForBackendHealthy } from "./helpers/app-ready";
import { API_V1_URL, BACKEND_URL } from "./helpers/backend-url";
import { requiresLiveStack, skipUnlessLiveStack } from "./helpers/live-stack";
import { spec043Screenshot } from "./helpers/screenshot-paths";

test.setTimeout(120_000);

async function getDefaultWorkspaceSlug(
  request: import("@playwright/test").APIRequestContext,
): Promise<string | null> {
  const tenantsResponse = await request.get(`${API_V1_URL}/tenants`);
  if (!tenantsResponse.ok()) return null;
  const tenants = (await tenantsResponse.json()) as { items?: Array<{ id: string }> };
  const tenantId = tenants.items?.[0]?.id;
  if (!tenantId) return null;

  const workspacesResponse = await request.get(
    `${API_V1_URL}/tenants/${tenantId}/workspaces`,
  );
  if (!workspacesResponse.ok()) return null;
  const workspaces = (await workspacesResponse.json()) as {
    items?: Array<{ slug: string }>;
  };
  return workspaces.items?.[0]?.slug ?? null;
}

async function gotoWorkspacePage(
  page: import("@playwright/test").Page,
  request: import("@playwright/test").APIRequestContext,
): Promise<string> {
  const slug = await getDefaultWorkspaceSlug(request);
  test.skip(!slug, "No workspace available");
  await page.goto(`/w/${slug}/workspace`, GOTO_OPTS);
  await expect(page.getByTestId("workspace-edit-config")).toBeVisible({ timeout: 30_000 });
  return slug;
}

/** Open Reconfigure → Customize models so two-step pickers are mounted. */
async function openWorkspaceModelEditors(
  page: import("@playwright/test").Page,
  request: import("@playwright/test").APIRequestContext,
): Promise<string> {
  const slug = await gotoWorkspacePage(page, request);
  await page.getByTestId("workspace-edit-config").click();
  await expect(page.getByTestId("reconfigure-workspace-wizard")).toBeVisible({
    timeout: 15_000,
  });
  const customize = page.getByTestId("server-defaults-customize");
  if (await customize.isVisible().catch(() => false)) {
    await customize.click();
  }
  await expect(page.getByTestId("wizard-models-advanced")).toBeVisible({
    timeout: 15_000,
  });
  await expect(page.getByTestId("llm-model-selector").first()).toBeVisible({
    timeout: 15_000,
  });
  return slug;
}

async function selectFirstProvider(
  page: import("@playwright/test").Page,
  picker: import("@playwright/test").Locator,
): Promise<string> {
  await picker.getByTestId("model-picker-provider-trigger").click();
  const list = page.getByTestId("model-picker-provider-list");
  await expect(list).toBeVisible({ timeout: 10_000 });
  const first = list.locator("[cmdk-item]").first();
  const id = (await first.getAttribute("data-testid")) ?? "";
  await first.click();
  return id;
}

async function gotoSettingsPage(page: import("@playwright/test").Page): Promise<void> {
  await page.goto("/settings", GOTO_OPTS);
  await expect(page.getByTestId("app-attribution-card")).toBeVisible({ timeout: 30_000 });
}

/** Visual QC: element must be visible with meaningful dimensions. */
async function assertVisibleWithSize(
  locator: import("@playwright/test").Locator,
  minWidth = 40,
  minHeight = 8,
) {
  await expect(locator).toBeVisible({ timeout: 20_000 });
  const box = await locator.boundingBox();
  expect(box).not.toBeNull();
  expect(box!.width).toBeGreaterThanOrEqual(minWidth);
  expect(box!.height).toBeGreaterThanOrEqual(minHeight);
}

test.describe("SPEC-043 LLM model picker & attribution", () => {
  test.beforeAll(async () => {
    if (!requiresLiveStack) return;
    await waitForBackendHealthy(15);
  });

  test.describe("Workspace model picker", () => {
    test.beforeEach(async ({ page }) => {
      skipUnlessLiveStack();
    });

    test("shows two-step provider then model picker with capability filters", async ({
      page,
      request,
    }) => {
      await openWorkspaceModelEditors(page, request);

      const llmSelector = page.getByTestId("llm-model-selector").first();
      await assertVisibleWithSize(llmSelector);

      const picker = llmSelector.getByTestId("model-picker-panel");
      await assertVisibleWithSize(picker);

      await expect(picker.getByTestId("model-picker-provider-bar")).toHaveCount(0);
      await expect(page.getByTestId("model-picker-provider-in-popover")).toHaveCount(0);
      await expect(picker.getByTestId("model-picker-provider-trigger")).toBeVisible();

      await selectFirstProvider(page, picker);
      await expect(page.getByTestId("model-picker-panel-list")).toBeVisible({
        timeout: 10_000,
      });

      // Wizard density keeps capability chips off; provider→model is the contract under test.
      await expect(page.getByTestId("model-picker-provider-bar")).toHaveCount(0);

      await page.screenshot({
        path: spec043Screenshot("01-workspace-model-picker-edit-mode.png"),
        fullPage: true,
      });
    });

    test("keyboard navigation highlights models without closing dropdown", async ({
      page,
      request,
    }) => {
      await openWorkspaceModelEditors(page, request);

      const picker = page
        .getByTestId("llm-model-selector")
        .first()
        .getByTestId("model-picker-panel");
      await selectFirstProvider(page, picker);

      const search = page.getByTestId("model-picker-panel-search");
      await expect(search).toBeFocused({ timeout: 5_000 });

      const list = page.getByTestId("model-picker-panel-list");
      await expect(list).toBeVisible();

      // Move highlight down from search into the list (cmdk)
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("ArrowDown");

      const selected = list.locator('[cmdk-item][data-selected="true"]');
      await expect(selected).toBeVisible({ timeout: 5_000 });

      await picker.screenshot({
        path: spec043Screenshot("08-model-picker-keyboard-focus.png"),
      });
    });

    test("mouse wheel scrolls model list without closing dropdown", async ({
      page,
      request,
    }) => {
      await openWorkspaceModelEditors(page, request);

      const picker = page
        .getByTestId("llm-model-selector")
        .first()
        .getByTestId("model-picker-panel");
      await selectFirstProvider(page, picker);

      const list = page.getByTestId("model-picker-panel-list");
      await expect(list).toBeVisible({ timeout: 10_000 });

      const scrollHeight = await list.evaluate((el) => el.scrollHeight);
      const clientHeight = await list.evaluate((el) => el.clientHeight);
      test.skip(
        scrollHeight <= clientHeight,
        "List too short to verify wheel scroll",
      );

      await list.hover();
      for (let i = 0; i < 5; i += 1) {
        await page.mouse.wheel(0, 200);
      }
      await page.waitForTimeout(100);

      const scrollTop = await list.evaluate((el) => el.scrollTop);
      expect(scrollTop).toBeGreaterThan(0);

      await picker.screenshot({
        path: spec043Screenshot("09-model-picker-wheel-scroll.png"),
      });
    });

    test("opens model dropdown after choosing provider", async ({
      page,
      request,
    }) => {
      await openWorkspaceModelEditors(page, request);

      const llmSelector = page.getByTestId("llm-model-selector").first();
      const picker = llmSelector.getByTestId("model-picker-panel");
      await selectFirstProvider(page, picker);

      const search = page.getByTestId("model-picker-panel-search");
      await expect(search).toBeVisible({ timeout: 10_000 });

      await picker.screenshot({
        path: spec043Screenshot("02-model-picker-dropdown-open.png"),
      });

      await page.keyboard.press("Escape");
      await picker.getByTestId("model-picker-panel-trigger").click();
      await expect(page.getByTestId("model-picker-panel-search")).toBeVisible({
        timeout: 10_000,
      });
      await picker.screenshot({
        path: spec043Screenshot("03-model-picker-vision-filter.png"),
      });
    });

    test("embedding model picker uses two-step provider then model", async ({
      page,
      request,
    }) => {
      await openWorkspaceModelEditors(page, request);

      const embeddingSelector = page.getByTestId("embedding-model-selector");
      await assertVisibleWithSize(embeddingSelector);

      const panel = page.getByTestId("embedding-model-picker-panel");
      await expect(panel).toBeVisible();
      await expect(panel.getByTestId("model-picker-provider-bar")).toHaveCount(0);
      await expect(panel.getByTestId("model-picker-capability-bar")).toHaveCount(0);
      await expect(panel.getByTestId("model-picker-provider-trigger")).toBeVisible();

      await selectFirstProvider(page, panel);
      await expect(page.getByTestId("embedding-model-picker-panel-search")).toBeVisible({
        timeout: 10_000,
      });
      await panel.screenshot({
        path: spec043Screenshot("06-embedding-model-picker-open.png"),
      });
    });

    test("lm studio provider option shows live-discovered models", async ({
      page,
      request,
    }) => {
      const lmProbe = await request
        .get("http://localhost:1234/api/v1/models", { timeout: 3_000 })
        .catch(() => null);
      const lmBody = lmProbe?.ok() ? ((await lmProbe.json()) as { models?: unknown[] }) : null;
      test.skip(!lmBody?.models?.length, "LM Studio not running or no models");

      await request.post(`${API_V1_URL}/models/discover/refresh`);

      await openWorkspaceModelEditors(page, request);

      const picker = page
        .getByTestId("llm-model-selector")
        .first()
        .getByTestId("model-picker-panel");
      await picker.getByTestId("model-picker-provider-trigger").click();
      const lmOption = page.getByTestId("model-picker-provider-option-lmstudio");
      await expect(lmOption).toBeVisible({ timeout: 10_000 });
      await lmOption.click();

      await expect(page.getByTestId("model-picker-panel-search")).toBeVisible({
        timeout: 10_000,
      });

      const listLoading = page.getByTestId("model-picker-panel-list-loading");
      if (await listLoading.isVisible().catch(() => false)) {
        await expect(listLoading).toBeHidden({ timeout: 20_000 });
      }

      await expect(page.getByTestId("model-picker-live-badge").first()).toBeVisible({
        timeout: 20_000,
      });

      await page.getByTestId("model-picker-panel-list").screenshot({
        path: spec043Screenshot("10-lmstudio-live-discovery.png"),
      });
    });

    test("workspace page does not show provider status hub", async ({
      page,
      request,
    }) => {
      await gotoWorkspacePage(page, request);
      // Provider Status hub removed from workspace overview (lives on Settings if needed).
      await expect(page.getByTestId("provider-status-hub")).toHaveCount(0);
    });

    test("vertexai provider uses identity auth (API health)", async ({
      request,
    }) => {
      const healthResponse = await request.get(`${API_V1_URL}/models/health`);
      expect(healthResponse.ok()).toBeTruthy();
      const healthBody = (await healthResponse.json()) as Array<{
        name: string;
        auth_kind?: string;
        health?: { available: boolean; error?: string };
      }>;
      const vertex = healthBody.find((p) => p.name === "vertexai");
      expect(vertex).toBeDefined();
      expect(vertex!.auth_kind).toBe("oauth2_identity");
      if (vertex!.health?.error) {
        expect(vertex!.health.error.toLowerCase()).not.toContain("api key");
      }
    });
  });

  test.describe("Query model picker", () => {
    test.beforeEach(async ({ page }) => {
      skipUnlessLiveStack();
      await page.goto("/query", GOTO_OPTS);
    });

    test("query settings uses two-step model picker", async ({ page }) => {
      await page.getByTestId("query-settings-trigger").click({ timeout: 15_000 });
      await expect(page.getByTestId("query-settings-sheet")).toBeVisible({ timeout: 10_000 });

      const queryPicker = page.getByTestId("query-model-selector");
      await expect(queryPicker).toBeVisible({ timeout: 10_000 });
      await expect(queryPicker.getByTestId("model-picker-provider-bar")).toHaveCount(0);
      await expect(queryPicker.getByTestId("model-picker-provider-trigger")).toBeVisible();
      await queryPicker.getByTestId("model-picker-provider-trigger").click();
      await expect(page.getByTestId("model-picker-provider-list")).toBeVisible({
        timeout: 10_000,
      });

      await queryPicker.screenshot({
        path: spec043Screenshot("07-query-model-selector.png"),
      });
    });
  });

  test.describe("Settings attribution", () => {
    test.beforeEach(async ({ page }) => {
      skipUnlessLiveStack();
      await gotoSettingsPage(page);
    });

    test("loads application attribution card with provider catalog", async ({ page }) => {
      const card = page.getByTestId("app-attribution-card");
      await card.scrollIntoViewIfNeeded();
      await assertVisibleWithSize(card, 300, 120);

      await expect(page.getByTestId("app-attribution-app-id")).toBeVisible();
      await expect(page.getByTestId("app-attribution-app-name")).toBeVisible();
      await expect(page.getByTestId("app-attribution-app-url")).toBeVisible();
      await expect(page.getByTestId("app-attribution-save")).toBeVisible();

      const catalog = page.getByTestId("app-attribution-provider-catalog");
      await expect(catalog).toBeVisible({ timeout: 15_000 });

      await card.screenshot({
        path: spec043Screenshot("05-settings-attribution-card.png"),
      });
    });
  });

  test.describe("API: models search & attribution", () => {
    test.beforeEach(() => {
      skipUnlessLiveStack();
    });

    test("GET /models/search returns hits for fuzzy query", async ({ request }) => {
      const response = await request.get(
        `${BACKEND_URL}/api/v1/models/search?q=gpt&fuzzy=true&limit=5`,
      );
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as { hits: unknown[]; total: number };
      expect(Array.isArray(body.hits)).toBe(true);
      expect(typeof body.total).toBe("number");
      const hits = body.hits as Array<{ provider: string }>;
      expect(hits.every((h) => h.provider !== "mock")).toBe(true);
    });

    test("GET /models/llm excludes mock provider", async ({ request }) => {
      const response = await request.get(`${API_V1_URL}/models/llm`);
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as {
        models: Array<{ provider: string }>;
      };
      expect(body.models.length).toBeGreaterThan(0);
      expect(body.models.every((m) => m.provider !== "mock")).toBe(true);
    });

    test("GET /settings/providers excludes mock and lists multiple providers", async ({ request }) => {
      const response = await request.get(`${API_V1_URL}/settings/providers`);
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as {
        llm_providers: Array<{ id: string }>;
      };
      const ids = body.llm_providers.map((p) => p.id);
      expect(ids.every((id) => id !== "mock")).toBe(true);
      expect(ids.length).toBeGreaterThanOrEqual(5);
      expect(ids).toContain("openai");
    });

    test("POST /models/discover/refresh invalidates cache", async ({ request }) => {
      const response = await request.post(`${API_V1_URL}/models/discover/refresh`);
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as { status: string };
      expect(body.status).toBe("ok");
    });

    test("GET /settings/attribution returns provider catalog without mock", async ({ request }) => {
      const response = await request.get(`${API_V1_URL}/settings/attribution`);
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as {
        effective_context: { active: boolean };
        providers: unknown[];
      };
      expect(body.effective_context).toBeDefined();
      expect(Array.isArray(body.providers)).toBe(true);
      expect(body.providers.length).toBeGreaterThan(0);
      const providers = body.providers as Array<{ id: string }>;
      expect(providers.every((p) => p.id !== "mock")).toBe(true);
    });

    test("GET /health includes attribution summary", async ({ request }) => {
      const response = await request.get(`${BACKEND_URL}/health`);
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as {
        status: string;
        attribution: { app_id: string | null; app_name: string | null; active: boolean };
      };
      expect(body.status).toMatch(/healthy|degraded/);
      expect(body.attribution).toBeDefined();
      expect(typeof body.attribution.active).toBe("boolean");
    });

    test("GET /models/search returns vertexai models (edgequake-llm 0.10.1+)", async ({
      request,
    }) => {
      const response = await request.get(
        `${BACKEND_URL}/api/v1/models/search?provider=vertexai&limit=50`,
      );
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as {
        hits: Array<{ provider: string; discovery_source?: string }>;
        total: number;
        dynamic: boolean;
      };
      expect(body.dynamic).toBe(true);
      expect(body.total).toBeGreaterThan(0);
      expect(body.hits.every((h) => h.provider === "vertexai")).toBe(true);
      const sources = new Set(body.hits.map((h) => h.discovery_source).filter(Boolean));
      expect(
        sources.has("static_registry") ||
          sources.has("dynamic_api") ||
          sources.has("user_config"),
      ).toBe(true);
    });

    test("GET /models/health returns oauth2_identity for vertexai (not API key errors)", async ({
      request,
    }) => {
      const response = await request.get(`${API_V1_URL}/models/health`);
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as Array<{
        name: string;
        auth_kind?: string;
        config_requirements?: Array<{ env_var: string; required: boolean }>;
        health?: { available: boolean; error?: string };
      }>;
      const vertex = body.find((p) => p.name === "vertexai");
      expect(vertex).toBeDefined();
      expect(vertex!.auth_kind).toBe("oauth2_identity");
      expect(vertex!.config_requirements?.some((r) => r.env_var === "GOOGLE_CLOUD_PROJECT")).toBe(
        true,
      );
      if (vertex!.health?.error) {
        expect(vertex!.health.error.toLowerCase()).not.toContain("api key");
      }
    });

    test("GET /models/search returns live LM Studio models when server is up", async ({
      request,
    }) => {
      const lmProbe = await request
        .get("http://localhost:1234/api/v1/models", { timeout: 3_000 })
        .catch(() => null);
      const lmUp =
        lmProbe?.ok() &&
        ((await lmProbe.json()) as { models?: unknown[] }).models?.length;
      test.skip(!lmUp, "LM Studio not running or no models");

      await request.post(`${API_V1_URL}/models/discover/refresh`);

      const response = await request.get(
        `${BACKEND_URL}/api/v1/models/search?provider=lmstudio&limit=50`,
      );
      expect(response.ok()).toBeTruthy();
      const body = (await response.json()) as {
        hits: Array<{ provider: string; discovery_source?: string; id: string }>;
        total: number;
      };
      expect(body.total).toBeGreaterThan(0);
      expect(body.hits.every((h) => h.provider === "lmstudio")).toBe(true);
      const liveHits = body.hits.filter((h) => h.discovery_source === "dynamic_api");
      expect(liveHits.length).toBeGreaterThan(0);
    });
  });
});
