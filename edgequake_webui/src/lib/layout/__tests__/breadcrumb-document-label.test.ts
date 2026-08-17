import { describe, expect, it } from "vitest";
import {
  extractDocumentIdFromPath,
  formatGuidShort,
  isDocumentIdSegment,
  resolveDocumentBreadcrumbLabel,
} from "../breadcrumb-document-label";

describe("breadcrumb-document-label", () => {
  it("detects UUID document segments", () => {
    expect(
      isDocumentIdSegment("019fc1a7-6172-7169-87ad-d13abbaf60a0"),
    ).toBe(true);
    expect(
      isDocumentIdSegment("staging:019fc1a7-6172-7169-87ad-d13abbaf60a0"),
    ).toBe(true);
    expect(isDocumentIdSegment("documents")).toBe(false);
  });

  it("extracts document id from path", () => {
    expect(
      extractDocumentIdFromPath(
        "/documents/019fc1a7-6172-7169-87ad-d13abbaf60a0",
      ),
    ).toBe("019fc1a7-6172-7169-87ad-d13abbaf60a0");
    expect(
      extractDocumentIdFromPath(
        "/w/acme/documents/019fc1a7-6172-7169-87ad-d13abbaf60a0",
      ),
    ).toBe("019fc1a7-6172-7169-87ad-d13abbaf60a0");
    expect(extractDocumentIdFromPath("/documents")).toBeNull();
  });

  it("prefers file_name over title for label", () => {
    expect(
      resolveDocumentBreadcrumbLabel({
        file_name: "HN_7ZaGW0AEt3Uc.jpeg",
        title: "Vision analysis",
      }),
    ).toBe("HN_7ZaGW0AEt3Uc.jpeg");
    expect(
      resolveDocumentBreadcrumbLabel({
        file_name: "  ",
        title: "Vision analysis",
      }),
    ).toBe("Vision analysis");
    expect(resolveDocumentBreadcrumbLabel(null)).toBeNull();
  });

  it("shortens GUIDs for chrome", () => {
    expect(formatGuidShort("019fc1a7-6172-7169-87ad-d13abbaf60a0")).toBe(
      "019fc1a7…",
    );
  });
});
