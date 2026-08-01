import { describe, expect, it } from "vitest";

import type { Document } from "@/types";

import { mergeMonotonicListDocuments } from "../merge-monotonic-list";

function doc(overrides: Partial<Document> = {}): Document {
  return {
    id: "doc-1",
    title: "vision.pdf",
    status: "pending",
    current_stage: "queued",
    display_status: "queued",
    track_id: "track-a",
    source_type: "pdf",
    ...overrides,
  } as Document;
}

describe("mergeMonotonicListDocuments", () => {
  it("SPEC-098: keeps cached deleting over stale completed poll", () => {
    const previous = [
      {
        id: "d1",
        status: "deleting",
        current_stage: "deleting",
      } as Document,
    ];
    const polled = [
      {
        id: "d1",
        status: "completed",
        current_stage: "completed",
      } as Document,
    ];
    const merged = mergeMonotonicListDocuments(polled, previous);
    expect(merged[0].status).toBe("deleting");
  });

  it("keeps WS converting ahead of a stale queued poll (same track)", () => {
    const previous = [
      doc({
        status: "processing",
        current_stage: "converting",
        display_status: "converting",
        ui_phase: "running",
        stage_message: "Converting page 7/17 (ocr)",
      }),
    ];
    const polled = [doc({ stage_message: "Waiting for a processing slot" })];
    const merged = mergeMonotonicListDocuments(polled, previous);
    expect(merged[0].current_stage).toBe("converting");
    expect(merged[0].display_status).toBe("converting");
    expect(merged[0].stage_message).toMatch(/Converting/i);
  });

  it("accepts a new track_id as a wholesale run replacement", () => {
    const previous = [
      doc({
        status: "processing",
        current_stage: "converting",
        display_status: "converting",
        track_id: "track-old",
      }),
    ];
    const polled = [
      doc({
        track_id: "track-new",
        stage_message: "Waiting for reprocess worker",
      }),
    ];
    const merged = mergeMonotonicListDocuments(polled, previous);
    expect(merged[0].track_id).toBe("track-new");
    expect(merged[0].current_stage).toBe("queued");
  });

  it("lets terminal poll win over in-flight cache", () => {
    const previous = [
      doc({
        status: "processing",
        current_stage: "converting",
        display_status: "converting",
      }),
    ];
    const polled = [
      doc({
        status: "completed",
        current_stage: "completed",
        display_status: "completed",
        ui_phase: "terminal",
      }),
    ];
    const merged = mergeMonotonicListDocuments(polled, previous);
    expect(merged[0].status).toBe("completed");
  });
});
