/**
 * SPEC-141 — List completeness: first page must not be “the list”.
 */
import { expect, test } from "@playwright/test";
import { bootstrapDeterministicUiContext } from "./helpers/bootstrap-ui";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { gotoApp } from "./helpers/navigation";
import { API_V1_URL } from "./helpers/backend-url";
import { tenantHeaders } from "./helpers/spec013-api";

test.describe("SPEC-141 list completeness", () => {
  test("knowledge grid shows the 51st injection", async ({ page, request }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      "spec141-inj",
    );
    const suffix = `${Date.now()}`;
    const names: string[] = [];
    for (let i = 0; i < 51; i += 1) {
      const name = `spec141-inj-${suffix}-${i.toString().padStart(2, "0")}`;
      names.push(name);
      const res = await request.put(
        `${API_V1_URL}/workspaces/${ctx.workspaceId}/injection`,
        {
          headers: tenantHeaders(ctx.tenantId, ctx.workspaceId),
          data: { name, content: `${name} glossary` },
        },
      );
      expect(res.ok() || res.status() === 202, await res.text()).toBeTruthy();
    }
    const last = names[50];

    await gotoApp(page, "/knowledge");
    await expect(page.getByTestId(`knowledge-injection-${last}`)).toBeVisible({
      timeout: 30_000,
    });
  });

  test("documents pager reaches a name only on page 2", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      "spec141-docs",
    );
    const oldest = `spec141-oldest-${Date.now()}`;
    const headers = tenantHeaders(ctx.tenantId, ctx.workspaceId);
    const first = await request.post(`${API_V1_URL}/documents`, {
      headers,
      data: {
        title: oldest,
        content: `${oldest} body`,
        async_processing: true,
      },
    });
    expect([200, 201, 202]).toContain(first.status());
    for (let i = 0; i < 20; i += 1) {
      const res = await request.post(`${API_V1_URL}/documents`, {
        headers,
        data: {
          title: `spec141-newer-${i.toString().padStart(2, "0")}`,
          content: `filler ${i}`,
          async_processing: true,
        },
      });
      expect([200, 201, 202]).toContain(res.status());
    }

    await gotoApp(page, "/documents");
    await expect(page.getByTestId("documents-inventory-section")).toBeVisible({
      timeout: 20_000,
    });

    const pager = page.getByTestId("documents-pagination");
    await expect(pager).toBeVisible({ timeout: 20_000 });
    await pager.getByRole("combobox").click();
    await page.getByRole("option", { name: "10", exact: true }).click();

    const next = page.getByTestId("documents-next-page");
    await expect(next).toBeEnabled({ timeout: 10_000 });
    await expect(page.getByText(oldest)).toHaveCount(0);

    await next.click();
    await next.click();
    await expect(page.getByText(oldest).first()).toBeVisible({ timeout: 15_000 });
  });

  test("chat history loads the 21st conversation", async ({ page, request }) => {
    skipUnlessLiveStack();
    const ctx = await bootstrapDeterministicUiContext(
      page,
      request,
      "spec141-hist",
    );
    const userId = await page.evaluate(() => localStorage.getItem("userId"));
    expect(userId).toBeTruthy();
    const suffix = `${Date.now()}`;
    const lastTitle = `SPEC141-CONV-20-${suffix}`;
    for (let i = 0; i < 21; i += 1) {
      const title =
        i === 20 ? lastTitle : `SPEC141-CONV-${i.toString().padStart(2, "0")}-${suffix}`;
      const res = await request.post(`${API_V1_URL}/conversations`, {
        headers: tenantHeaders(ctx.tenantId, ctx.workspaceId, {
          "X-User-ID": userId!,
        }),
        data: { title, mode: "hybrid" },
      });
      expect(res.ok(), await res.text()).toBeTruthy();
    }

    await gotoApp(page, "/query");
    const history = page.getByRole("complementary", { name: /^history$/i });
    await expect(history).toBeVisible({ timeout: 20_000 });
    const scroll = page.getByTestId("conversation-history-scroll");
    await expect(scroll).toBeVisible();
    for (let i = 0; i < 8; i += 1) {
      if (await page.getByText(lastTitle).count()) break;
      await scroll.evaluate((el) => {
        el.scrollTop = el.scrollHeight;
      });
      await page.waitForTimeout(400);
    }
    await expect(page.getByText(lastTitle)).toBeVisible({ timeout: 20_000 });
  });

  test("admin quotas list the 101st tenant", async ({ page, request }) => {
    skipUnlessLiveStack();
    await bootstrapDeterministicUiContext(page, request, "spec141-admin");
    const suffix = `${Date.now()}`;
    const lastName = `spec141-org-100-${suffix}`;
    for (let i = 0; i < 101; i += 1) {
      const name =
        i === 100 ? lastName : `spec141-org-${i.toString().padStart(3, "0")}-${suffix}`;
      const res = await request.post(`${API_V1_URL}/tenants`, {
        data: { name, plan: "pro" },
      });
      expect(res.ok(), await res.text()).toBeTruthy();
    }

    await gotoApp(page, "/settings");
    const section = page.getByTestId("spec100-admin-quota-section");
    if (!(await section.isVisible().catch(() => false))) {
      test.skip(true, "Admin quota section not visible (non-admin session)");
      return;
    }
    await expect(page.getByTestId(`admin-quota-tenant-${lastName}`)).toBeVisible({
      timeout: 30_000,
    });
  });
});
