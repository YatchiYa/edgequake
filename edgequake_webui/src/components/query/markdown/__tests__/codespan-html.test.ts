import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import {
  isHtmlCodespanCandidate,
  tryHtmlCodespan,
} from "../utils/codespan-html";

describe("isHtmlCodespanCandidate", () => {
  it("accepts c<sub>i</sub>(W)", () => {
    expect(isHtmlCodespanCandidate("c<sub>i</sub>(W)")).toBe(true);
  });

  it("accepts a<sub>i</sub><sup>*</sup>", () => {
    expect(isHtmlCodespanCandidate("a<sub>i</sub><sup>*</sup>")).toBe(true);
  });

  it("accepts φ<sub>i</sub>", () => {
    expect(isHtmlCodespanCandidate("φ<sub>i</sub>")).toBe(true);
  });

  it("rejects plain codespan", () => {
    expect(isHtmlCodespanCandidate("plain")).toBe(false);
  });

  it("rejects codespan without sub/sup", () => {
    expect(isHtmlCodespanCandidate("<em>x</em>")).toBe(false);
  });

  it("rejects script tags", () => {
    expect(isHtmlCodespanCandidate("<script>x</script>")).toBe(false);
  });

  it("rejects mixed junk with script", () => {
    expect(isHtmlCodespanCandidate("foo <script>")).toBe(false);
  });

  it("rejects event handlers", () => {
    expect(isHtmlCodespanCandidate('c<sub onclick="alert(1)">i</sub>')).toBe(
      false,
    );
  });

  it("rejects unknown tags mixed with sub", () => {
    expect(isHtmlCodespanCandidate("c<sub>i</sub><div>x</div>")).toBe(false);
  });
});

describe("tryHtmlCodespan", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("returns null in node (no window) even for valid candidates", async () => {
    // vitest env is node — window undefined
    expect(tryHtmlCodespan("c<sub>i</sub>(W)")).toBeNull();
  });

  it("returns sanitized HTML when window + sanitize keep sub", async () => {
    vi.stubGlobal("window", {});
    vi.doMock("../utils/sanitize-html", () => ({
      sanitizeHtml: (html: string) => html,
    }));
    const { tryHtmlCodespan: tryFn } = await import("../utils/codespan-html");
    expect(tryFn("c<sub>i</sub>(W)")).toBe("c<sub>i</sub>(W)");
    expect(tryFn("plain")).toBeNull();
    expect(tryFn("<script>x</script>")).toBeNull();
  });

  it("returns null when sanitize strips all sub/sup", async () => {
    vi.stubGlobal("window", {});
    vi.doMock("../utils/sanitize-html", () => ({
      sanitizeHtml: (html: string) => html.replace(/<[^>]*>/g, ""),
    }));
    const { tryHtmlCodespan: tryFn } = await import("../utils/codespan-html");
    expect(tryFn("c<sub>i</sub>(W)")).toBeNull();
  });
});
