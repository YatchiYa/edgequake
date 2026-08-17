import { describe, expect, it } from "bun:test";
import {
<<<<<<< HEAD
  getServerDefaultPdfParserBackend,
=======
  formatServerDefaultPdfParserLabel,
  getServerDefaultPdfParserBackend,
  pdfParserBackendDisplayName,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  resolvePdfParserBackend,
  resolvesToVisionParser,
  type PdfParserResolutionContext,
} from "@/lib/pdf/resolve-pdf-parser-backend";
import {
  LARGE_PDF_PAGE_THRESHOLD,
  shouldPromptLargePdfParserChoice,
} from "@/lib/pdf/large-pdf-admission";

describe("resolvePdfParserBackend", () => {
  it("upload override wins over workspace and server", () => {
    const ctx: PdfParserResolutionContext = {
      uploadChoice: "edgeparse",
      workspaceBackend: "vision",
      serverBackend: "vision",
    };
    const resolved = resolvePdfParserBackend(ctx);
    expect(resolved.backend).toBe("edgeparse");
    expect(resolved.source).toBe("upload");
    expect(resolved.isExplicit).toBe(true);
  });

  it("workspace default wins when upload is default", () => {
    const ctx: PdfParserResolutionContext = {
      uploadChoice: "default",
      workspaceBackend: "edgeparse",
      serverBackend: "vision",
    };
    const resolved = resolvePdfParserBackend(ctx);
    expect(resolved.backend).toBe("edgeparse");
    expect(resolved.source).toBe("workspace");
  });

  it("falls back to server then vision", () => {
    const ctx: PdfParserResolutionContext = {
      uploadChoice: "default",
      workspaceBackend: undefined,
      serverBackend: "vision",
    };
    expect(resolvePdfParserBackend(ctx).backend).toBe("vision");
    expect(resolvePdfParserBackend(ctx).source).toBe("server");
  });

  it("getServerDefaultPdfParserBackend defaults to vision", () => {
    expect(getServerDefaultPdfParserBackend()).toBe("vision");
  });
<<<<<<< HEAD
=======

  it("never-silent server default label includes resolved backend", () => {
    expect(pdfParserBackendDisplayName("vision")).toBe("Vision");
    expect(pdfParserBackendDisplayName("edgeparse")).toBe("EdgeParse");
    const label = formatServerDefaultPdfParserLabel(
      (_key, defaultValue) => defaultValue,
      "vision",
    );
    expect(label).toBe("Server Default (Vision)");
    expect(label).not.toBe("Server Default");
  });
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
});

describe("shouldPromptLargePdfParserChoice", () => {
  it("prompts for large PDF when Vision is resolved", () => {
    expect(
      shouldPromptLargePdfParserChoice(LARGE_PDF_PAGE_THRESHOLD, {
        uploadChoice: "default",
        workspaceBackend: undefined,
        serverBackend: "vision",
      }),
    ).toBe(true);
  });

  it("skips dialog when upload selects EdgeParse", () => {
    expect(
      shouldPromptLargePdfParserChoice(603, {
        uploadChoice: "edgeparse",
        workspaceBackend: undefined,
        serverBackend: "vision",
      }),
    ).toBe(false);
  });

  it("skips dialog when workspace default is EdgeParse", () => {
    expect(
      shouldPromptLargePdfParserChoice(603, {
        uploadChoice: "default",
        workspaceBackend: "edgeparse",
        serverBackend: "vision",
      }),
    ).toBe(false);
  });

  it("skips dialog below page threshold even with Vision", () => {
    expect(
      shouldPromptLargePdfParserChoice(LARGE_PDF_PAGE_THRESHOLD - 1, {
        uploadChoice: "default",
        serverBackend: "vision",
      }),
    ).toBe(false);
  });

  it("resolvesToVisionParser matches backend chain", () => {
    expect(
      resolvesToVisionParser({
        uploadChoice: "vision",
        workspaceBackend: "edgeparse",
      }),
    ).toBe(true);
  });
});
