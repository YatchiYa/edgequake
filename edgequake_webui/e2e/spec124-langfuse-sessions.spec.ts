import { expect, test } from "@playwright/test";
import {
  liveStackSkipReason,
  requiresLiveStack,
  skipUnlessLiveStack,
} from "./helpers/live-stack";

/**
 * SPEC-124: Langfuse Sessions — durable session id on exported spans.
 *
 * Uses /query `session_id` (explicit client session; LAW-124-11) for a
 * deterministic live probe. When LANGFUSE_* are in the Playwright env,
 * polls Langfuse observations v2 filtered by sessionId.
 */

const DEFAULT_TENANT = "00000000-0000-0000-0000-000000000002";
const DEFAULT_WORKSPACE = "00000000-0000-0000-0000-000000000003";
const DEFAULT_USER = "00000000-0000-0000-0000-000000000001";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

test.describe("SPEC-124 Langfuse sessions", () => {
  test.describe.configure({
    skip: !requiresLiveStack,
    reason: liveStackSkipReason,
  });

  test("two queries with same session_id surface in Langfuse when export active", async ({
    request,
  }) => {
    test.setTimeout(360_000);
    const statusRes = await request.get("/api/v1/settings/langfuse");
    expect(statusRes.ok()).toBeTruthy();
    const status = (await statusRes.json()) as {
      export_active?: boolean;
      base_url?: string;
      ui_url?: string;
      project_id?: string;
    };

    test.skip(
      !status.export_active,
      "Langfuse export_active=false — set LANGFUSE_* and restart backend",
    );

    const sessionId = crypto.randomUUID();
    const headers = {
      "Content-Type": "application/json",
      "X-Tenant-ID": DEFAULT_TENANT,
      "X-Workspace-ID": DEFAULT_WORKSPACE,
      "X-User-ID": DEFAULT_USER,
    };

    for (const turn of [1, 2]) {
      const res = await request.post("/api/v1/query", {
        headers,
        data: {
          query: `spec124 sessions turn ${turn}`,
          mode: "naive",
          session_id: sessionId,
        },
      });
      expect(res.ok(), await res.text()).toBeTruthy();
    }

    const pk = process.env.LANGFUSE_PUBLIC_KEY?.replace(/^["']|["']$/g, "");
    const sk = process.env.LANGFUSE_SECRET_KEY?.replace(/^["']|["']$/g, "");
    const base = String(status.base_url || status.ui_url || "")
      .replace(/^["']|["']$/g, "")
      .replace(/\/$/, "");

    test.skip(
      !pk || !sk || !base,
      "LANGFUSE_PUBLIC_KEY/SECRET_KEY not in Playwright env — spans exported; skip API poll",
    );

    if (base.includes("localhost")) {
      expect(base).not.toMatch(/cloud\.langfuse\.com/);
    }

    const auth = Buffer.from(`${pk}:${sk}`).toString("base64");
    const filter = JSON.stringify([
      {
        type: "string",
        column: "sessionId",
        operator: "=",
        value: sessionId,
      },
    ]);

    let found = false;
    for (let attempt = 0; attempt < 12; attempt++) {
      await new Promise((r) => setTimeout(r, 2500));
      const url = new URL(`${base!.replace(/\/$/, "")}/api/public/v2/observations`);
      url.searchParams.set("filter", filter);
      url.searchParams.set("limit", "20");
      const lf = await request.get(url.toString(), {
        headers: { Authorization: `Basic ${auth}` },
      });
      if (!lf.ok()) continue;
      const body = (await lf.json()) as { data?: unknown[] };
      const rows = body.data ?? [];
      if (rows.length >= 1) {
        found = true;
        break;
      }
      // Legacy sessions API (pre-v4) fallback
      const legacy = await request.get(
        `${base!.replace(/\/$/, "")}/api/public/sessions/${sessionId}`,
        { headers: { Authorization: `Basic ${auth}` } },
      );
      if (legacy.ok()) {
        found = true;
        break;
      }
    }

    expect(found).toBeTruthy();
  });
});
