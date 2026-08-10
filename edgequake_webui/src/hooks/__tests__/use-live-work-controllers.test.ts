/**
 * SPEC-099 — ordinary Failed must not open feedback zone (empty 208px gap).
 */
import { describe, expect, it } from "vitest";
import { deriveLiveWorkControllers } from "@/hooks/use-live-work-controllers";
import type { Document } from "@/types";

function doc(partial: Partial<Document> & { id: string }): Document {
  return {
    title: partial.title ?? partial.file_name ?? partial.id,
    file_name: partial.file_name ?? `${partial.id}.md`,
    chunk_count: 0,
    ...partial,
  } as Document;
}

describe("deriveLiveWorkControllers", () => {
  it("Failed-only: showActiveRuns and hasLiveWork are false", () => {
    const result = deriveLiveWorkControllers({
      documents: [
        doc({
          id: "doc-failed",
          file_name: "ts_rag_2608.06223v1.md",
          status: "failed",
          current_stage: "failed",
          error_message: "Pipeline processing failed: entity extraction timeout",
          stage_message: "Pipeline processing failed: entity extraction timeout",
        }),
      ],
      pipelineStatus: {
        is_busy: false,
        running_tasks: 0,
        processing_tasks: 0,
        pending_tasks: 0,
        queued_tasks: 0,
      } as never,
      uploadingFiles: [],
      reprocessEntries: [],
      deleteSessionCount: 0,
    });

    expect(result.allRuns.length).toBeGreaterThanOrEqual(1);
    expect(result.showActiveRuns).toBe(false);
    expect(result.hasLiveWork).toBe(false);
    expect(result.showUploadList).toBe(false);
  });

  it("orphan Failed attention still opens ActiveRuns", () => {
    const result = deriveLiveWorkControllers({
      documents: [
        doc({
          id: "doc-orphan",
          file_name: "orphan.md",
          status: "failed",
          current_stage: "failed",
          error_message:
            "Orphaned staging admission — please re-upload the document.",
          stage_message:
            "Upload interrupted during 'uploading'. Please re-upload the document.",
          track_id: "insert-dead",
          admission_staging: true,
        }),
      ],
      pipelineStatus: undefined,
      uploadingFiles: [],
      reprocessEntries: [],
      deleteSessionCount: 0,
    });

    expect(result.showActiveRuns).toBe(true);
    expect(result.hasLiveWork).toBe(true);
  });

  it("live extracting run opens ActiveRuns", () => {
    const result = deriveLiveWorkControllers({
      documents: [
        doc({
          id: "doc-live",
          file_name: "busy.pdf",
          status: "processing",
          current_stage: "extracting",
          stage_message: "Extracting entities",
          track_id: "track-live",
          source_type: "pdf",
        }),
      ],
      pipelineStatus: {
        is_busy: true,
        running_tasks: 1,
        processing_tasks: 1,
        pending_tasks: 0,
        queued_tasks: 0,
      } as never,
      uploadingFiles: [],
      reprocessEntries: [],
      deleteSessionCount: 0,
    });

    expect(result.showActiveRuns).toBe(true);
    expect(result.hasLiveWork).toBe(true);
  });
});
