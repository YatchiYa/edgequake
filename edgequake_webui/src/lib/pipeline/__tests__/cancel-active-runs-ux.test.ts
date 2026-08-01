/**
 * Cancelled Active Runs UX — first-class cancel (never Failed).
 */
import { describe, expect, it } from "vitest";
import {
  canDismissCancelledRun,
  canDismissFailedRun,
} from "@/components/documents/ingestion-run-card";
import { partitionActiveRuns } from "@/components/documents/active-runs-panel";
import {
  CANCELLED_ACTIVE_RUN_TTL_MS,
  cancelledRetentionDeadlines,
  createCancelledObservationClock,
  filterWorkingRunsForRetention,
  workingSectionTitleForRuns,
} from "@/lib/pipeline/active-runs-retention";
import {
  buildIngestionRunView,
  buildIngestionRunViewFromProgress,
  resolveRunTerminal,
  stageStatusFor,
  type IngestionRunView,
} from "@/lib/pipeline/ingestion-run-view";
import { buildStageTimeline } from "@/lib/pipeline/stage-timeline";
import type { Document } from "@/types";
import type { IngestionProgress } from "@/types/ingestion";

function doc(partial: Partial<Document> & { id: string }): Document {
  return {
    title: partial.title ?? partial.file_name ?? partial.id,
    chunk_count: 0,
    ...partial,
  } as Document;
}

function run(partial: Partial<IngestionRunView> & { documentId: string }): IngestionRunView {
  return {
    trackId: partial.trackId ?? "t1",
    filename: partial.filename ?? "doc.md",
    sourceType: partial.sourceType ?? "markdown",
    stage: partial.stage ?? "extracting",
    stageStatus: partial.stageStatus ?? "active",
    message: partial.message ?? "",
    ...partial,
  };
}

function progress(status: string, stage: string): IngestionProgress {
  return {
    track_id: "insert-1",
    document_id: "doc-1",
    document_name: "notes.md",
    status,
    overall_progress: 40,
    progress: {
      current_stage: stage as never,
      completion_percentage: 40,
      latest_message: status === "cancelled" ? "Cancelled by user" : "Working",
      stages: [],
    },
    started_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:01:00Z",
  };
}

describe("resolveRunTerminal / stageStatusFor (cancel SSOT)", () => {
  it("maps stopping → stageStatus=stopping (not active/failed)", () => {
    expect(resolveRunTerminal("stopping")).toEqual({
      stage: "stopping",
      stageStatus: "stopping",
    });
    expect(stageStatusFor("stopping", "stopping")).toBe("stopping");
  });

  it("maps cancelled → stageStatus=cancelled (never failed)", () => {
    expect(resolveRunTerminal("cancelled")).toEqual({
      stage: "cancelled",
      stageStatus: "cancelled",
    });
    expect(stageStatusFor("cancelled", "cancelled")).toBe("cancelled");
    expect(stageStatusFor("cancelled", "cancelled")).not.toBe("failed");
  });
});

describe("buildIngestionRunView cancel path", () => {
  it("list-path: stopping keeps stageStatus=stopping", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d-stop",
        file_name: "stop.md",
        status: "pending",
        current_stage: "extracting",
        ui_phase: "stopping",
        stage_message: "Stopping…",
        track_id: "t-stop",
        source_type: "markdown",
      }),
    );
    expect(view?.stage).toBe("stopping");
    expect(view?.stageStatus).toBe("stopping");
    expect(view?.cancelledAtStage).toBe("extracting");
  });

  it("list-path: cancelled is cancelled not failed", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d-cancel",
        file_name: "stop.md",
        status: "cancelled",
        current_stage: "cancelled",
        display_status: "cancelled",
        stage_message: "Cancelled by user",
        track_id: "t-cancel",
        source_type: "markdown",
      }),
    );
    expect(view?.stage).toBe("cancelled");
    expect(view?.stageStatus).toBe("cancelled");
    expect(view?.stageStatus).not.toBe("failed");
  });

  it("list-path: cancelled_from_stage freezes honest progress", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d-freeze",
        file_name: "freeze.md",
        status: "cancelled",
        current_stage: "cancelled",
        display_status: "cancelled",
        cancelled_from_stage: "extracting",
        stage_message: "Cancelled by user",
        stage_progress: 0.99,
        track_id: "t-freeze",
        source_type: "markdown",
      }),
    );
    expect(view?.cancelledAtStage).toBe("extracting");
    expect(view?.progress01).toBeUndefined();
    const tl = buildStageTimeline(view!);
    expect(tl.steps.find((s) => s.id === "extracting")?.status).toBe(
      "cancelled",
    );
    expect(tl.steps.find((s) => s.id === "embedding")?.status).toBe("pending");
    expect(tl.overallProgress01).toBeLessThan(0.99);
  });

  it("progress-path matches list-path for cancelled", () => {
    const fromProgress = buildIngestionRunViewFromProgress(
      progress("cancelled", "storing"),
      { sourceType: "markdown", filename: "notes.md" },
    );
    expect(fromProgress.stage).toBe("cancelled");
    expect(fromProgress.stageStatus).toBe("cancelled");
    expect(fromProgress.cancelledAtStage).toBe("storing");

    const fromList = buildIngestionRunView(
      doc({
        id: "doc-1",
        file_name: "notes.md",
        status: "cancelled",
        current_stage: "storing",
        display_status: "cancelled",
        source_type: "markdown",
      }),
    );
    expect(fromList?.stageStatus).toBe(fromProgress.stageStatus);
    expect(fromList?.stage).toBe(fromProgress.stage);
  });

  it("cancel while queued freezes at queued", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d-q",
        file_name: "q.md",
        status: "cancelled",
        current_stage: "queued",
        display_status: "cancelled",
        source_type: "markdown",
      }),
    );
    expect(view?.stageStatus).toBe("cancelled");
    expect(view?.cancelledAtStage).toBe("queued");
  });
});

describe("buildStageTimeline cancel path", () => {
  it("cancelled chip is Cancelled, never Failed", () => {
    const tl = buildStageTimeline(
      run({
        documentId: "d1",
        stage: "cancelled",
        stageStatus: "cancelled",
        cancelledAtStage: "storing",
      }),
    );
    const completed = tl.steps.find((s) => s.id === "completed");
    expect(completed?.label).toBe("Cancelled");
    expect(completed?.status).toBe("cancelled");
    expect(tl.steps.some((s) => s.label === "Failed")).toBe(false);
    expect(tl.steps.some((s) => s.status === "failed")).toBe(false);
  });

  it("freezes prior steps done at cancelledAtStage", () => {
    const tl = buildStageTimeline(
      run({
        documentId: "d1",
        stage: "cancelled",
        stageStatus: "cancelled",
        cancelledAtStage: "extracting",
        sourceType: "markdown",
      }),
    );
    expect(tl.steps.find((s) => s.id === "chunking")?.status).toBe("done");
    expect(tl.steps.find((s) => s.id === "extracting")?.status).toBe(
      "cancelled",
    );
    expect(tl.steps.find((s) => s.id === "embedding")?.status).toBe("pending");
  });

  it("stopping keeps last stage active without Failed chip", () => {
    const tl = buildStageTimeline(
      run({
        documentId: "d1",
        stage: "stopping",
        stageStatus: "stopping",
        cancelledAtStage: "merging",
      }),
    );
    expect(tl.steps.find((s) => s.id === "merging")?.status).toBe("active");
    expect(tl.steps.find((s) => s.id === "completed")?.status).toBe("pending");
    expect(tl.steps.some((s) => s.label === "Failed")).toBe(false);
  });
});

describe("active-runs retention", () => {
  it("keeps cancelled in Working during TTL and excludes after", () => {
    const clock = createCancelledObservationClock();
    const cancelled = run({
      documentId: "c1",
      stage: "cancelled",
      stageStatus: "cancelled",
      updatedAt: new Date(1_000_000).toISOString(),
    });
    const t0 = 1_000_000;
    const during = filterWorkingRunsForRetention([cancelled], {
      clock,
      dismissedCancelledIds: new Set(),
      nowMs: t0,
    });
    expect(during).toHaveLength(1);

    const after = filterWorkingRunsForRetention([cancelled], {
      clock,
      dismissedCancelledIds: new Set(),
      nowMs: t0 + CANCELLED_ACTIVE_RUN_TTL_MS + 1,
    });
    expect(after).toHaveLength(0);
  });

  it("uses durable updatedAt so refresh past TTL stays hidden", () => {
    const clock = createCancelledObservationClock();
    const cancelAt = Date.now() - CANCELLED_ACTIVE_RUN_TTL_MS - 5_000;
    const cancelled = run({
      documentId: "c-old",
      stage: "cancelled",
      stageStatus: "cancelled",
      updatedAt: new Date(cancelAt).toISOString(),
    });
    // Fresh clock (simulates page reload) — still excluded via updatedAt.
    const filtered = filterWorkingRunsForRetention([cancelled], {
      clock,
      dismissedCancelledIds: new Set(),
      nowMs: Date.now(),
    });
    expect(filtered).toHaveLength(0);
  });

  it("schedules no TTL deadline once already expired (avoids sync bump loop)", () => {
    const clock = createCancelledObservationClock();
    const t0 = 1_000_000;
    const cancelled = run({
      documentId: "c-expired",
      stage: "cancelled",
      stageStatus: "cancelled",
      updatedAt: new Date(t0).toISOString(),
    });
    const deadlines = cancelledRetentionDeadlines([cancelled], {
      clock,
      dismissedCancelledIds: new Set(),
      nowMs: t0 + CANCELLED_ACTIVE_RUN_TTL_MS + 1,
    });
    expect(deadlines).toEqual([]);

    const live = cancelledRetentionDeadlines([cancelled], {
      clock,
      dismissedCancelledIds: new Set(),
      nowMs: t0 + 1_000,
    });
    expect(live).toEqual([
      { id: "c-expired", deadline: t0 + CANCELLED_ACTIVE_RUN_TTL_MS },
    ]);
  });

  it("dismiss excludes cancelled immediately", () => {
    const clock = createCancelledObservationClock();
    const cancelled = run({
      documentId: "c1",
      stage: "cancelled",
      stageStatus: "cancelled",
    });
    const filtered = filterWorkingRunsForRetention([cancelled], {
      clock,
      dismissedCancelledIds: new Set(["c1"]),
      nowMs: 1_000_000,
    });
    expect(filtered).toHaveLength(0);
  });

  it("orphan failed still partitions to attention", () => {
    const orphan = run({
      documentId: "orphan",
      stage: "failed",
      stageStatus: "failed",
      message: "Prior interrupted upload — please re-upload the document.",
    });
    const cancelled = run({
      documentId: "c1",
      stage: "cancelled",
      stageStatus: "cancelled",
    });
    const { working, attention } = partitionActiveRuns([orphan, cancelled]);
    expect(attention.map((r) => r.documentId)).toEqual(["orphan"]);
    expect(working.map((r) => r.documentId)).toEqual(["c1"]);
  });

  it("section title prefers Recently cancelled when only cancelled dwell", () => {
    expect(
      workingSectionTitleForRuns([
        run({
          documentId: "c1",
          stage: "cancelled",
          stageStatus: "cancelled",
        }),
      ]),
    ).toBe("Recently cancelled");
  });
});

describe("IngestionRunCard dismiss gates", () => {
  it("allows dismiss for cancelled; not for stopping", () => {
    expect(
      canDismissCancelledRun(
        { stage: "cancelled", stageStatus: "cancelled" },
        true,
      ),
    ).toBe(true);
    expect(
      canDismissCancelledRun(
        { stage: "stopping", stageStatus: "stopping" },
        true,
      ),
    ).toBe(false);
    expect(
      canDismissFailedRun(
        { stage: "cancelled", stageStatus: "cancelled" },
        true,
      ),
    ).toBe(false);
  });
});
