/**
 * SPEC-120 / cancel dual-SSOT: optimistic cancel intent → Stopping…
 */
import { describe, expect, it, beforeEach } from "vitest";
import {
  applyCancelIntentToDocument,
  clearCancelIntent,
  getCancelIntent,
  pinCancelIntent,
} from "../cancel-intent";
import { getDocumentDisplayStatus } from "@/components/documents/status-badge";

describe("cancel-intent optimistic Stopping", () => {
  beforeEach(() => {
    clearCancelIntent("insert-opt-1");
  });

  it("pinCancelIntent then applyCancelIntentToDocument sets ui_phase=stopping", () => {
    pinCancelIntent("insert-opt-1");
    const patched = applyCancelIntentToDocument({
      track_id: "insert-opt-1",
      status: "embedding",
      current_stage: "embedding",
      display_status: "embedding",
      ui_phase: "running",
    });
    expect(patched.ui_phase).toBe("stopping");
    expect(
      getDocumentDisplayStatus({
        status: patched.status,
        current_stage: patched.current_stage,
        display_status: patched.display_status,
        ui_phase: patched.ui_phase,
      }),
    ).toBe("stopping");
  });

  it("clears intent when document already terminal cancelled", () => {
    pinCancelIntent("insert-opt-1");
    expect(getCancelIntent("insert-opt-1")).toBeDefined();
    const patched = applyCancelIntentToDocument({
      track_id: "insert-opt-1",
      status: "cancelled",
      ui_phase: "terminal",
      display_status: "cancelled",
    });
    expect(patched.status).toBe("cancelled");
    expect(getCancelIntent("insert-opt-1")).toBeUndefined();
  });
});
