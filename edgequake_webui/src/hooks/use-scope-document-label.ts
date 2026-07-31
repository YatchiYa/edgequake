"use client";
/**
 * @module use-scope-document-label
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
}
