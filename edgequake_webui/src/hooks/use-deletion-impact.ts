/**
 * @module useDeletionImpact
 * @description Fetches pre-delete impact analysis for a document.
 *
 * WHY: Before confirming a destructive delete, the user should see exactly
 * what will be removed (entities, relationships, chunks). This hook fetches
 * the impact from GET /documents/{id}/deletion-impact and caches it for 30s
 * so repeat opens of the confirm dialog don't cause extra network requests.
 *
 * @implements SPEC-050: Impact preview before delete.
 * @implements AC-050-01: Impact counts shown before user can confirm.
 */
'use client';

import { getDeletionImpact, type DeletionImpact } from '@/lib/api/edgequake';
import { useQuery } from '@tanstack/react-query';

/**
 * Return type for the useDeletionImpact hook.
 */
export interface UseDeletionImpactReturn {
  /** Impact data, or null if not yet loaded. */
  impact: DeletionImpact | null;
  /** True while the impact is being fetched. */
  isLoading: boolean;
  /** Error if the fetch failed. */
  error: Error | null;
  /** Manually trigger a refetch (e.g. after document state changes). */
  refetch: () => void;
}

/**
 * Fetch and cache deletion impact for a document.
 *
 * @param documentId - The document to analyse. Pass `null` to disable the query.
 */
export function useDeletionImpact(
  documentId: string | null | undefined,
): UseDeletionImpactReturn {
  const query = useQuery({
    queryKey: ['deletion-impact', documentId],
    queryFn: () => getDeletionImpact(documentId!),
    // WHY: Only fetch when we have a document ID — avoids spurious requests.
    enabled: !!documentId,
    // WHY: 30s stale time — impact doesn't change rapidly; caches for repeat
    // dialog opens without an extra network round-trip.
    staleTime: 30_000,
    // WHY: Retry once in case of a transient network error, but don't block
    // the confirm flow — the dialog must still let the user delete even if
    // the impact analysis is unavailable.
    retry: 1,
    retryDelay: 500,
  });

  return {
    impact: query.data ?? null,
    isLoading: query.isLoading,
    error: query.error as Error | null,
    refetch: query.refetch,
  };
}
