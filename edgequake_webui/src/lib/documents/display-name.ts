/**
 * @module documents/display-name
 * @description Pure helpers for human-readable document labels (scope pills, etc.).
 * Never prefers a raw UUID/GUID over title or file_name.
 */

import type { QueryClient, QueryKey } from "@tanstack/react-query";
import type { Document, DocumentSearchResponse } from "@/types";

/** Fields needed to resolve a display name. */
export type DocumentNameSource = {
  id: string;
  title?: string | null;
  file_name?: string | null;
};

const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** True when `value` is empty or equals the document id (backend id fallback). */
function isUnusableLabel(value: string | null | undefined, documentId: string): boolean {
  const trimmed = (value ?? "").trim();
  if (!trimmed) return true;
  if (trimmed === documentId) return true;
  if (UUID_RE.test(trimmed) && trimmed.toLowerCase() === documentId.toLowerCase()) {
    return true;
  }
  return false;
}

/**
 * Resolve a human-readable document name.
 * Prefer title, then file_name; never prefer the raw document id / UUID.
 * Last resort: short opaque label (not a full GUID).
 */
export function resolveDocumentDisplayName(doc: DocumentNameSource): string {
  const title = doc.title?.trim();
  const fileName = doc.file_name?.trim();

  if (title && !isUnusableLabel(title, doc.id)) return title;
  if (fileName && !isUnusableLabel(fileName, doc.id)) return fileName;
  if (title) return title;
  if (fileName) return fileName;
  return `Document ${doc.id.slice(0, 8)}`;
}

export const documentDetailQueryKey = (documentId: string): QueryKey =>
  ["documents", "detail", documentId] as const;

type ListCacheShape = {
  items?: DocumentNameSource[];
};

function isSearchOrDetailKey(key: QueryKey): boolean {
  return key[1] === "search" || key[1] === "detail";
}

/**
 * Scan React Query caches for a display label for `documentId`.
 * Uses prefix matching so workspace-scoped list keys are found.
 */
export function findCachedDocumentLabel(
  qc: QueryClient,
  documentId: string,
): string | undefined {
  // 1. Detail cache
  const detail = qc.getQueryData<Document | string>(
    documentDetailQueryKey(documentId),
  );
  if (typeof detail === "string" && detail.trim()) return detail;
  if (detail && typeof detail === "object" && "id" in detail) {
    return resolveDocumentDisplayName(detail);
  }

  // 2. Search caches (picker)
  const searchCaches = qc.getQueriesData<DocumentSearchResponse>({
    queryKey: ["documents", "search"],
  });
  for (const [, data] of searchCaches) {
    const found = data?.items?.find((item) => item.id === documentId);
    if (found?.title && !isUnusableLabel(found.title, documentId)) {
      return found.title;
    }
    if (found?.title?.trim()) return found.title.trim();
  }

  // 3. List page caches (prefix match; skip search/detail subkeys)
  const listCaches = qc.getQueriesData<ListCacheShape>({
    queryKey: ["documents"],
  });
  for (const [key, data] of listCaches) {
    if (isSearchOrDetailKey(key)) continue;
    const found = data?.items?.find((item) => item.id === documentId);
    if (found) return resolveDocumentDisplayName(found);
  }

  return undefined;
}
