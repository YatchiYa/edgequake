/**
 * SPEC-120 A6: a run must transition from Queued to Converting without
 * a stale poll reverting the row or Active Runs back to Queued.
 *
 * Strategy: first list poll is queued; subsequent polls are mid-convert.
 */

import { expect, test } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";

const MOCK_TENANT_ID = "tenant-bbbbbbbb-cccc-dddd-eeee-ffffffffffff";
const MOCK_WORKSPACE_ID = "ws-bbbbbbbb-cccc-dddd-eeee-ffffffffffff";
const MOCK_DOC_ID = "15f3095a-aaaa-bbbb-cccc-dddddddddddd";

const MOCK_TENANT = {
  id: MOCK_TENANT_ID,
  name: "ConvertTenant",
  slug: "convert-tenant",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

const MOCK_WORKSPACE = {
  id: MOCK_WORKSPACE_ID,
  name: "Convert Workspace",
  slug: "convert-workspace",
  tenant_id: MOCK_TENANT_ID,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

const CONVERTING_DOC = {
  id: MOCK_DOC_ID,
  title: "vision-paper.pdf",
  file_name: "vision-paper.pdf",
  status: "processing",
  current_stage: "converting",
  display_status: "converting",
  ui_phase: "running",
  stage_message: "Converting PDF (7/17 pages)",
  stage_progress: 0.41,
  track_id: "pdf-15f3095a-convert",
  source_type: "pdf",
  chunk_count: 0,
  entity_count: 0,
  created_at: "2026-07-27T04:00:00Z",
  updated_at: "2026-07-27T04:05:00Z",
};

const QUEUED_DOC = {
  ...CONVERTING_DOC,
  status: "pending",
  current_stage: "queued",
  display_status: "queued",
  ui_phase: "idle",
  stage_message: "Waiting for a processing slot",
  stage_progress: 0,
  updated_at: "2026-07-27T04:04:00Z",
};

const REPROCESS_QUEUED_DOC = {
  ...QUEUED_DOC,
  track_id: "pdf-15f3095a-reprocess",
  stage_message: "Waiting for reprocess worker",
  updated_at: "2026-07-27T04:06:00Z",
};

async function seedTenant(page: import("@playwright/test").Page) {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await page.evaluate(
    ({ tenant, workspace }) => {
      localStorage.clear();
      sessionStorage.clear();
      localStorage.setItem("userId", crypto.randomUUID());
      localStorage.setItem("tenantId", tenant.id);
      localStorage.setItem("workspaceId", workspace.id);
      localStorage.setItem(
        "edgequake-tenant",
        JSON.stringify({
          state: {
            selectedTenantId: tenant.id,
            selectedWorkspaceId: workspace.id,
            workspaces: [workspace],
            tenants: [tenant],
          },
          version: 0,
        }),
      );
    },
    { tenant: MOCK_TENANT, workspace: MOCK_WORKSPACE },
  );
}

async function mockApis(
  page: import("@playwright/test").Page,
  opts?: {
    documents?: Array<typeof QUEUED_DOC>;
    pipeline?: {
      pending_tasks?: number;
      processing_tasks?: number;
      held_or_fairness_held_tasks?: number;
      capacity_wait?: boolean;
      claimable_pending_tasks?: number;
    };
  },
) {
  let documents: Array<typeof QUEUED_DOC> = opts?.documents
    ? [...opts.documents]
    : [QUEUED_DOC];
  let pipelineOverride = opts?.pipeline;
  let documentPollCount = 0;
  await page.route("**/health", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        status: "healthy",
        version: "0.1.0-test",
        storage_mode: "postgresql",
      }),
    });
  });

  await page.route("**/ready", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ status: "ready" }),
    });
  });

  await page.route("**/live", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ status: "live" }),
    });
  });

  await page.route("**/api/v1/tenants", async (route) => {
    if (route.request().method() === "GET") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([MOCK_TENANT]),
      });
    } else {
      await route.fallback();
    }
  });

  await page.route("**/api/v1/tenants/*/workspaces**", async (route) => {
    if (route.request().method() === "GET") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([MOCK_WORKSPACE]),
      });
    } else {
      await route.fallback();
    }
  });

  await page.route("**/api/v1/documents**", async (route) => {
    const url = route.request().url();
    const method = route.request().method();
    if (method === "GET" && !url.includes("/documents/pdf")) {
      documentPollCount += 1;
      const pending = documents.filter(
        (d) => d.current_stage === "queued" || d.current_stage === "pending",
      ).length;
      const processing = documents.length - pending;
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          documents,
          total: documents.length,
          page: 1,
          page_size: 50,
          total_pages: 1,
          has_more: false,
          status_counts: {
            pending,
            processing,
            completed: 0,
            partial_failure: 0,
            failed: 0,
            cancelled: 0,
            unknown: 0,
          },
        }),
      });
    } else {
      await route.fallback();
    }
  });

  await page.route("**/api/v1/pipeline/status**", async (route) => {
    const first = documents[0];
    const processingDefault =
      first && first.current_stage === "queued" ? 0 : documents.some((d) => d.current_stage !== "queued" && d.current_stage !== "pending") ? 1 : 0;
    const pendingDefault = documents.filter(
      (d) => d.current_stage === "queued" || d.current_stage === "pending",
    ).length;
    const processing = pipelineOverride?.processing_tasks ?? processingDefault;
    const pending = pipelineOverride?.pending_tasks ?? pendingDefault;
    const held = pipelineOverride?.held_or_fairness_held_tasks ?? 0;
    const capacity =
      pipelineOverride?.capacity_wait ??
      (processing > 0 && held > 0);
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        is_busy: processing > 0,
        total_documents: documents.length,
        processed_documents: 0,
        current_batch: 0,
        total_batches: 0,
        history_messages: [],
        cancellation_requested: false,
        pending_tasks: pending,
        processing_tasks: processing,
        completed_tasks: 0,
        failed_tasks: 0,
        held_or_fairness_held_tasks: held,
        claimable_pending_tasks:
          pipelineOverride?.claimable_pending_tasks ?? Math.max(0, pending - held),
        capacity_wait: capacity,
      }),
    });
  });

  await page.route("**/api/v1/tasks**", async (route) => {
    const first = documents[0];
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        tasks: [],
        items: [],
        total: 0,
        statistics: {
          pending:
            pipelineOverride?.pending_tasks ??
            (first?.current_stage === "queued" ? 1 : 0),
          processing:
            pipelineOverride?.processing_tasks ??
            (first?.current_stage === "queued" ? 0 : 1),
          indexed: 0,
          failed: 0,
          cancelled: 0,
        },
      }),
    });
  });

  return {
    setDocument(next: typeof QUEUED_DOC) {
      documents = [next];
      pipelineOverride = undefined;
    },
    setDocuments(next: Array<typeof QUEUED_DOC>) {
      documents = [...next];
    },
    setPipeline(next: NonNullable<typeof opts>["pipeline"]) {
      pipelineOverride = next;
    },
    getDocumentPollCount() {
      return documentPollCount;
    },
  };
}

test.describe("SPEC-120 converting not queued", () => {
  test("WS converting survives stale poll and a new run replaces it", async ({
    page,
  }) => {
    // This scenario specifically verifies the poll transition path. The app
    // normally disables background polling under browser automation.
    await page.addInitScript(() => {
      const NativeWebSocket = window.WebSocket;
      class FakeWebSocket extends EventTarget {
        static readonly CONNECTING = 0;
        static readonly OPEN = 1;
        static readonly CLOSING = 2;
        static readonly CLOSED = 3;
        readonly url: string;
        readyState = FakeWebSocket.OPEN;
        onopen: ((event: Event) => void) | null = null;
        onmessage: ((event: MessageEvent) => void) | null = null;
        onclose: ((event: CloseEvent) => void) | null = null;

        constructor(url: string | URL) {
          super();
          this.url = String(url);
          if (!this.url.includes("/ws/pipeline/progress")) {
            return new NativeWebSocket(url);
          }
          (
            window as typeof window & {
              __emitSpec120Ws?: (payload: unknown) => void;
            }
          ).__emitSpec120Ws = (payload) => {
            const event = new MessageEvent("message", {
              data: JSON.stringify(payload),
            });
            this.dispatchEvent(event);
            this.onmessage?.(event);
          };
          queueMicrotask(() => {
            const event = new Event("open");
            this.dispatchEvent(event);
            this.onopen?.(event);
          });
        }

        send(
          _data: string | ArrayBufferLike | Blob | ArrayBufferView,
        ) {}
        close() {
          this.readyState = FakeWebSocket.CLOSED;
          const event = new CloseEvent("close");
          this.dispatchEvent(event);
          this.onclose?.(event);
        }
      }
      Object.defineProperty(window, "WebSocket", {
        configurable: true,
        value: FakeWebSocket,
      });
      Object.defineProperty(Navigator.prototype, "webdriver", {
        configurable: true,
        get: () => false,
      });
      Object.defineProperty(Navigator.prototype, "userAgent", {
        configurable: true,
        get: () =>
          "Mozilla/5.0 AppleWebKit/537.36 Chrome/130.0.0.0 Safari/537.36",
      });
      Object.defineProperty(window, "__PLAYWRIGHT__", {
        configurable: true,
        value: false,
      });
    });
    await seedTenant(page);
    const { setDocument, getDocumentPollCount } = await mockApis(page);
    await page.goto("/documents", GOTO_OPTS);

    const badge = page.getByTestId("status-badge").first();
    await expect(badge).toBeVisible({ timeout: 15000 });
    await expect(badge).toContainText(/Queued/i);

    const activeRuns = page.getByTestId("spec048-active-runs-panel");
    await expect(activeRuns).toBeVisible({ timeout: 15000 });
    await expect(activeRuns).toContainText(/Queued/i);

    await page.evaluate(
      ({ documentId, trackId }) => {
        (
          window as typeof window & {
            __emitSpec120Ws?: (payload: unknown) => void;
          }
        ).__emitSpec120Ws?.({
          type: "PdfPageProgress",
          data: {
            document_id: documentId,
            task_id: trackId,
            current_page: 7,
            total_pages: 17,
            progress: 0.41,
            phase: "ocr",
          },
        });
      },
      { documentId: MOCK_DOC_ID, trackId: CONVERTING_DOC.track_id },
    );
    await expect(badge).toContainText(/Converting/i);
    await expect(badge).not.toContainText(/Queued/i);

    await expect(activeRuns).toContainText(/Converting/i);
    await expect(activeRuns).toContainText(/Active run/i);
    await expect(activeRuns).not.toContainText("Queued — Queued");
    await expect(activeRuns).not.toContainText("Queued run");
    await expect(activeRuns).not.toContainText(/Waiting for a processing slot/i);

    // The API still returns its older queued projection for this run. Wait
    // through a polling interval and assert it cannot clobber the WS update.
    const pollCountAfterWs = getDocumentPollCount();
    await expect
      .poll(getDocumentPollCount, { timeout: 10_000 })
      .toBeGreaterThan(pollCountAfterWs);
    await expect(badge).toContainText(/Converting/i);
    await expect(activeRuns).toContainText(/Converting/i);

    // A different non-empty track is a new run and must replace all old-run
    // fields wholesale rather than creating a hybrid row.
    setDocument(REPROCESS_QUEUED_DOC);
    await expect(badge).toContainText(/Queued/i, { timeout: 5000 });
    await expect(activeRuns).toContainText(/Waiting for reprocess worker/i);
    await expect(activeRuns).not.toContainText(/7\/17/);
  });

  test("held capacity wait clears when task advances to converting", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      Object.defineProperty(Navigator.prototype, "webdriver", {
        configurable: true,
        get: () => false,
      });
      Object.defineProperty(window, "__PLAYWRIGHT__", {
        configurable: true,
        value: false,
      });
    });

    const HELD_DOC = {
      ...QUEUED_DOC,
      status: "pending",
      current_stage: "queued",
      display_status: "queued",
      ui_phase: "idle",
      stage_message: "Waiting for capacity",
      presentation: {
        badge: "Waiting for capacity",
        tone: "neutral",
        stop_affordance: "cancel",
        progress_mode: "none",
      },
      track_id: "pdf-15f3095a-held",
    };
    const RUNNING_DOC = {
      ...CONVERTING_DOC,
      track_id: "pdf-15f3095a-held",
      stage_message: "Converting PDF (7/17 pages)",
      presentation: {
        badge: "Running",
        tone: "info",
        stop_affordance: "stop",
        progress_mode: "determinate",
      },
    };

    await seedTenant(page);
    const { setDocument } = await mockApis(page);
    setDocument(HELD_DOC);
    await page.goto("/documents", GOTO_OPTS);

    const activeRuns = page.getByTestId("spec048-active-runs-panel");
    await expect(activeRuns).toBeVisible({ timeout: 15000 });
    await expect(activeRuns).toContainText(/Waiting for capacity/i);
    await expect(activeRuns).not.toContainText(/Waiting for a processing slot/i);

    setDocument(RUNNING_DOC);
    await expect(activeRuns).toContainText(/Converting/i, { timeout: 10000 });
    await expect(activeRuns).not.toContainText(/Waiting for capacity/i);
    await expect(activeRuns).not.toContainText(/Waiting for a processing slot/i);

    const badge = page.getByTestId("status-badge").first();
    await expect(badge).toContainText(/Converting/i);
    await expect(badge).not.toContainText(/Queued/i);
  });

  test("capacity wait banner must not say Workers are idle", async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(Navigator.prototype, "webdriver", {
        configurable: true,
        get: () => false,
      });
      Object.defineProperty(window, "__PLAYWRIGHT__", {
        configurable: true,
        value: false,
      });
    });

    const ACTIVE_DOC = {
      ...CONVERTING_DOC,
      id: "doc-active-capacity",
      track_id: "insert-active-capacity",
      title: "active.pdf",
      file_name: "active.pdf",
    };
    const WAITING_A = {
      ...QUEUED_DOC,
      id: "doc-wait-a",
      track_id: "insert-wait-a",
      title: "wait-a.pdf",
      file_name: "wait-a.pdf",
      stage_message: "Waiting for Ollama/gemma3 capacity (1 of 1)",
      presentation: {
        badge: "Waiting for Ollama/gemma3 capacity (1 of 1)",
        tone: "neutral",
        stop_affordance: "cancel",
        progress_mode: "none",
      },
    };
    const WAITING_B = {
      ...WAITING_A,
      id: "doc-wait-b",
      track_id: "insert-wait-b",
      title: "wait-b.pdf",
      file_name: "wait-b.pdf",
    };

    await seedTenant(page);
    await mockApis(page, {
      documents: [ACTIVE_DOC, WAITING_A, WAITING_B],
      pipeline: {
        pending_tasks: 2,
        processing_tasks: 1,
        held_or_fairness_held_tasks: 2,
        claimable_pending_tasks: 0,
        capacity_wait: true,
        capacity_wait_reason: "Waiting for Ollama/gemma3 capacity (1 of 1)",
      },
    });
    await page.goto("/documents", GOTO_OPTS);

    // Active-runs feedback zone demotes the amber banner; header shortcut is the
    // durable capacity-wait signal under demotePipelineBanner.
    const headerBtn = page.getByTestId("pipeline-header-button");
    await expect(headerBtn).toBeVisible({ timeout: 15000 });
    await expect(headerBtn).toContainText(/Waiting for a processing slot/i);
    await expect(page.locator("body")).not.toContainText(/Workers are idle/i);
    // Named provider capacity reason must surface; Gleaning must not be the active chip.
    await expect(page.locator("body")).toContainText(
      /Ollama|Waiting for capacity|tenant fair-share/i,
    );
    const gleaningActive = page.locator(
      '[data-testid="spec048-stage-gleaning"][data-state="active"]',
    );
    await expect(gleaningActive).toHaveCount(0);
  });

  test("terminal docs with capacity_wait must not show ghost document capacity", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      Object.defineProperty(Navigator.prototype, "webdriver", {
        configurable: true,
        get: () => false,
      });
      Object.defineProperty(window, "__PLAYWRIGHT__", {
        configurable: true,
        value: false,
      });
    });

    const TERMINAL_A = {
      ...CONVERTING_DOC,
      id: "doc-terminal-a",
      track_id: "insert-terminal-a",
      title: "done-a.pdf",
      file_name: "done-a.pdf",
      status: "completed",
      current_stage: "completed",
      display_status: "completed",
      ui_phase: "terminal",
      stage_message: "Processing complete",
      stage_progress: 1,
    };
    const TERMINAL_B = {
      ...TERMINAL_A,
      id: "doc-terminal-b",
      track_id: "insert-terminal-b",
      title: "done-b.pdf",
      file_name: "done-b.pdf",
      status: "indexed",
      current_stage: "completed",
      display_status: "indexed",
    };

    await seedTenant(page);
    await mockApis(page, {
      documents: [TERMINAL_A, TERMINAL_B],
      pipeline: {
        pending_tasks: 1,
        processing_tasks: 1,
        held_or_fairness_held_tasks: 1,
        claimable_pending_tasks: 0,
        capacity_wait: true,
      },
    });
    await page.goto("/documents", GOTO_OPTS);

    await expect(page.getByTestId("pipeline-header-button")).toHaveCount(0);
    await expect(page.getByTestId("ingestion-alert-capacity")).toHaveCount(0);
    await expect(page.locator("body")).not.toContainText(
      /document\(s\) waiting for a free processing slot/i,
    );
    await expect(page.locator("body")).not.toContainText(/Workers are idle/i);
  });
});
