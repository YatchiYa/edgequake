import { describe, expect, it } from "vitest";
import { langfuseSessionHref } from "@/components/settings/langfuse-open-trace-link";

describe("langfuseSessionHref", () => {
  it("builds sessions deep-link from ui_url", () => {
    expect(langfuseSessionHref("https://us.cloud.langfuse.com/", "conv-1")).toBe(
      "https://us.cloud.langfuse.com/sessions/conv-1",
    );
  });

  it("encodes session path segment", () => {
    expect(langfuseSessionHref("https://cloud.langfuse.com", "a/b")).toBe(
      "https://cloud.langfuse.com/sessions/a%2Fb",
    );
  });
});
