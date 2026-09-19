/**
 * Resolve PDF sources that require API auth into in-memory bytes.
 *
 * react-pdf / pdf.js cannot attach Authorization on a bare download URL.
 * Demo (auth_enabled) returns 401 for HEAD/GET without Bearer — same class of
 * bug as AuthenticatedMarkdownImage.
 *
 * We return `{ data: Uint8Array }` (not a blob: URL) so pdf.js never re-fetches
 * and we avoid revoke races with React effect cleanup.
 */

import { buildHeaders } from "@/lib/api/client";

export type PdfFileSource =
  | string
  | { url: string }
  | { data: ArrayBuffer | Uint8Array }
  | null;

/** API paths that serve binary PDF bytes behind auth middleware. */
export function isApiProtectedPdfUrl(url: string): boolean {
  if (!url.includes("/api/v1/documents/")) return false;
  return (
    url.includes("/download") ||
    /\/documents\/pdf\/[^/]+\/download/.test(url) ||
    /\/documents\/[^/]+\/download\/original/.test(url)
  );
}

export function extractPdfSourceUrl(file: PdfFileSource): string | null {
  if (typeof file === "string") return file;
  if (file && typeof file === "object" && "url" in file && typeof file.url === "string") {
    return file.url;
  }
  return null;
}

/**
 * Fetch a protected PDF URL with session headers and return bytes for react-pdf.
 */
export async function fetchAuthenticatedPdfData(
  url: string,
  signal?: AbortSignal,
): Promise<{ data: Uint8Array }> {
  const headers = buildHeaders();
  headers.delete("Content-Type");
  const res = await fetch(url, { headers, method: "GET", signal });
  if (!res.ok) {
    throw new Error(
      `ResponseException: Unexpected server response (${res.status})`,
    );
  }
  const buffer = await res.arrayBuffer();
  return { data: new Uint8Array(buffer) };
}

/** @deprecated Prefer {@link fetchAuthenticatedPdfData} — blob URLs race with revoke. */
export async function fetchAuthenticatedPdfBlobUrl(
  url: string,
  signal?: AbortSignal,
): Promise<string> {
  const { data } = await fetchAuthenticatedPdfData(url, signal);
  const ab = data.buffer.slice(
    data.byteOffset,
    data.byteOffset + data.byteLength,
  ) as ArrayBuffer;
  return URL.createObjectURL(new Blob([ab], { type: "application/pdf" }));
}
