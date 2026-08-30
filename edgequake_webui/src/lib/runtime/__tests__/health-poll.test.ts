import { describe, expect, it } from "bun:test";
import {
  HEALTH_DETAILS_MIN_MS,
  resolveHealthPollIntervals,
} from "../health-poll";

describe("resolveHealthPollIntervals", () => {
  it("disables both intervals by default (configured false)", () => {
    expect(resolveHealthPollIntervals(false, false)).toEqual({
      backendReady: false,
      healthDetails: false,
    });
  });

  it("uses the configured interval for backend-ready and a 30s floor for details", () => {
    expect(resolveHealthPollIntervals(10_000, false)).toEqual({
      backendReady: 10_000,
      healthDetails: HEALTH_DETAILS_MIN_MS,
    });
  });

  it("raises health details to match a slower configured interval", () => {
    expect(resolveHealthPollIntervals(60_000, false)).toEqual({
      backendReady: 60_000,
      healthDetails: 60_000,
    });
  });

  it("keeps both intervals off under Playwright even when configured", () => {
    expect(resolveHealthPollIntervals(10_000, true)).toEqual({
      backendReady: false,
      healthDetails: false,
    });
  });
});
