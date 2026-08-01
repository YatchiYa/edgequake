/**
 * Durable dismiss / freeze cache for Cancelled Active Runs.
 */
import { afterEach, describe, expect, it } from "vitest";
import {
  clearCancelledActiveRunDismissStorage,
  loadCancelledFromStage,
  loadDismissedCancelledIds,
  persistDismissedCancelledId,
  pruneDismissedCancelledIds,
  rememberCancelledFromStage,
} from "@/lib/pipeline/cancelled-active-run-dismiss";

afterEach(() => {
  clearCancelledActiveRunDismissStorage();
});

describe("cancelled-active-run-dismiss", () => {
  it("persists dismiss across load (refresh simulation)", () => {
    persistDismissedCancelledId("doc-a");
    expect(loadDismissedCancelledIds().has("doc-a")).toBe(true);
    // Fresh read = refresh
    expect([...loadDismissedCancelledIds()]).toEqual(["doc-a"]);
  });

  it("prunes dismiss when doc leaves cancelled", () => {
    persistDismissedCancelledId("doc-a");
    persistDismissedCancelledId("doc-b");
    const pruned = pruneDismissedCancelledIds(new Set(["doc-b"]));
    expect([...pruned]).toEqual(["doc-b"]);
    expect([...loadDismissedCancelledIds()]).toEqual(["doc-b"]);
  });

  it("remembers cancelled_from_stage for freeze honesty", () => {
    rememberCancelledFromStage("doc-x", "storing");
    expect(loadCancelledFromStage("doc-x")).toBe("storing");
    rememberCancelledFromStage("doc-x", "cancelled"); // ignored
    expect(loadCancelledFromStage("doc-x")).toBe("storing");
  });
});
