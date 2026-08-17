import { getRuntimeServerBaseUrl } from "@/lib/runtime-config";
import { api } from "../client";

export interface PdfContentResponse {
  pdf_id: string;
  document_id?: string | null;
  filename: string;
  file_size_bytes: number;
  content_type: string;
  markdown_content: string | null;
  is_processed: boolean;
}

export async function getPdfContent(
  pdfId: string,
): Promise<PdfContentResponse> {
  return api.get<PdfContentResponse>(`/documents/pdf/${pdfId}/content`);
}

export function getPdfDownloadUrl(pdfId: string): string {
  return `${getRuntimeServerBaseUrl()}/api/v1/documents/pdf/${pdfId}/download`;
}

export function getDocumentOriginalDownloadUrl(documentId: string): string {
  return `${getRuntimeServerBaseUrl()}/api/v1/documents/${documentId}/download/original`;
}

export function getDocumentMarkdownDownloadUrl(documentId: string): string {
  return `${getRuntimeServerBaseUrl()}/api/v1/documents/${documentId}/download/markdown`;
}

export function getDocumentMmAssetUrl(
  documentId: string,
  assetRelPath: string,
): string {
  const baseUrl = getRuntimeServerBaseUrl();
  const cleaned = assetRelPath.replace(/^\/+/, "");
  const stem = cleaned.split("/").pop()?.replace(/\.[^.]+$/, "") ?? cleaned;
  if (stem && !stem.includes("..")) {
    return `${baseUrl}/api/v1/documents/${documentId}/assets/${encodeURIComponent(stem)}`;
  }
  return `${baseUrl}/api/v1/documents/${documentId}/mm-assets/${cleaned}`;
}

export async function listDocumentAssets(
  documentId: string,
): Promise<{
  document_id: string;
  assets: Array<{ asset_id?: string; path?: string }>;
}> {
  return api.get(`/documents/${documentId}/assets`);
}

export async function includeDocumentAssetsFromPdf(
  documentId: string,
): Promise<{
  document_id: string;
  pages_rendered: number;
  assets_persisted: number;
  markdown_updated: boolean;
}> {
  return api.post(`/documents/${documentId}/assets/include-from-pdf`);
}

export function isDurableMmAssetHref(href: string): boolean {
  const value = href.trim();
  return (
    value.startsWith("assets/") ||
    value.includes("/mm-assets/") ||
    /\/documents\/[^/]+\/assets\//.test(value)
  );
}

function preferredViewerAsset(page: number, chunk: string): string | null {
  const durable = chunk.match(
    /!?\[[^\]]*\]\((assets\/page-\d{4}-(?:fig-\d{2}|chart|table-\d{2})\.png)\)/i,
  );
  void page;
  return durable?.[1] ?? null;
}

export function bindFigureImagesToPageAssets(markdown: string): string {
  if (!markdown) return markdown;
  const pagePattern = /<!--\s*edgequake-page:(\d+)\s*-->/g;
  const markers: { index: number; page: number }[] = [];
  let match: RegExpExecArray | null;
  while ((match = pagePattern.exec(markdown)) !== null) {
    markers.push({ index: match.index, page: Number.parseInt(match[1], 10) });
  }
  if (markers.length === 0) {
    const asset = preferredViewerAsset(1, markdown);
    if (!asset) return markdown;
    return injectFigureLocalImages(
      markdown.replace(
        /!\[([^\]]*)\]\(([^)]*)\)/g,
        (full, alt: string, href: string) =>
          isDurableMmAssetHref(href) ? full : `![${alt}](${asset})`,
      ),
      asset,
    );
  }

  let output = "";
  for (let index = 0; index < markers.length; index += 1) {
    const start = markers[index].index;
    const end =
      index + 1 < markers.length ? markers[index + 1].index : markdown.length;
    const chunk = markdown.slice(start, end);
    const asset = preferredViewerAsset(markers[index].page, chunk);
    if (!asset) {
      output += chunk;
      continue;
    }
    const bound = chunk.replace(
      /!\[([^\]]*)\]\(([^)]*)\)/g,
      (full, alt: string, href: string) =>
        isDurableMmAssetHref(href)
          ? full
          : `![${alt || "Page image"}](${asset})`,
    );
    output += injectFigureLocalImages(bound, asset);
  }
  return markers[0].index > 0
    ? markdown.slice(0, markers[0].index) + output
    : output;
}

function isFigureHeadingLine(line: string): boolean {
  const stripped = line
    .trim()
    .replace(/^#+\s*/, "")
    .replace(/^[*_]+/, "")
    .replace(/[*_]+$/, "")
    .trim();
  return /^figure\s+\d/i.test(stripped);
}

function figureHeadingAlt(line: string): string {
  return (
    line
      .trim()
      .replace(/^#+\s*/, "")
      .replace(/^\*+|\*+$/g, "")
      .trim()
      .slice(0, 120)
      .replaceAll("[", "")
      .replaceAll("]", "") || "Figure"
  );
}

export function injectFigureLocalImages(
  markdown: string,
  assetRelPath: string,
): string {
  if (!markdown.trim()) return markdown;
  const lines = markdown.split("\n");
  const output: string[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    output.push(line);
    if (!isFigureHeadingLine(line)) continue;
    let nextIndex = index + 1;
    while (nextIndex < lines.length && lines[nextIndex].trim() === "") {
      nextIndex += 1;
    }
    const next =
      nextIndex < lines.length ? lines[nextIndex].trimStart() : "";
    const hasNearby =
      next.startsWith("![") ||
      next.includes("](assets/") ||
      next.startsWith("<drawing");
    if (!hasNearby) {
      output.push("", `![${figureHeadingAlt(line)}](${assetRelPath})`);
    }
  }
  return output.join("\n");
}

export function stripDrawingTags(markdown: string): string {
  return markdown
    .replace(/<drawing\b[^>]*\/?\s*>/gi, "")
    .replace(/\n{3,}/g, "\n\n");
}

export function rewriteMarkdownMmAssetUrls(
  markdown: string,
  documentId: string | null | undefined,
): string {
  if (!markdown || !documentId) return markdown;
  const bound = stripDrawingTags(bindFigureImagesToPageAssets(markdown));
  return bound.replace(
    /!\[([^\]]*)\]\((assets\/[^)\s]+)\)/g,
    (_match, alt: string, relative: string) =>
      `![${alt}](${getDocumentMmAssetUrl(documentId, relative)})`,
  );
}
