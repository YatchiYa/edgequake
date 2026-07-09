import { describe, expect, it } from "bun:test";
import type { Document } from "@/types";
import {
  canDownloadMarkdown,
  canDownloadOriginal,
  getMarkdownFilename,
  getOriginalDownloadUrl,
  resolvePdfId,
} from "@/lib/document-download";
import { getDocumentMarkdownDownloadUrl } from "@/lib/api/edgequake/documents";

const baseDoc: Document = {
  id: "doc-123",
  title: "Test Document",
  file_name: "report.pdf",
  content: "# Hello\n\nWorld",
};

describe("document-download", () => {
  it("resolvePdfId prefers pdf_id then pdf source_type", () => {
    expect(resolvePdfId({ ...baseDoc, pdf_id: "pdf-1", source_type: "pdf" })).toBe("pdf-1");
    expect(resolvePdfId({ ...baseDoc, pdf_id: undefined, source_type: "pdf" })).toBe("doc-123");
    expect(resolvePdfId({ ...baseDoc, source_type: "text" })).toBeNull();
  });

  it("canDownloadOriginal for PDF and stored originals", () => {
    expect(canDownloadOriginal({ ...baseDoc, pdf_id: "pdf-1", source_type: "pdf" })).toBe(true);
    expect(
      canDownloadOriginal({
        ...baseDoc,
        source_type: "image",
        metadata: { has_original: true },
      }),
    ).toBe(true);
    expect(canDownloadOriginal({ ...baseDoc, source_type: "image" })).toBe(true);
    expect(canDownloadOriginal({ ...baseDoc, source_type: "text" })).toBe(false);
  });

  it("canDownloadMarkdown requires non-empty content or server-side signals", () => {
    expect(canDownloadMarkdown(baseDoc)).toBe(true);
    expect(canDownloadMarkdown({ ...baseDoc, content: "  " })).toBe(false);
    expect(canDownloadMarkdown({ ...baseDoc, content: undefined, content_length: 42 })).toBe(true);
    expect(
      canDownloadMarkdown({
        ...baseDoc,
        content: undefined,
        content_summary: "Preview text",
      }),
    ).toBe(true);
    expect(
      canDownloadMarkdown({
        ...baseDoc,
        content: undefined,
        chunk_count: 3,
        status: "completed",
      }),
    ).toBe(true);
    expect(
      canDownloadMarkdown({
        ...baseDoc,
        content: undefined,
        chunk_count: 3,
        status: "processing",
      }),
    ).toBe(false);
    expect(canDownloadMarkdown(baseDoc, "# override")).toBe(true);
  });

  it("getOriginalDownloadUrl routes PDF vs stored original", () => {
    expect(getOriginalDownloadUrl({ ...baseDoc, pdf_id: "pdf-1" })).toContain(
      "/api/v1/documents/pdf/pdf-1/download",
    );
    expect(
      getOriginalDownloadUrl({
        ...baseDoc,
        metadata: { has_original: true },
      }),
    ).toContain("/api/v1/documents/doc-123/download/original");
    expect(getOriginalDownloadUrl({ ...baseDoc, source_type: "text" })).toBeNull();
  });

  it("getMarkdownFilename normalizes extension", () => {
    expect(getMarkdownFilename({ ...baseDoc, file_name: "notes.md" })).toBe("notes.md");
    expect(getMarkdownFilename({ ...baseDoc, file_name: "notes.txt" })).toBe("notes.md");
    expect(getMarkdownFilename({ id: "x", title: "My Doc!" })).toBe("my-doc.md");
  });

  it("getDocumentMarkdownDownloadUrl targets server markdown endpoint", () => {
    expect(getDocumentMarkdownDownloadUrl("doc-123")).toContain(
      "/api/v1/documents/doc-123/download/markdown",
    );
  });
});
