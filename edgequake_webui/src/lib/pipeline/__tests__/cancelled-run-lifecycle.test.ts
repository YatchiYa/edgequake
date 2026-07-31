/**
 * Cancelled Active Runs lifecycle SSOT.
 */
import { afterEach, describe, expect, it } from "vitest";
import {
  CANCELLED_RUN_GRACE_MS,
  cancelledRunKey,
  cancelledRunRemainingMs,
  dismissCancelledRun,
  isCancelledRunVisible,
  observeCancelledRun,
  pauseCancelledRunGrace,
  resetCancelledRunLifecycleForTests,
  resumeCancelledRunGrace,
} from "../cancelled-run-lifecycle";

describe("cancelled-run-lifecycle", () => {
  afterEach(() => {
    resetCancelledRunLifecycleForTests();
  });

  it("is visible within grace and hidden after", () => {
    const key = cancelledRunKey("doc-1", "track-1");
    const t0 = 1_000_000;
    observeCancelledRun(key, t0);
    expect(isCancelledRunVisible(key, t0)).toBe(true);
    expect(isCancelledRunVisible(key, t0 + CANCELLED_RUN_GRACE_MS - 1)).toBe(
      true,
    );
    expect(isCancelledRunVisible(key, t0 + CANCELLED_RUN_GRACE_MS)).toBe(false);
  });

  it("dismiss hides immediately", () => {
    const key = cancelledRunKey("doc-2", null);
    observeCancelledRun(key, 0);
    dismissCancelledRun(key);
    expect(isCancelledRunVisible(key, 100)).toBe(false);
    expect(cancelledRunRemainingMs(key, 100)).toBe(0);
  });

  it("pause extends grace window", () => {
    const key = cancelledRunKey("doc-3", "t");
    const t0 = 5_000;
    observeCancelledRun(key, t0);
    pauseCancelledRunGrace(key, t0 + 1_000);
    // 10s wall clock with 9s paused → only 1s effective
    expect(isCancelledRunVisible(key, t0 + 10_000)).toBe(true);
    resumeCancelledRunGrace(key, t0 + 10_000);
    expect(isCancelledRunVisible(key, t0 + 10_000 + CANCELLED_RUN_GRACE_MS)).toBe(
      false,
    );
  });

  it("ages out immediately when cancelledAt is older than grace", () => {
    const key = cancelledRunKey("doc-old", "t");
    const now = 2_000_000;
    const old = now - CANCELLED_RUN_GRACE_MS - 60_000;
    expect(
      isCancelledRunVisible(key, now, CANCELLED_RUN_GRACE_MS, {
        cancelledAt: old,
      }),
    ).toBe(false);
  });

  it("aged cancel does not reappear after dismiss then re-observe", () => {
    const key = cancelledRunKey("doc-4", "t");
    dismissCancelledRun(key);
    // observe is no-op when already present (dismissed)
    observeCancelledRun(key, Date.now());
    expect(isCancelledRunVisible(key)).toBe(false);
  });
});
