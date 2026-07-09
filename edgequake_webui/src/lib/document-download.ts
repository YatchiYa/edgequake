/**
 * @module document-download
 * @description Utilities for downloading document originals and extracted markdown.
 *
 * @implements SPEC-002 - Document Viewer download actions
 */

import {
  getDocumentMarkdownDownloadUrl,
  getDocumentOriginalDownloadUrl,
  getPdfDownloadUrl,
} from "@/lib/api/edgequake/documents";
import { downloadFile, sanitizeFilename } from "@/lib/export-conversation";
import type { Document } from "@/types";

/** Resolve the PDF id used for original PDF download URLs. */
export function resolvePdfId(document: Pick<Document, "id" | "pdf_id" | "source_type">): string | null {
  if (document.pdf_id) return document.pdf_id;
  if (document.source_type === "pdf") return document.id;
  return null;
}

/** Whether the document has a retrievable original binary (PDF or stored upload). */
export function canDownloadOriginal(document: Document): boolean {
  if (resolvePdfId(document)) return true;
  if (document.metadata?.has_original === true) return true;
  // Phase 2: non-PDF uploads store originals; legacy rows may 404 at download time.
  return document.source_type === "image" || document.source_type === "file";
}

/** Status values where extracted markdown is expected to exist server-side. */
function isMarkdownReadyStatus(status?: Document["status"]): boolean {
  return status === "completed" || status === "partial_failure" || status === "indexed";
}

/** Whether markdown exists on the server but was omitted from the list payload. */
function hasServerMarkdown(
  document: Pick<Document, "content_length" | "content_summary" | "chunk_count" | "status">,
): boolean {
  if ((document.content_length ?? 0) > 0) return true;
  if (document.content_summary?.trim()) return true;
  return (document.chunk_count ?? 0) > 0 && isMarkdownReadyStatus(document.status);
}

/** Whether markdown/text content is available for download. */
export function canDownloadMarkdown(
  document: Pick<
    Document,
    "content" | "content_length" | "content_summary" | "chunk_count" | "status"
  >,
  contentOverride?: string | null,
): boolean {
  const content = contentOverride ?? document.content;
  if (content?.trim()) return true;
  return hasServerMarkdown(document);
}

/** Build the download URL for the original file. */
export function getOriginalDownloadUrl(document: Document): string | null {
  const pdfId = resolvePdfId(document);
  if (pdfId) return getPdfDownloadUrl(pdfId);
  if (document.metadata?.has_original === true) {
    return getDocumentOriginalDownloadUrl(document.id);
  }
  if (document.source_type === "image" || document.source_type === "file") {
    return getDocumentOriginalDownloadUrl(document.id);
  }
  return null;
}

/** Suggested filename for the original download. */
export function getOriginalFilename(document: Document): string {
  return document.file_name || document.title || `document-${document.id.slice(0, 8)}`;
}

/** Suggested filename for markdown download. */
export function getMarkdownFilename(document: Pick<Document, "id" | "file_name" | "title">): string {
  const baseName = document.file_name || document.title || `document-${document.id.slice(0, 8)}`;
  if (baseName.toLowerCase().endsWith(".md")) return baseName;
  const stem = baseName.includes(".") ? baseName.replace(/\.[^.]+$/, "") : baseName;
  return `${sanitizeFilename(stem)}.md`;
}

function triggerUrlDownload(url: string, filename: string): void {
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}

/** Download the original file (PDF or stored upload bytes). */
export function downloadDocumentOriginal(document: Document): void {
  const url = getOriginalDownloadUrl(document);
  if (!url) {
    throw new Error("Original file is not available for this document");
  }
  triggerUrlDownload(url, getOriginalFilename(document));
}

/** Download extracted markdown as a .md file (client blob or server URL). */
export function downloadDocumentMarkdown(
  document: Pick<
    Document,
    "id" | "file_name" | "title" | "content" | "content_length" | "content_summary" | "chunk_count" | "status"
  >,
  contentOverride?: string | null,
): void {
  const content = contentOverride ?? document.content;
  if (content?.trim()) {
    downloadFile(content, getMarkdownFilename(document), "text/markdown;charset=utf-8");
    return;
  }
  if (hasServerMarkdown(document)) {
    triggerUrlDownload(
      getDocumentMarkdownDownloadUrl(document.id),
      getMarkdownFilename(document),
    );
    return;
  }
  throw new Error("No markdown content available");
}
