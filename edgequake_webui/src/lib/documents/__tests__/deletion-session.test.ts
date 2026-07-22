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
  formatDeleteSuccessDetail,
  getDeleteSessions,
  isHexShortDocumentLabel,
  preferDocumentName,
  patchDocumentsDeletingOptimistic,
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
});
