import { describe, expect, it } from "vitest";

import {
  resolvePdfProgressTrackId,
  resolveProgressTrackId,
} from "../progress-track-id";

describe("resolveProgressTrackId", () => {
  it("prefers non-empty task_id over client track_id", () => {
    expect(
      resolveProgressTrackId({
        task_id: "pdf-abc",
        track_id: "upload_batch",
      }),
    ).toBe("pdf-abc");
  });

  it("prefers reprocess task_id over batch reprocess_* id", () => {
    expect(
      resolveProgressTrackId({
        task_id: "pdf_processing-11111111-2222-3333-4444-555555555555",
        track_id: "reprocess_20260716_120000_abcd1234",
      }),
    ).toBe("pdf_processing-11111111-2222-3333-4444-555555555555");
  });

  it("falls back to track_id when task_id is missing or blank", () => {
    expect(
      resolveProgressTrackId({
        task_id: "",
        track_id: "legacy",
      }),
    ).toBe("legacy");
    expect(
      resolveProgressTrackId({
        task_id: "   ",
        track_id: "legacy",
      }),
    ).toBe("legacy");
    expect(
      resolveProgressTrackId({
        track_id: "legacy",
      }),
    ).toBe("legacy");
  });

  it("returns undefined when neither id is usable", () => {
    expect(resolveProgressTrackId({})).toBeUndefined();
    expect(
      resolveProgressTrackId({ task_id: "", track_id: "" }),
    ).toBeUndefined();
  });
});

describe("resolvePdfProgressTrackId", () => {
  it("is an alias of resolveProgressTrackId", () => {
    expect(
      resolvePdfProgressTrackId({
        task_id: "pdf-abc",
        track_id: "upload_batch",
      }),
    ).toBe("pdf-abc");
  });
});
