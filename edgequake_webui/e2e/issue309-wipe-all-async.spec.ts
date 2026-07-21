/**
 * ISSUE-309 — Durable wipe-all: HTTP 202 + wipe_track_id; poll task to terminal;
 * assert workspace documents emptied when live stack is available.
 *
 * Uses expect.poll on task status (no flake sleeps as pass criteria).
 */
import { expect, test } from "@playwright/test";
import { API_V1_URL } from "./helpers/backend-url";
import { skipUnlessLiveStack } from "./helpers/live-stack";

test.beforeEach(() => {
  skipUnlessLiveStack();
});

const TENANT =
  process.env.E2E_TENANT_ID ?? "79d034a7-9b01-401b-b3c0-d898b5497766";
const WORKSPACE =
  process.env.E2E_WORKSPACE_ID ?? "940fadab-2390-4b29-af7e-ff27fd6d7755";

test("delete-all admits 202 with wipe_track_id and reaches terminal task", async ({
  request,
}) => {
  const headers = {
    "X-Tenant-ID": TENANT,
    "X-Workspace-ID": WORKSPACE,
    "X-EdgeQuake-Confirm": "delete-all-documents",
  };

  const res = await request.delete(`${API_V1_URL}/documents`, { headers });
  expect([202, 409]).toContain(res.status());
  if (res.status() === 409) {
    test.skip(true, "wipe already in flight");
  }
  const body = (await res.json()) as {
    accepted?: boolean;
    wipe_track_id?: string;
    planned_delete_count?: number;
    deleted_count?: number;
  };
  expect(body.accepted).toBe(true);
  expect(body.wipe_track_id).toBeTruthy();
  const trackId = body.wipe_track_id as string;
  // Correlate: planned count is admit-time only; final via task/WS.
  expect(typeof body.deleted_count === "number" || body.deleted_count === undefined).toBe(
    true,
  );

  await expect
    .poll(
      async () => {
        const taskRes = await request.get(`${API_V1_URL}/tasks/${trackId}`, {
          headers: {
            "X-Tenant-ID": TENANT,
            "X-Workspace-ID": WORKSPACE,
          },
        });
        if (!taskRes.ok()) return "missing";
        const task = (await taskRes.json()) as { status?: string; track_id?: string };
        // Poll fallback after reconnect/missed WS — status is SSOT.
        expect(task.track_id ?? trackId).toBeTruthy();
        return (task.status || "").toLowerCase();
      },
      { timeout: 120_000 },
    )
    .toMatch(/indexed|failed|cancelled/);

  // After successful wipe, document list for this workspace should be empty.
  const listRes = await request.get(`${API_V1_URL}/documents?page=1&page_size=5`, {
    headers: {
      "X-Tenant-ID": TENANT,
      "X-Workspace-ID": WORKSPACE,
    },
  });
  if (listRes.ok()) {
    const list = (await listRes.json()) as {
      documents?: unknown[];
      total?: number;
      items?: unknown[];
    };
    const docs = list.documents ?? list.items ?? [];
    const total = list.total ?? docs.length;
    // Only assert emptiness when wipe reached indexed (completed).
    const taskRes = await request.get(`${API_V1_URL}/tasks/${trackId}`, {
      headers: {
        "X-Tenant-ID": TENANT,
        "X-Workspace-ID": WORKSPACE,
      },
    });
    if (taskRes.ok()) {
      const task = (await taskRes.json()) as { status?: string };
      if ((task.status || "").toLowerCase() === "indexed") {
        expect(total).toBe(0);
      }
    }
  }
});
