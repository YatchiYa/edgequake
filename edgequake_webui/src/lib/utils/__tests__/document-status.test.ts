/**
 * Document status notice resolution — edge cases for vision fallback false failures.
 */

import { describe, expect, it } from "vitest";
import type { Document } from "@/types";
import {
  getEffectiveErrorMessage,
  getEffectiveWarningMessage,
  isInformationalNotice,
  isTerminalFailureDocument,
  resolveDocumentDisplayStatus,
  shouldShowDocumentError,
} from "../document-status";

const baseDoc = (overrides: Partial<Document> = {}): Document => ({
  id: "doc-1",
  ...overrides,
});

describe("isInformationalNotice", () => {
  it("detects vision fallback messages", () => {
    expect(
      isInformationalNotice("Vision unavailable. Falling back to EdgeParse."),
    ).toBe(true);
    expect(isInformationalNotice("Pipeline processing failed")).toBe(false);
  });
});

describe("getEffectiveErrorMessage", () => {
  it("returns undefined for processing doc with legacy fallback in error_message", () => {
    const doc = baseDoc({
      status: "processing",
      current_stage: "chunking",
      error_message: "Vision unavailable. Falling back to EdgeParse.",
    });
    expect(getEffectiveErrorMessage(doc)).toBeUndefined();
    expect(shouldShowDocumentError(doc)).toBe(false);
  });

  it("returns error for terminal failed status", () => {
    const doc = baseDoc({
      status: "failed",
      error_message: "Pipeline processing failed: timeout",
    });
    expect(getEffectiveErrorMessage(doc)).toBe(
      "Pipeline processing failed: timeout",
    );
  });

  it("clears stale fallback on completed documents", () => {
    const doc = baseDoc({
      status: "completed",
      error_message: "Vision unavailable. Falling back to EdgeParse.",
    });
    expect(getEffectiveErrorMessage(doc)).toBeUndefined();
  });
});

describe("getEffectiveWarningMessage", () => {
  it("moves legacy error_message to warning during processing", () => {
    const msg = "Vision unavailable. Falling back to EdgeParse.";
    const doc = baseDoc({
      status: "processing",
      current_stage: "chunking",
      error_message: msg,
    });
    expect(getEffectiveWarningMessage(doc)).toBe(msg);
  });

  it("prefers explicit warning_message", () => {
    const doc = baseDoc({
      status: "processing",
      warning_message: "Using EdgeParse",
      error_message: "other",
    });
    expect(getEffectiveWarningMessage(doc)).toBe("Using EdgeParse");
  });

  it("returns undefined for terminal failures", () => {
    const doc = baseDoc({
      status: "failed",
      error_message: "Real failure",
    });
    expect(getEffectiveWarningMessage(doc)).toBeUndefined();
  });
});

describe("resolveDocumentDisplayStatus", () => {
  it("does not show failed for processing doc with legacy error_message", () => {
    const doc = baseDoc({
      status: "processing",
      current_stage: "chunking",
      error_message: "Vision unavailable. Falling back to EdgeParse.",
    });
    expect(resolveDocumentDisplayStatus(doc)).toBe("chunking");
  });

  it("shows failed when status is failed with error", () => {
    const doc = baseDoc({
      status: "failed",
      error_message: "Entity extraction failed",
    });
    expect(resolveDocumentDisplayStatus(doc)).toBe("failed");
  });

  it("shows partial_failure for partial_failure status", () => {
    const doc = baseDoc({
      status: "partial_failure",
      error_message: "0 entities extracted",
    });
    expect(resolveDocumentDisplayStatus(doc)).toBe("partial_failure");
    expect(isTerminalFailureDocument(doc)).toBe(true);
  });
});
