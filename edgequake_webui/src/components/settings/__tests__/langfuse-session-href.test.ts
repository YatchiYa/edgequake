import { describe, expect, it } from "vitest";
import { langfuseSessionHref } from "@/components/settings/langfuse-open-trace-link";

describe("langfuseSessionHref", () => {
  it("builds project-scoped session URL from the configured host", () => {
    expect(
      langfuseSessionHref(
        "https://us.cloud.langfuse.com/",
        "conv-1",
        "clkproj",
      ),
    ).toBe("https://us.cloud.langfuse.com/project/clkproj/sessions/conv-1");
  });

  it("uses a self-hosted configured base URL as-is", () => {
    expect(
      langfuseSessionHref("http://localhost:3000", "sess-9", "proj-local"),
    ).toBe("http://localhost:3000/project/proj-local/sessions/sess-9");
    expect(
      langfuseSessionHref("http://localhost:3310", "conv-1", "edgequake-local"),
    ).toBe("http://localhost:3310/project/edgequake-local/sessions/conv-1");
  });

  it("encodes session and project path segments", () => {
    expect(langfuseSessionHref("https://cloud.langfuse.com", "a/b", "p 1")).toBe(
      "https://cloud.langfuse.com/project/p%201/sessions/a%2Fb",
    );
  });

  it("returns null without project id (no 404 /sessions/ URL)", () => {
    expect(langfuseSessionHref("https://us.cloud.langfuse.com", "conv-1")).toBeNull();
    expect(langfuseSessionHref("https://us.cloud.langfuse.com", "conv-1", "")).toBeNull();
    expect(langfuseSessionHref("https://us.cloud.langfuse.com", "", "proj")).toBeNull();
  });
});
