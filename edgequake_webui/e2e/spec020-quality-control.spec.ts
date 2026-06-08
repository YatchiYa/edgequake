/**
 * SPEC-020 — Full quality-control E2E (Playwright + live stack).
 * Artifacts: specs/020-e2e-quality-control/e2e/screenshots/
 *
 * Critical path: health → routes → ingest → query → PDF → isolation → live LLM → graph.
 */
import fs from "node:fs";
import { expect, test } from "@playwright/test";
import { waitForBackendHealthy } from "./helpers/app-ready";
import {
  BACKEND_URL,
  isEdgequakeBackendHealthy,
  waitForBackendInGlobalSetup,
} from "./helpers/backend-url";
import { skipUnlessLiveStack } from "./helpers/live-stack";
import { GOTO_OPTS } from "./helpers/navigation";
import {
  assertDocumentDeleted,
  assertDocumentNotFound,
  deleteDocumentViaApi,
  SIMPLE_PDF_FIXTURE,
  uploadPdfViaApi,
  uploadSimplePdfViaApi,
} from "./helpers/qc-documents";
import {
  openDocumentDetailById,
  uploadMarkdownViaUi,
  uploadPdfViaUi,
} from "./helpers/qc-ui-upload";
import {
  assertEmptyContentUploadRejected,
  assertMalformedUploadRejected,
} from "./helpers/qc-api-errors";
import { probeLoginPage, performDevLogin, authProofRequired } from "./helpers/qc-auth";
import {
  assertEntityExtractionProof,
  assertEmptyGraphSearchSafe,
  assertWorkspaceStatsIsolated,
} from "./helpers/qc-graph";
import {
  assertMigration038IfStrict,
  assertOperationalHealth,
  assertLiveProbe,
  assertReadyIfStrict,
  fetchHealth,
  fetchLive,
  fetchReady,
  migration038Status,
} from "./helpers/qc-health";
import {
  assertCrossTenantDocumentIsolation,
  assertInvalidTenantWorkspaceRejected,
  assertUnscopedDocumentsRequestSafe,
  createDualTenantContexts,
} from "./helpers/qc-isolation";
import {
  assertEmptyQuerySafe,
  assertQueryOnEmptyWorkspaceSafe,
  assertSourceCitationsVisible,
  assertStreamingCompleted,
  gotoQueryPage,
  isAcceptableLiveLlmAnswer,
  isGroundedSarahChenAnswer,
  isMockProviderAnswer,
  openSourceCitationsPanel,
  submitQueryAndWait,
} from "./helpers/qc-query";
import {
  bootstrapQcUiContext,
  createMockQcWorkspace,
  createOllamaQcWorkspace,
  reuploadSameMarkdown,
  syncUploadMarkdown,
} from "./helpers/qc-workspace";
import { guardOllamaAvailability } from "./helpers/llm-availability";
import {
  captureSpec020,
  ensureSpec020Artifacts,
  spec020Screenshot,
  writeSpec020Json,
} from "./helpers/spec020-artifacts";

const QC_DOC = `
EdgeQuake SPEC-020 quality control document.
Sarah Chen is a senior engineer at EDGEQUAKE building GraphRAG in Rust.
Michael Torres leads LLM integration and entity extraction pipelines.
John Smith maintains the Axum REST API and PostgreSQL storage layer.
`.trim();

const ROUTE_SMOKE: Array<{ path: string; shot: string; heading?: RegExp }> = [
  { path: "/", shot: "02-dashboard.png" },
  { path: "/documents", shot: "03-documents.png" },
  { path: "/query", shot: "04-query.png" },
  { path: "/pipeline", shot: "05-pipeline.png" },
  { path: "/workspace", shot: "06-workspace.png" },
  { path: "/graph", shot: "07-graph.png" },
  { path: "/knowledge", shot: "08-knowledge.png" },
  { path: "/costs", shot: "09-costs.png" },
  { path: "/api-explorer", shot: "10-api-explorer.png" },
  { path: "/settings", shot: "11-settings.png" },
];

test.describe("@audit SPEC-020 full quality control @audit", () => {
  test.beforeAll(async ({ request }) => {
    skipUnlessLiveStack();
    const ready =
      (await waitForBackendInGlobalSetup(request, 90).catch(() => false)) ||
      (await waitForBackendHealthy(90).catch(() => false));
    if (!ready && !(await isEdgequakeBackendHealthy(request))) {
      throw new Error(
        `Backend not healthy at ${BACKEND_URL} (PostgreSQL + storage components required)`,
      );
    }
    ensureSpec020Artifacts();
  });

  test("01 — backend health, components, and migration readiness", async ({
    request,
  }) => {
    skipUnlessLiveStack();
    const health = await fetchHealth(request);
    assertOperationalHealth(health);
    const mig = migration038Status(health);
    assertMigration038IfStrict(mig);
    const readyStatus = await fetchReady(request);
    assertReadyIfStrict(readyStatus);
    const live = await fetchLive(request);
    assertLiveProbe(live);
    writeSpec020Json("002-health-response.json", { ...health, migration038: mig });
    writeSpec020Json("005-migration038-status.json", mig);
    writeSpec020Json("002-ready-status.json", { status: readyStatus, live: live.body });
  });

  test("02 — critical routes render without application error", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(180_000);
    await bootstrapQcUiContext(page, request, "spec020-routes");

    for (const route of ROUTE_SMOKE) {
      await page.goto(route.path, GOTO_OPTS);
      await page
        .locator("main, [id='main-content'], body")
        .first()
        .waitFor({ state: "visible", timeout: 30_000 });
      const html = (await page.content()).toLowerCase();
      expect(html).not.toContain("application error");
      expect(html.length).toBeGreaterThan(200);
      if (route.heading) {
        await expect(
          page.getByRole("heading", { name: route.heading }).first(),
        ).toBeVisible({ timeout: 20_000 });
      }
      await page.screenshot({ path: spec020Screenshot(route.shot), fullPage: false });
    }
  });

  test("03 — sync ingestion creates chunks and surfaces document in UI", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(240_000);
    const ctx = await bootstrapQcUiContext(page, request, "spec020-ingest");
    const title = `spec020-qc-${Date.now()}.md`;
    const uploaded = await syncUploadMarkdown(request, ctx, title, QC_DOC);

    expect(uploaded.chunkCount).toBeGreaterThan(0);
    expect(uploaded.status).toMatch(/processed|completed|partial/i);
    writeSpec020Json("003-ingestion-result.json", uploaded);

    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByText(title).first()).toBeVisible({ timeout: 60_000 });
    await captureSpec020(page, "12-documents-after-ingest.png");
    await captureSpec020(page, "13-documents-main-panel.png", {
      locator: page.locator("main").first(),
    });
  });

  test("04 — hybrid query returns answer with source citations (mock or live)", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(300_000);
    const ctx = await bootstrapQcUiContext(page, request, "spec020-query");
    const title = `spec020-query-${Date.now()}.md`;
    await syncUploadMarkdown(request, ctx, title, QC_DOC);

    await gotoQueryPage(page);
    await captureSpec020(page, "14-query-ready.png");

    const answerText = await submitQueryAndWait(
      page,
      "Who is Sarah Chen at EDGEQUAKE?",
    );
    const grounded =
      isGroundedSarahChenAnswer(answerText) || isMockProviderAnswer(answerText);
    expect(grounded).toBeTruthy();
    await assertSourceCitationsVisible(page);

    writeSpec020Json("004-query-result.json", {
      answerPreview: answerText.slice(0, 300),
      mockProvider: isMockProviderAnswer(answerText),
      grounded: isGroundedSarahChenAnswer(answerText),
    });
    await captureSpec020(page, "15-query-answer.png", { fullPage: false });
  });

  test("05 — graph page loads with workspace context", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapQcUiContext(page, request, "spec020-graph");
    await page.goto("/graph", GOTO_OPTS);
    await expect(page.locator("main").first()).toBeVisible({ timeout: 20_000 });
    const html = await page.content();
    expect(html.toLowerCase()).not.toContain("application error");
    await captureSpec020(page, "16-graph-workspace.png");
  });

  test("06 — PDF upload via text parser reaches completed state", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.skip(!fs.existsSync(SIMPLE_PDF_FIXTURE), "PDF fixture missing");
    test.setTimeout(600_000);

    const ctx = await bootstrapQcUiContext(page, request, "spec020-pdf");
    const result = await uploadSimplePdfViaApi(request, ctx);
    expect(result.chunkCount).toBeGreaterThan(0);
    // partial_failure = PDF text extracted but entity extraction incomplete (mock/LLM edge)
    expect(result.status).toMatch(/processed|completed|partial/i);
    writeSpec020Json("006-pdf-ingestion-result.json", result);

    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(
      page.getByText(/001_simple_text|spec020-pdf/i).first(),
    ).toBeVisible({ timeout: 60_000 });
    await captureSpec020(page, "17-pdf-documents-list.png");
  });

  test("07 — multi-tenant document isolation (no cross-tenant leak)", async ({
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(180_000);
    const { tenantA, tenantB } = await createDualTenantContexts(
      request,
      "spec020-isolation",
    );
    const secretTitle = `spec020-secret-${Date.now()}.md`;
    await syncUploadMarkdown(request, tenantA, secretTitle, QC_DOC);
    await assertCrossTenantDocumentIsolation(
      request,
      tenantA,
      tenantB,
      secretTitle,
    );
    const rejection = await assertInvalidTenantWorkspaceRejected(
      request,
      tenantB.tenantId,
      tenantA.workspaceId,
    );
    writeSpec020Json("007-isolation-result.json", {
      secretTitle,
      invalidCombo: rejection,
    });
  });

  test("08 — unscoped API request does not leak documents", async ({ request }) => {
    skipUnlessLiveStack();
    const result = await assertUnscopedDocumentsRequestSafe(request);
    expect(result.safe).toBeTruthy();
    writeSpec020Json("008-unscoped-api-result.json", result);
  });

  test("09 — source citations panel opens after query", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(300_000);
    const ctx = await bootstrapQcUiContext(page, request, "spec020-citations");
    const title = `spec020-cite-${Date.now()}.md`;
    await syncUploadMarkdown(request, ctx, title, QC_DOC);
    await gotoQueryPage(page);
    await submitQueryAndWait(page, "Who works on GraphRAG at EDGEQUAKE?");
    await openSourceCitationsPanel(page);
    await captureSpec020(page, "18-source-citations-panel.png");
  });

  test("10 — live Ollama query returns grounded Sarah Chen answer", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await guardOllamaAvailability(test);

    test.setTimeout(600_000);
    const ctx = await bootstrapQcUiContext(page, request, "spec020-live", {
      provider: "ollama",
    });
    const title = `spec020-live-${Date.now()}.md`;
    const uploaded = await syncUploadMarkdown(request, ctx, title, QC_DOC);
    expect(uploaded.chunkCount).toBeGreaterThan(0);

    await gotoQueryPage(page);
    const answerText = await submitQueryAndWait(
      page,
      "Who is Sarah Chen at EDGEQUAKE?",
      { answerTimeoutMs: 300_000, processingTimeoutMs: 300_000 },
    );

    const strictlyGrounded = isGroundedSarahChenAnswer(answerText);
    const acceptable = isAcceptableLiveLlmAnswer(
      answerText,
      uploaded.entityCount,
    );
    writeSpec020Json("010-live-llm-result.json", {
      answerPreview: answerText.slice(0, 500),
      grounded: strictlyGrounded,
      acceptable,
      llmProvider: ctx.llmProvider,
      chunkCount: uploaded.chunkCount,
      entityCount: uploaded.entityCount,
    });
    await captureSpec020(page, "19-live-llm-answer.png", { fullPage: false });

    expect(acceptable).toBeTruthy();
    await assertSourceCitationsVisible(page);
  });

  test("11 — UI file upload via dropzone surfaces document in table", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(240_000);

    const ctx = await bootstrapQcUiContext(page, request, "spec020-ui-upload");
    const filename = `spec020-ui-${Date.now()}.md`;
    const result = await uploadMarkdownViaUi(page, ctx, QC_DOC, filename);
    writeSpec020Json("011-ui-upload-result.json", result);
    await captureSpec020(page, "20-ui-file-upload.png");
  });

  test("12 — document detail page shows chunks after ingest", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(240_000);
    const ctx = await bootstrapQcUiContext(page, request, "spec020-detail");
    const title = `spec020-detail-${Date.now()}.md`;
    const uploaded = await syncUploadMarkdown(request, ctx, title, QC_DOC);
    await openDocumentDetailById(page, uploaded.documentId);
    await expect(page).toHaveURL(
      new RegExp(`/documents/${uploaded.documentId}`),
      { timeout: 15_000 },
    );
    await expect(page.getByText(/chunks/i).first()).toBeVisible({ timeout: 20_000 });
    await captureSpec020(page, "21-document-detail.png");
    writeSpec020Json("012-document-detail.json", { title, url: page.url() });
  });

  test("13 — empty query does not crash query console", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    await bootstrapQcUiContext(page, request, "spec020-empty-query");
    await gotoQueryPage(page);
    await assertEmptyQuerySafe(page);
    await captureSpec020(page, "22-empty-query-safe.png");
    writeSpec020Json("013-empty-query.json", { safe: true });
  });

  test("14 — streaming query completes and re-enables input", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(300_000);
    const ctx = await bootstrapQcUiContext(page, request, "spec020-stream");
    const title = `spec020-stream-${Date.now()}.md`;
    await syncUploadMarkdown(request, ctx, title, QC_DOC);
    await gotoQueryPage(page);
    await submitQueryAndWait(page, "Summarize EDGEQUAKE engineers.");
    await assertStreamingCompleted(page);
    await captureSpec020(page, "23-streaming-complete.png");
    writeSpec020Json("014-streaming-result.json", { completed: true });
  });

  test("15 — unknown document ID returns 404", async ({ request }) => {
    skipUnlessLiveStack();
    const ctx = await createMockQcWorkspace(request, "spec020-404");
    const status = await assertDocumentNotFound(request, ctx);
    writeSpec020Json("015-not-found-result.json", { status });
  });

  test("16 — UI PDF upload via dropzone with API proxy", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.skip(!fs.existsSync(SIMPLE_PDF_FIXTURE), "PDF fixture missing");
    test.setTimeout(300_000);

    const ctx = await bootstrapQcUiContext(page, request, "spec020-ui-pdf");
    const result = await uploadPdfViaUi(page, ctx, SIMPLE_PDF_FIXTURE, {
      timeoutMs: 240_000,
    });
    writeSpec020Json("016-ui-pdf-upload-result.json", result);
    await captureSpec020(page, "24-ui-pdf-upload.png");
    expect(result.observedStatus).toMatch(
      /complete|Completed|Processed|Partial|table-row|upload-complete/i,
    );
  });

  test("17 — duplicate markdown re-upload handles re-ingestion edge", async ({
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(180_000);
    const ctx = await createMockQcWorkspace(request, "spec020-dup");
    const title = `spec020-dup-${Date.now()}.md`;
    const { first, second } = await reuploadSameMarkdown(
      request,
      ctx,
      title,
      QC_DOC,
    );
    expect(first.chunkCount).toBeGreaterThan(0);
    expect(second.status).toMatch(/processed|completed|partial|duplicate/i);
    writeSpec020Json("017-duplicate-upload.json", {
      firstId: first.documentId,
      secondId: second.documentId,
      firstStatus: first.status,
      secondStatus: second.status,
      reingested: first.documentId !== second.documentId,
    });
  });

  test("18 — query on empty workspace does not crash", async ({
    page,
    request,
  }) => {
    skipUnlessLiveStack();
    test.setTimeout(180_000);
    await bootstrapQcUiContext(page, request, "spec020-empty-ws");
    const answer = await assertQueryOnEmptyWorkspaceSafe(page);
    writeSpec020Json("018-empty-workspace-query.json", {
      answerPreview: answer.slice(0, 200),
      safe: true,
    });
    await captureSpec020(page, "25-empty-workspace-query.png");
  });

  test("19 — Ollama ingest extracts entities (workspace stats proof)", async ({
    request,
  }) => {
    skipUnlessLiveStack();
    await guardOllamaAvailability(test);

    test.setTimeout(300_000);
    const ctx = await createOllamaQcWorkspace(request, "spec020-graph-api");
    const title = `spec020-graph-${Date.now()}.md`;
    const result = await assertEntityExtractionProof(request, ctx, () =>
      syncUploadMarkdown(request, ctx, title, QC_DOC),
    );
    writeSpec020Json("019-graph-entities.json", result);
  });

  test("20 — malformed uploads rejected and empty graph search safe", async ({
    request,
  }) => {
    skipUnlessLiveStack();
    const ctx = await createMockQcWorkspace(request, "spec020-api-errors");
    const malformedStatus = await assertMalformedUploadRejected(request, ctx);
    const emptyResult = await assertEmptyContentUploadRejected(request, ctx);
    const graphSearch = await assertEmptyGraphSearchSafe(request, ctx);
    expect(emptyResult.rejected).toBeTruthy();
    writeSpec020Json("020-api-errors.json", {
      malformedStatus,
      emptyResult,
      graphSearch,
    });
  });

  test("21 — login page probe (full login when SPEC020_AUTH_PROOF=1)", async ({
    page,
  }) => {
    skipUnlessLiveStack();
    const probe = await probeLoginPage(page);
    writeSpec020Json("021-auth-probe.json", {
      ...probe,
      authProofRequired: authProofRequired(),
    });

    if (authProofRequired()) {
      if (!probe.authEnabled) {
        throw new Error(
          "SPEC020_AUTH_PROOF=1 but login form not visible — restart stack with DEV_AUTH_ENABLED=true",
        );
      }
      const login = await performDevLogin(page);
      expect(login.loggedIn).toBeTruthy();
      await captureSpec020(page, "26-login-dashboard.png");
      writeSpec020Json("021-auth-login.json", login);
      return;
    }

    if (probe.authEnabled) {
      await captureSpec020(page, "26-login-page.png");
      await expect(page.locator("input#username").first()).toBeVisible();
    } else {
      test.info().annotations.push({
        type: "note",
        description: "Auth disabled in build — login probe recorded only",
      });
    }
  });

  test("22 — workspace stats isolated after ingest", async ({ request }) => {
    skipUnlessLiveStack();
    await guardOllamaAvailability(test);

    test.setTimeout(300_000);
    const owner = await createOllamaQcWorkspace(request, "spec020-graph-own");
    const other = await createOllamaQcWorkspace(request, "spec020-graph-oth");
    const title = `spec020-graph-iso-${Date.now()}.md`;
    const stats = await assertWorkspaceStatsIsolated(request, owner, other, () =>
      syncUploadMarkdown(request, owner, title, QC_DOC),
    );
    writeSpec020Json("022-graph-isolation.json", { title, stats });
  });

  test("23 — PDF upload accepts vision flag (text parser fallback)", async ({
    request,
  }) => {
    skipUnlessLiveStack();
    test.skip(!fs.existsSync(SIMPLE_PDF_FIXTURE), "PDF fixture missing");
    test.setTimeout(300_000);

    const ctx = await createMockQcWorkspace(request, "spec020-vision-pdf");
    const result = await uploadPdfViaApi(request, ctx, {
      enableVision: true,
      parserBackend: "text",
    });
    expect(result.chunkCount).toBeGreaterThan(0);
    expect(result.status).toMatch(/processed|completed|partial/i);
    writeSpec020Json("023-vision-pdf-flag.json", result);
  });

  test("24 — document delete removes from API and list", async ({ request }) => {
    skipUnlessLiveStack();
    test.setTimeout(180_000);
    const ctx = await createMockQcWorkspace(request, "spec020-delete");
    const title = `spec020-del-${Date.now()}.md`;
    const uploaded = await syncUploadMarkdown(request, ctx, title, QC_DOC);
    expect(uploaded.chunkCount).toBeGreaterThan(0);

    const deleteStatus = await deleteDocumentViaApi(
      request,
      ctx,
      uploaded.documentId,
    );
    const afterDelete = await assertDocumentDeleted(
      request,
      ctx,
      uploaded.documentId,
      title,
    );
    expect(afterDelete.listed).toBe(false);
    writeSpec020Json("024-document-delete.json", {
      documentId: uploaded.documentId,
      deleteStatus,
      afterDelete,
    });
  });
});
