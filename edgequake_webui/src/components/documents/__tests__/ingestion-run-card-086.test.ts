/**
 * SPEC-086 — IngestionRunCard nesting rules + dismiss failed gate (no RTL).
 */
import { afterEach, describe, expect, it } from "vitest";
import {
  isOrphanFailedAttention,
  partitionActiveRuns,
  workingSectionTitle,
} from "@/components/documents/active-runs-panel";
import {
  canDismissFailedRun,
  canDismissTerminalRun,
  shouldNestPdfPageMeter,
} from "@/components/documents/ingestion-run-card";
import {
  CANCELLED_RUN_GRACE_MS,
  cancelledRunKey,
  dismissCancelledRun,
  resetCancelledRunLifecycleForTests,
} from "@/lib/pipeline/cancelled-run-lifecycle";
import type { IngestionRunView } from "@/lib/pipeline/ingestion-run-view";

describe("IngestionRunCard PDF nest gate (LAW-IS2 / ux086_v_one_presenter)", () => {
  it("shows nest only for pdf + converting without structured counts", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
      }),
    ).toBe(true);
  });

  it("hides nest when progress_counts pages already on run (no dual bar)", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
        counts: { current: 4, total: 9, unit: "pages" },
      }),
    ).toBe(false);
  });

  it("hides nest for markdown even if converting stage leaked", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "markdown",
        stage: "converting",
      }),
    ).toBe(false);
  });

  it("hides nest for pdf after converting (chunking)", () => {
    expect(
      shouldNestPdfPageMeter({ sourceType: "pdf", stage: "chunking" }),
    ).toBe(false);
  });
});

describe("IngestionRunCard dismiss failed (ux086 orphan staging)", () => {
  it("allows dismiss only for failed runs with handler", () => {
    expect(
      canDismissFailedRun({ stage: "failed", stageStatus: "failed" }, true),
    ).toBe(true);
    expect(
      canDismissFailedRun({ stage: "failed", stageStatus: "failed" }, false),
    ).toBe(false);
    expect(
      canDismissFailedRun({ stage: "extracting", stageStatus: "active" }, true),
    ).toBe(false);
  });

  it("allows dismiss for cancelled acknowledgement", () => {
    expect(
      canDismissTerminalRun(
        { stage: "cancelled", stageStatus: "cancelled" },
        true,
      ),
    ).toBe(true);
  });

  it("classifies orphan re-upload failures for ActiveRuns dismiss", () => {
    const orphan: IngestionRunView = {
      documentId: "doc-1",
      trackId: "insert-dead",
      filename: "invarian.md",
      sourceType: "markdown",
      stage: "failed",
      stageStatus: "failed",
      message:
        "Prior interrupted upload — Upload interrupted during 'uploading'. Please re-upload the document.",
    };
    expect(isOrphanFailedAttention(orphan)).toBe(true);
    expect(canDismissFailedRun(orphan, true)).toBe(true);
  });
});

describe("ActiveRunsPanel partition (dual-run UX)", () => {
  afterEach(() => {
    resetCancelledRunLifecycleForTests();
  });

  const orphan: IngestionRunView = {
    documentId: "doc-orphan",
    trackId: "insert-dead",
    filename: "areal.md",
    sourceType: "markdown",
    stage: "failed",
    stageStatus: "failed",
    message: "Prior interrupted upload — please re-upload the document.",
  };
  const pdf: IngestionRunView = {
    documentId: "doc-pdf",
    trackId: "pdf-live",
    filename: "paper.pdf",
    sourceType: "pdf",
    stage: "converting",
    stageStatus: "active",
    message: "Converting PDF · 4/30 pages",
  };

  it("splits orphan failed shell from live PDF convert", () => {
    const { working, attention } = partitionActiveRuns([orphan, pdf]);
    expect(working.map((r) => r.documentId)).toEqual(["doc-pdf"]);
    expect(attention.map((r) => r.documentId)).toEqual(["doc-orphan"]);
  });

  it("keeps only attention when no live work", () => {
    const { working, attention } = partitionActiveRuns([orphan]);
    expect(working).toHaveLength(0);
    expect(attention).toHaveLength(1);
  });

  it("keeps cancelled within grace; drops after grace or dismiss", () => {
    const cancelled: IngestionRunView = {
      documentId: "doc-cancel",
      trackId: "insert-c",
      filename: "ticket.pdf",
      sourceType: "pdf",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Processing cancelled",
      updatedAt: new Date(1_000_000).toISOString(),
    };
    const t0 = 1_000_000;
    const mid = partitionActiveRuns([cancelled], t0);
    expect(mid.working).toHaveLength(1);
    expect(workingSectionTitle(mid.working)).toBe("Cancelled");

    const aged = partitionActiveRuns(
      [cancelled],
      t0 + CANCELLED_RUN_GRACE_MS + 1,
    );
    expect(aged.working).toHaveLength(0);

    resetCancelledRunLifecycleForTests();
    partitionActiveRuns([cancelled], t0);
    dismissCancelledRun(
      cancelledRunKey(cancelled.documentId, cancelled.trackId),
    );
    const dismissed = partitionActiveRuns([cancelled], t0 + 100);
    expect(dismissed.working).toHaveLength(0);
  });

  it("hides hours-old cancelled on first paint (no flash)", () => {
    const hoursAgo = new Date(Date.now() - 11 * 60 * 60 * 1000).toISOString();
    const cancelled: IngestionRunView = {
      documentId: "doc-stale",
      trackId: "insert-stale",
      filename: "01-databricks-ticket.pdf",
      sourceType: "pdf",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Processing cancelled",
      updatedAt: hoursAgo,
    };
    const { working } = partitionActiveRuns([cancelled]);
    expect(working).toHaveLength(0);
  });

  it("keeps stopping in working; title Stopping…", () => {
    const stopping: IngestionRunView = {
      documentId: "doc-stop",
      trackId: "t-stop",
      filename: "a.pdf",
      sourceType: "pdf",
      stage: "stopping",
      stageStatus: "active",
      message: "Cancellation requested…",
    };
    const { working } = partitionActiveRuns([stopping]);
    expect(working).toHaveLength(1);
    expect(workingSectionTitle(working)).toBe("Stopping…");
  });

  it("does not title cancelled-only as Queued run", () => {
    const cancelled: IngestionRunView = {
      documentId: "doc-c",
      trackId: "t",
      filename: "x.pdf",
      sourceType: "pdf",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Processing cancelled",
    };
    expect(workingSectionTitle([cancelled])).toBe("Cancelled");
  });
});
