"use client";
/**
 * @module use-document-search
 * @description Type-ahead document search hook for the scope picker.
 * Debounces the query, caches results via React Query.
<<<<<<< HEAD
 * Returns [] on empty query to show most recent completed documents.
=======
 * Returns the most recent completed documents on an empty query.
 *
 * The backend `/documents/search` caps `page_size` at 50 and exposes no offset
 * (`has_more = total > page_size`), so "load more" is modelled as growing the
 * requested `page_size` up to that cap; specific documents beyond the window are
 * reachable via the type-ahead query (server-side title filter).
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
 * @implements SPEC-031: Document search for scope picker
 */

import { useDebounce } from "@/hooks/use-debounce";
import { searchDocuments } from "@/lib/api/edgequake/documents";
<<<<<<< HEAD
import type { DocumentSearchItem } from "@/types";
import { useQuery } from "@tanstack/react-query";
=======
import type { DocumentSearchItem, DocumentSearchResponse } from "@/types";
import { useQuery } from "@tanstack/react-query";
import { useCallback, useEffect, useState } from "react";
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

const SEARCH_DEBOUNCE_MS = 300;
/** 30s stale time — documents don't change frequently enough to need fresh data on each keystroke. */
const SEARCH_STALE_TIME_MS = 30_000;
<<<<<<< HEAD

/**
 * Returns a list of matching DocumentSearchItems.
 * When `query` is empty, returns the 20 most recently created completed docs.
=======
/** First window of documents shown when the picker opens. */
const INITIAL_PAGE_SIZE = 20;
/** Backend hard cap for /documents/search page_size. */
const MAX_PAGE_SIZE = 50;
/** Increment applied by each "load more" action (clamped to MAX_PAGE_SIZE). */
const PAGE_SIZE_INCREMENT = 30;

/**
 * Returns matching DocumentSearchItems plus paging affordances.
 * When `query` is empty, returns the most recently created completed docs.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
 */
export function useDocumentSearch(
  query: string,
  enabled = true,
): {
  data: DocumentSearchItem[];
<<<<<<< HEAD
=======
  total: number;
  hasMore: boolean;
  loadMore: () => void;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  isLoading: boolean;
  isError: boolean;
} {
  const debounced = useDebounce(query.trim(), SEARCH_DEBOUNCE_MS);
<<<<<<< HEAD

  const result = useQuery<DocumentSearchItem[]>({
    queryKey: ["documents", "search", debounced],
    queryFn: async () => {
      const res = await searchDocuments({
        q: debounced || undefined,
        page_size: 20,
        status: "completed",
      });
      return res.items;
    },
=======
  const [limit, setLimit] = useState(INITIAL_PAGE_SIZE);

  // Reset to the first window whenever the effective query changes so a fresh
  // search always starts from the top of its result set.
  useEffect(() => {
    setLimit(INITIAL_PAGE_SIZE);
  }, [debounced]);

  const result = useQuery<DocumentSearchResponse>({
    queryKey: ["documents", "search", debounced, limit],
    queryFn: () =>
      searchDocuments({
        q: debounced || undefined,
        page_size: limit,
        status: "completed",
      }),
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    enabled,
    staleTime: SEARCH_STALE_TIME_MS,
    gcTime: 60_000,
    // Show stale results while fetching new ones — prevents flicker
    placeholderData: (prev) => prev,
  });

<<<<<<< HEAD
  return {
    data: result.data ?? [],
=======
  const loadMore = useCallback(() => {
    setLimit((l) => Math.min(l + PAGE_SIZE_INCREMENT, MAX_PAGE_SIZE));
  }, []);

  const total = result.data?.total ?? 0;
  return {
    data: result.data?.items ?? [],
    total,
    // `has_more` is total > current window; also hide the action once the backend
    // cap is reached (no offset available to page further).
    hasMore: (result.data?.has_more ?? false) && limit < MAX_PAGE_SIZE,
    loadMore,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    isLoading: result.isLoading || result.isFetching,
    isError: result.isError,
  };
}
