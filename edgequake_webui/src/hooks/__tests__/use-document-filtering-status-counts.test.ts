/**
 * SPEC-057 P0: failedCount must not include cancelled documents.
 */
import { describe, expect, it } from "vitest";
import { countClientStatusCounts } from "../use-document-filtering";

describe("countClientStatusCounts", () => {
  it("counts failed and cancelled separately", () => {
    const counts = countClientStatusCounts([
      { status: "failed" },
      { status: "failed" },
      { status: "cancelled" },
      { status: "completed" },
      { status: "processing" },
    ]);
    expect(counts.failed).toBe(2);
    expect(counts.cancelled).toBe(1);
    expect(counts.completed).toBe(1);
    expect(counts.processing).toBe(1);
    expect(counts.all).toBe(5);
  });
});
