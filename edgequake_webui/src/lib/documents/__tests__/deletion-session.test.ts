import { QueryClient } from "@tanstack/react-query";
import { afterEach, describe, expect, it } from "vitest";

import { admissionPhaseCopy } from "@/components/documents/admission-phase-row";
import type { Document } from "@/types";

import {
  applyDeletionCompleted,
  applyDeletionFailed,
  applyDeletionPhase,
  applyDeletionStarted,
  beginDeleteSession,
  clearDeleteSessionsForTests,
  dismissDeleteSession,
  formatDeleteCountsLabel,
  formatDeleteLivenessLabel,
  formatDeleteProgressHeader,
  formatDeleteSuccessDetail,
  getActiveDeletingDocumentIds,
  getDeleteSessions,
  isDeletingPinned,
  isHexShortDocumentLabel,
  preferDocumentName,
  patchDocumentsDeletingOptimistic,
  protectDeletingDocumentsInQueryData,
} from "../deletion-session";

afterEach(() => {
  clearDeleteSessionsForTests();
});

describe("deletion-session", () => {
  it("beginDeleteSession paints active row before network", () => {
    beginDeleteSession({
      documentId: "doc-1",
      documentName: "paper.pdf",
    });
    const sessions = getDeleteSessions();
    expect(sessions).toHaveLength(1);
    expect(sessions[0].documentName).toBe("paper.pdf");
    expect(sessions[0].status).toBe("active");
    expect(sessions[0].phaseLabel).toMatch(/Removing document data/i);
  });

  it("SPEC-069: beginDeleteSession does not downgrade filename to hex", () => {
    beginDeleteSession({
      documentId: "019f878a-bbbb-cccc-dddd-eeeeeeeeeeee",
      documentName: "report.pdf",
    });
    beginDeleteSession({
      documentId: "019f878a-bbbb-cccc-dddd-eeeeeeeeeeee",
      documentName: "019f878a",
    });
    expect(getDeleteSessions()[0].documentName).toBe("report.pdf");
    expect(preferDocumentName("report.pdf", "019f878a")).toBe("report.pdf");
    expect(isHexShortDocumentLabel("019f878a")).toBe(true);
  });

  it("SPEC-069: liveness label after silent graph phase", () => {
    beginDeleteSession({ documentId: "doc-1", documentName: "a.pdf" });
    applyDeletionPhase({
      documentId: "doc-1",
      phase: "removing_graph",
      phaseLabel: "Removing graph entities & edges",
      itemsProcessed: 0,
      itemsTotal: 0,
    });
    const entry = getDeleteSessions()[0];
    const now = entry.phaseUpdatedAt + 5000;
    expect(formatDeleteLivenessLabel(entry, now)).toMatch(/Still working/);
  });

  it("applies WS phases and counts", () => {
    beginDeleteSession({ documentId: "doc-1", documentName: "a.pdf" });
    applyDeletionStarted("doc-1");
    applyDeletionPhase({
      documentId: "doc-1",
      phase: "removing_graph",
      phaseLabel: "Removing graph entities & edges",
      itemsProcessed: 2,
      itemsTotal: 10,
    });
    const entry = getDeleteSessions()[0];
    expect(entry.phase).toBe("removing_graph");
    expect(entry.phaseLabel).toContain("graph");
    expect(formatDeleteCountsLabel(entry)).toBe("2/10");
  });

  it("completes with success detail", () => {
    beginDeleteSession({ documentId: "doc-1", documentName: "a.pdf" });
    applyDeletionCompleted({
      documentId: "doc-1",
      chunksDeleted: 3,
      entitiesRemoved: 5,
      relationshipsRemoved: 2,
      embeddingsDeleted: 3,
      partialFailure: false,
      error: null,
    });
    const entry = getDeleteSessions()[0];
    expect(entry.status).toBe("completed");
    expect(entry.phaseLabel).toMatch(/Removed/);
    expect(entry.phaseLabel).toMatch(/5 entities/);
  });

  it("keeps failed sessions until dismiss", () => {
    beginDeleteSession({ documentId: "doc-1", documentName: "a.pdf" });
    applyDeletionFailed("doc-1", "network error");
    expect(getDeleteSessions()[0].status).toBe("failed");
    dismissDeleteSession("doc-1");
    expect(getDeleteSessions()).toHaveLength(0);
  });

  it("patchDocumentsDeletingOptimistic sets deleting badge fields", () => {
    const queryClient = new QueryClient();
    queryClient.setQueryData(["documents", "ws"], {
      items: [
        {
          id: "doc-1",
          title: "a",
          status: "completed",
          entity_count: 99,
        } as Document,
      ],
    });
    patchDocumentsDeletingOptimistic(queryClient, "doc-1");
    const cached = queryClient.getQueryData<{ items: Document[] }>([
      "documents",
      "ws",
    ]);
    expect(cached?.items[0]?.status).toBe("deleting");
    expect(cached?.items[0]?.current_stage).toBe("deleting");
  });

  it("formatDeleteSuccessDetail handles empty stats", () => {
    expect(
      formatDeleteSuccessDetail({
        entitiesRemoved: 0,
        relationshipsRemoved: 0,
      }),
    ).toBe("Document removed");
  });

  it("SPEC-098: beginDeleteSession pins deleting against Completed poll", () => {
    beginDeleteSession({ documentId: "doc-1", documentName: "a.pdf" });
    expect(isDeletingPinned("doc-1")).toBe(true);
    expect(getActiveDeletingDocumentIds().has("doc-1")).toBe(true);
    const protectedData = protectDeletingDocumentsInQueryData({
      items: [
        {
          id: "doc-1",
          title: "a",
          status: "completed",
          current_stage: "completed",
        } as Document,
      ],
    });
    expect(protectedData.items?.[0]?.status).toBe("deleting");
    applyDeletionCompleted({
      documentId: "doc-1",
      chunksDeleted: 0,
      entitiesRemoved: 0,
      relationshipsRemoved: 0,
      embeddingsDeleted: 0,
      partialFailure: false,
      error: null,
    });
    expect(isDeletingPinned("doc-1")).toBe(false);
  });
});

describe("admissionPhaseCopy deleting", () => {
  const t = (_k: string, fallback: string) => fallback;

  it("uses deleting title and prefers live phase label", () => {
    const copy = admissionPhaseCopy(
      "deleting",
      t,
      "Removing graph entities & edges",
    );
    expect(copy.title).toBe("Deleting");
    expect(copy.detail).toContain("graph");
  });

  it("SPEC-098: completed session shows Deleted title", () => {
    const copy = admissionPhaseCopy(
      "deleting",
      t,
      "Document removed",
      "completed",
    );
    expect(copy.title).toBe("Deleted");
    expect(copy.detail).toBe("Document removed");
  });
});

describe("formatDeleteProgressHeader (SPEC-098 LAW-098-11)", () => {
  it("all failed → Delete failed (N), no pulse", () => {
    const h = formatDeleteProgressHeader([
      { status: "failed" },
      { status: "failed" },
    ]);
    expect(h.text).toBe("Delete failed (2)");
    expect(h.pulse).toBe(false);
  });

  it("mixed → Deleting A · failed B with pulse", () => {
    const h = formatDeleteProgressHeader([
      { status: "active" },
      { status: "failed" },
      { status: "active" },
    ]);
    expect(h.text).toBe("Deleting 2 · failed 1");
    expect(h.pulse).toBe(true);
  });

  it("all active → Deleting N with pulse", () => {
    const h = formatDeleteProgressHeader([
      { status: "active" },
      { status: "active" },
    ]);
    expect(h.text).toBe("Deleting 2 document(s)");
    expect(h.pulse).toBe(true);
  });
});
