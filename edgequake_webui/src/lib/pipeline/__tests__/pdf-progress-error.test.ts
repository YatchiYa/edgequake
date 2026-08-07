import { describe, expect, it } from "vitest";
import { presentPdfProgressError } from "@/lib/pipeline/pdf-progress-error";

describe("presentPdfProgressError", () => {
  it("nested + task not found → muted progress tracking ended (not red Task ended)", () => {
    const view = presentPdfProgressError("Not found: Task not found: pdf-abc", {
      nested: true,
    });
    expect(view.kind).toBe("nested_ended");
    expect(view.message).toBe("Progress tracking ended");
    expect(view.message).not.toMatch(/Task ended/i);
  });

  it("standalone task not found → terminal Task ended copy", () => {
    const view = presentPdfProgressError("Task not found: pdf-xyz", {
      nested: false,
    });
    expect(view.kind).toBe("terminal");
    expect(view.message).toMatch(/Task ended — progress is no longer available/);
  });

  it("progress miss while polling → reconnecting", () => {
    const view = presentPdfProgressError("Progress not found", {
      isPolling: true,
    });
    expect(view.kind).toBe("reconnecting");
  });
});
