import { describe, expect, it } from "vitest";
import {
  buildIngestionRunView,
  formatRunHeadline,
  normalizeRunStage,
  parseCountsFromMessage,
  selectPrimaryRun,
  buildIngestionRunViews,
  stageStatusFor,
} from "@/lib/pipeline/ingestion-run-view";
import type { Document } from "@/types";

function doc(partial: Partial<Document> & { id: string }): Document {
  return {
    title: partial.title ?? partial.file_name ?? partial.id,
    chunk_count: 0,
    ...partial,
  } as Document;
}

describe("ingestion-run-view", () => {
  it("normalizes pending → queued and indexing → storing", () => {
    expect(normalizeRunStage("pending", "pending")).toBe("queued");
    expect(normalizeRunStage("indexing", "indexing")).toBe("storing");
  });

  it("parses chunk counts preferring chunk unit", () => {
    const c = parseCountsFromMessage("Extracting entities — chunk 42/351");
    expect(c).toEqual({ current: 42, total: 351, unit: "chunks" });
  });

  it("parses figure vision analyze counts", () => {
    const c = parseCountsFromMessage(
      "Analyzing figures with Vision LLM — figure 3/12",
    );
    expect(c).toEqual({ current: 3, total: 12, unit: "figures" });
  });

  it("builds run view for vision figure analyze during converting", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d-vision",
        file_name: "paper.pdf",
        status: "processing",
        current_stage: "converting",
        stage_message: "Analyzing figures with Vision LLM — figure 5/17",
        stage_progress: 0.99,
        source_type: "pdf",
        track_id: "t-vision",
      }),
    );
    expect(view?.stage).toBe("converting");
    expect(view?.counts).toEqual({ current: 5, total: 17, unit: "figures" });
    expect(formatRunHeadline(view!)).toContain("5/17");
  });

  it("builds run view for extracting document", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d1",
        file_name: "areal.pdf",
        status: "processing",
        current_stage: "extracting",
        stage_message: "chunk 10/100",
        stage_progress: 0.1,
        source_type: "pdf",
        track_id: "t1",
      }),
    );
    expect(view?.stage).toBe("extracting");
    expect(view?.counts?.current).toBe(10);
    expect(formatRunHeadline(view!)).toContain("10/100");
  });

  it("treats converting as active even when coarse status is still pending", () => {
    const view = buildIngestionRunView(
      doc({
        id: "d1",
        file_name: "fast_graph.pdf",
        status: "pending",
        current_stage: "converting",
        stage_message: "Converting PDF",
        stage_progress: 0.5,
        source_type: "pdf",
        track_id: "t1",
      }),
    );
    expect(view?.stage).toBe("converting");
    expect(view?.stageStatus).toBe("active");
    expect(stageStatusFor("converting", "pending")).toBe("active");
    expect(stageStatusFor("queued", "pending")).toBe("pending");
  });

  it("selectPrimaryRun prefers active over queued", () => {
    const map = buildIngestionRunViews([
      doc({
        id: "q1",
        status: "pending",
        current_stage: "queued",
        file_name: "q.md",
      }),
      doc({
        id: "a1",
        status: "processing",
        current_stage: "extracting",
        file_name: "a.md",
        stage_message: "working",
      }),
    ]);
    const primary = selectPrimaryRun(map);
    expect(primary?.documentId).toBe("a1");
  });
});
