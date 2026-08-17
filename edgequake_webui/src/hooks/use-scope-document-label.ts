"use client";
/**
 * @module use-scope-document-label
<<<<<<< HEAD
 * @description Resolves a display label for a document ID from the React Query cache.
 * Cache-only — does NOT trigger any fetch. Used by scope pills to show titles.
 * Falls back to undefined if not cached (caller shows truncated ID).
 * @implements SPEC-031
 */

import type { DocumentSearchItem } from "@/types";
import { useQueryClient } from "@tanstack/react-query";

export function useScopeDocumentLabel(documentId: string): string | undefined {
  const qc = useQueryClient();

  // 1. Try search result caches (warm after picker interaction)
  const searchCaches = qc.getQueriesData<DocumentSearchItem[]>({
    queryKey: ["documents", "search"],
  });
  for (const [, items] of searchCaches) {
    if (items) {
      const found = items.find((item) => item.id === documentId);
      if (found) return found.title;
    }
  }

  // 2. Try full documents list cache
  const listData = qc.getQueryData<{
    items?: Array<{ id: string; title?: string | null; file_name?: string | null }>;
  }>(["documents"]);
  if (listData?.items) {
    const found = listData.items.find((item) => item.id === documentId);
    if (found) return found.title ?? found.file_name ?? undefined;
  }

  return undefined;
=======
 * @description Resolves a display label for a document ID.
 * Cache-first (search + workspace-scoped list + detail), then fetches
 * GET /documents/:id on miss. Never exposes a raw GUID as the label.
 * @implements SPEC-031
 */

import { getDocument } from "@/lib/api/edgequake";
import {
  documentDetailQueryKey,
  findCachedDocumentLabel,
  resolveDocumentDisplayName,
} from "@/lib/documents/display-name";
import { useQuery, useQueryClient } from "@tanstack/react-query";

export interface ScopeDocumentLabelResult {
  /** Human-readable document name when resolved. */
  label: string | undefined;
  /** True while fetching and no cached label is available yet. */
  isLoading: boolean;
}

/**
 * Resolve a document display name for scope / filter pills.
 */
export function useScopeDocumentLabel(
  documentId: string | null | undefined,
): ScopeDocumentLabelResult {
  const qc = useQueryClient();
  const id = documentId?.trim() || "";

  const cached = id ? findCachedDocumentLabel(qc, id) : undefined;

  const query = useQuery({
    queryKey: documentDetailQueryKey(id),
    queryFn: async () => {
      const doc = await getDocument(id);
      return resolveDocumentDisplayName(doc);
    },
    enabled: !!id && !cached,
    staleTime: 5 * 60 * 1000,
    placeholderData: () => findCachedDocumentLabel(qc, id),
  });

  const label = cached ?? query.data ?? undefined;
  const isLoading = !!id && !label && query.isFetching;

  return { label, isLoading };
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}
