/**
 * SPEC-099 — ActiveRuns panel admission (ordinary Failed must not open zone).
 */
import { describe, expect, it } from "vitest";
import {
  hasPanelVisibleActiveRuns,
  isOrphanFailedAttention,
  partitionActiveRuns,
} from "@/lib/pipeline/active-runs-partition";
import type { IngestionRunView } from "@/lib/pipeline/ingestion-run-view";

function run(
  partial: Partial<IngestionRunView> & { documentId: string },
): IngestionRunView {
  return {
    trackId: null,
    filename: `${partial.documentId}.md`,
    sourceType: "markdown",
    stage: "failed",
    stageStatus: "failed",
    message: "Pipeline processing failed: entity extraction timeout",
    ...partial,
  };
}

describe("active-runs-partition / hasPanelVisibleActiveRuns", () => {
  it("ordinary Failed is not panel-visible", () => {
    const ordinary = run({ documentId: "doc-failed" });
    expect(isOrphanFailedAttention(ordinary)).toBe(false);
    expect(partitionActiveRuns([ordinary])).toEqual({
      working: [],
      attention: [],
    });
    expect(hasPanelVisibleActiveRuns([ordinary])).toBe(false);
  });

  it("orphan Failed attention is panel-visible", () => {
    const orphan = run({
      documentId: "doc-orphan",
      message:
        "Prior interrupted upload — please re-upload the document.",
    });
    expect(isOrphanFailedAttention(orphan)).toBe(true);
    expect(hasPanelVisibleActiveRuns([orphan])).toBe(true);
  });

  it("active / pending / cancelled are panel-visible", () => {
    expect(
      hasPanelVisibleActiveRuns([
        run({
          documentId: "a",
          stage: "extracting",
          stageStatus: "active",
          message: "Extracting",
          trackId: "t-a",
        }),
      ]),
    ).toBe(true);
    expect(
      hasPanelVisibleActiveRuns([
        run({
          documentId: "p",
          stage: "queued",
          stageStatus: "pending",
          message: "Queued",
          trackId: "t-p",
        }),
      ]),
    ).toBe(true);
    expect(
      hasPanelVisibleActiveRuns([
        run({
          documentId: "c",
          stage: "cancelled",
          stageStatus: "cancelled",
          message: "Cancelled",
          trackId: "t-c",
        }),
      ]),
    ).toBe(true);
  });
});
