/**
 * Resolve PDF sources that require API auth into blob URLs.
 *
 * react-pdf / pdf.js cannot attach Authorization on a bare download URL.
 * Demo (auth_enabled) returns 401 for HEAD/GET without Bearer — same class of
 * bug as AuthenticatedMarkdownImage.
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
 * Fetch a protected PDF URL with session headers and return an object URL.
 * Caller must revoke the URL when done.
 */
export async function fetchAuthenticatedPdfBlobUrl(
  url: string,
  signal?: AbortSignal,
): Promise<string> {
  const headers = buildHeaders();
  headers.delete("Content-Type");
  const res = await fetch(url, { headers, method: "GET", signal });
  if (!res.ok) {
    throw new Error(
      `ResponseException: Unexpected server response (${res.status})`,
    );
  }
  const blob = await res.blob();
  return URL.createObjectURL(blob);
}
