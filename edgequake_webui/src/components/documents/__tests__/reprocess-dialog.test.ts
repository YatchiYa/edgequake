/**
 * Unit tests for ReprocessDialog helper logic.
 *
 * WHY: The dialog's safety guards (PDF detection, in-flight blocking) are
 * pure functions that gate the reprocess mutation. Pinning their behavior
 * prevents regressions where a full re-conversion is offered for a text doc
 * or allowed while a task is already running.
 */
import { describe, expect, it } from "vitest";
import {
  isInflight,
  isInterruptedRestartDoc,
  isPdfDocument,
} from "../reprocess-dialog";

describe("isPdfDocument", () => {
  it("returns true when source_type is pdf", () => {
    expect(isPdfDocument({ source_type: "pdf" })).toBe(true);
  });

  it("returns true when document_type is pdf", () => {
    expect(isPdfDocument({ document_type: "pdf" })).toBe(true);
  });

  it("returns true when mime_type is application/pdf", () => {
    expect(isPdfDocument({ mime_type: "application/pdf" })).toBe(true);
  });

  it("returns false for text/markdown documents", () => {
    expect(isPdfDocument({ source_type: "text" })).toBe(false);
    expect(isPdfDocument({ source_type: "markdown" })).toBe(false);
    expect(isPdfDocument({ document_type: "text" })).toBe(false);
  });

  it("returns false for null/undefined", () => {
    expect(isPdfDocument(null)).toBe(false);
    expect(isPdfDocument(undefined)).toBe(false);
  });
});

describe("isInflight", () => {
  it("returns true for processing and pending", () => {
    expect(isInflight({ status: "processing" })).toBe(true);
    expect(isInflight({ status: "pending" })).toBe(true);
  });

  it("returns false for terminal and completed states", () => {
    expect(isInflight({ status: "completed" })).toBe(false);
    expect(isInflight({ status: "failed" })).toBe(false);
    expect(isInflight({ status: "cancelled" })).toBe(false);
  });

  it("returns false for null/undefined", () => {
    expect(isInflight(null)).toBe(false);
    expect(isInflight(undefined)).toBe(false);
  });
});

describe("isInterruptedRestartDoc", () => {
  it("returns true for structured failure_code", () => {
    expect(
      isInterruptedRestartDoc({
        status: "failed",
        failure_code: "server_restart_interrupted",
      }),
    ).toBe(true);
  });

  it("returns true for legacy Interrupted error_message", () => {
    expect(
      isInterruptedRestartDoc({
        status: "failed",
        error_message: "Interrupted by server restart — use Reprocess to resume",
      }),
    ).toBe(true);
  });

  it("returns false for ordinary extraction failures", () => {
    expect(
      isInterruptedRestartDoc({
        status: "failed",
        error_message: "Entity extraction timed out",
      }),
    ).toBe(false);
  });

  it("returns false for null/undefined", () => {
    expect(isInterruptedRestartDoc(null)).toBe(false);
    expect(isInterruptedRestartDoc(undefined)).toBe(false);
  });
});
