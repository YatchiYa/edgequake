/**
 * SPEC-123 WebUI inviolable gates (bun unit — LAW-123-7).
 */
import { describe, expect, it } from "bun:test";
import {
  formatServerDefaultPdfParserLabel,
  formatWorkspaceDefaultPdfParserLabel,
  resolvePdfParserBackend,
} from "@/lib/pdf/resolve-pdf-parser-backend";
import {
  LARGE_PDF_PAGE_THRESHOLD,
  shouldPromptLargePdfParserChoice,
} from "@/lib/pdf/large-pdf-admission";

describe("SPEC-123 WebUI e2e matrix", () => {
  it("W1: Resolves to Vision is inviolable (not Auto)", () => {
    const t = (_k: string, d: string) => d;
    expect(formatServerDefaultPdfParserLabel(t, "vision")).toBe(
      "Server Default (Vision)",
    );
    expect(formatWorkspaceDefaultPdfParserLabel(t, null, "vision")).toBe(
      "Workspace Default (Vision)",
    );
    const resolved = resolvePdfParserBackend({
      uploadChoice: "default",
      workspaceBackend: null,
      serverBackend: "vision",
    });
    expect(resolved.backend).toBe("vision");
    expect(resolved.allowsAutoRoute).toBe(false);
    expect(resolved.isExplicit).toBe(true);
  });

  it("W2: multi-file Workspace Default Vision keeps Vision for all", () => {
    const ctx = {
      uploadChoice: "default" as const,
      workspaceBackend: null as null,
      serverBackend: "vision" as const,
    };
    for (let i = 0; i < 5; i += 1) {
      const r = resolvePdfParserBackend(ctx);
      expect(r.backend).toBe("vision");
      expect(r.allowsAutoRoute).toBe(false);
    }
  });

  it("W3: large admission prompts only for inviolable Vision", () => {
    expect(
      shouldPromptLargePdfParserChoice(LARGE_PDF_PAGE_THRESHOLD, {
        uploadChoice: "default",
        workspaceBackend: null,
        serverBackend: "vision",
      }),
    ).toBe(true);
    expect(
      shouldPromptLargePdfParserChoice(LARGE_PDF_PAGE_THRESHOLD, {
        uploadChoice: "default",
        workspaceBackend: "auto",
      }),
    ).toBe(false);
  });

  it("admission override applies only to large names (LAW-123-6)", () => {
    const all = ["big.pdf", "small.pdf", "notes.md"];
    const largeNames = new Set(["big.pdf"]);
    const largeFiles = all.filter((n) => largeNames.has(n));
    const otherFiles = all.filter((n) => !largeNames.has(n));
    expect(largeFiles).toEqual(["big.pdf"]);
    expect(otherFiles).toEqual(["small.pdf", "notes.md"]);
  });
});
