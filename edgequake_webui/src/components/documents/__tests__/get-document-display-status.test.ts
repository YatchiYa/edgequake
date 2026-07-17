/**
 * SPEC-057 P4: getDocumentDisplayStatus prefers API SSOT fields.
 */
import { describe, expect, it } from "vitest";
import { getDocumentDisplayStatus } from "../status-badge";

describe("getDocumentDisplayStatus (SPEC-057 P4)", () => {
  it("passthrough display_status when present", () => {
    expect(
      getDocumentDisplayStatus({
        status: "processing",
        current_stage: "chunking",
        display_status: "extracting",
      }),
    ).toBe("extracting");
  });

  it("shows stopping when ui_phase is stopping", () => {
    expect(
      getDocumentDisplayStatus({
        status: "processing",
        current_stage: "extracting",
        display_status: "extracting",
        ui_phase: "stopping",
      }),
    ).toBe("stopping");
  });

  it("prefers terminal display_status cancelled over stage", () => {
    expect(
      getDocumentDisplayStatus({
        status: "processing",
        current_stage: "extracting",
        display_status: "cancelled",
        ui_phase: "terminal",
      }),
    ).toBe("cancelled");
  });

  it("falls back to current_stage when API fields absent", () => {
    expect(
      getDocumentDisplayStatus({
        status: "processing",
        current_stage: "re_embedding",
      }),
    ).toBe("re_embedding");
  });
});
