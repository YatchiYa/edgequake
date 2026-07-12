import { describe, expect, it } from "vitest";
import {
  assertQueryModeMetaComplete,
  getQueryModeMeta,
  QUERY_MODE_META,
} from "@/lib/query/query-mode-meta";
import { isQueryMode, QUERY_MODES, QUERY_MODES_SELECTOR } from "@/types";

describe("QueryMode backend parity (UI-DRY-005)", () => {
  it("includes mix and bypass modes from backend", () => {
    expect(QUERY_MODES).toContain("mix");
    expect(QUERY_MODES).toContain("bypass");
  });

  it("validates known backend mode strings", () => {
    for (const mode of QUERY_MODES) {
      expect(isQueryMode(mode)).toBe(true);
    }
    expect(isQueryMode("invalid")).toBe(false);
  });

  it("surfaces every backend mode in the selector", () => {
    expect([...QUERY_MODES_SELECTOR].sort()).toEqual([...QUERY_MODES].sort());
  });
});

describe("QueryModeMeta tooltips (UX)", () => {
  it("has meta + description for every backend mode", () => {
    expect(assertQueryModeMetaComplete()).toBe(true);
    expect(QUERY_MODE_META).toHaveLength(QUERY_MODES.length);
    for (const mode of QUERY_MODES) {
      const meta = getQueryModeMeta(mode);
      expect(meta.id).toBe(mode);
      expect(meta.label.length).toBeGreaterThan(0);
      expect(meta.description.length).toBeGreaterThan(40);
      expect(meta.apiName).toBe(mode);
    }
  });

  it("marks mix as the recommended Smart mode", () => {
    const mix = getQueryModeMeta("mix");
    expect(mix.recommended).toBe(true);
    expect(mix.label).toBe("Smart");
  });

  it("does not claim naive skips all retrieval (that is bypass)", () => {
    const naive = getQueryModeMeta("naive");
    const bypass = getQueryModeMeta("bypass");
    expect(naive.description.toLowerCase()).toMatch(/chunk/);
    expect(naive.description.toLowerCase()).not.toMatch(/skips retrieval/);
    expect(bypass.description.toLowerCase()).toMatch(/skip/);
  });
});
