import { describe, expect, it, vi } from "vitest";

vi.mock("@/lib/runtime-config", () => ({
  getRuntimeServerBaseUrl: () => "https://api.example.test",
}));

import { isTrustedImageOrigin } from "../AuthenticatedMarkdownImage";

describe("markdown image origin guard (token / data exfiltration)", () => {
  it("trusts same-origin, API-origin, data and blob images", () => {
    expect(isTrustedImageOrigin("/api/v1/documents/d1/assets/page-0001")).toBe(true);
    expect(isTrustedImageOrigin(`${window.location.origin}/x/mm-assets/a.png`)).toBe(true);
    expect(isTrustedImageOrigin("https://api.example.test/api/v1/documents/d/mm-assets/a.png")).toBe(true);
    expect(isTrustedImageOrigin("data:image/png;base64,AAAA")).toBe(true);
  });

  it("refuses foreign hosts even when the path looks like an asset", () => {
    expect(isTrustedImageOrigin("https://evil.test/mm-assets/a.png?d=secret")).toBe(false);
    expect(isTrustedImageOrigin("//evil.test/documents/x/assets/y")).toBe(false);
    expect(isTrustedImageOrigin("https://api.example.test.evil.test/a.png")).toBe(false);
  });
});
