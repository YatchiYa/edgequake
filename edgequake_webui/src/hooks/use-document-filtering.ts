/**
 * @module useDocumentFiltering
 * @description Client-side document filtering and sorting logic.
 * Extracted from DocumentManager for SRP compliance (OODA-19).
 *
 * WHY: Filter and sort functions were inline in DocumentManager.
 * This hook provides:
 * - Search filtering (title, file_name, id)
 * - Status filtering
 * - Multi-field sorting
 *
 * @implements FEAT0401 - Document search and filtering
 */
"use client";

import type { Document } from "@/types";
import {
  sortDocuments,
  type SortDirection,
  type SortField,
} from "@/lib/documents/document-sort";
<<<<<<< HEAD
import { useMemo } from "react";
import type { DocStatus } from "./use-document-preferences";
=======
import {
  countClientStatusCounts,
  type StatusCounts,
} from "@/lib/documents/inventory-view-model";
import { useMemo } from "react";
import type { DocStatus } from "./use-document-preferences";

export type { StatusCounts };
export { countClientStatusCounts };
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

/**
 * Options for useDocumentFiltering hook.
 */
export interface UseDocumentFilteringOptions {
  /** Raw documents from API */
  documents: Document[];
  /** Search query string */
  searchQuery: string;
  /** Status filter value */
  statusFilter: DocStatus;
  /** Sort field */
  sortField: SortField;
  /** Sort direction */
  sortDirection: SortDirection;
  /** Page size for pagination */
  pageSize: number;
  /** Server-side status counts (optional, for efficiency) */
  serverStatusCounts?: {
    pending: number;
    processing: number;
    completed: number;
    failed: number;
    partial_failure?: number;
    cancelled?: number;
  };
}

/**
<<<<<<< HEAD
 * Status counts for document status tabs.
 */
export interface StatusCounts {
  all: number;
  pending: number;
  processing: number;
  completed: number;
  failed: number;
  partial_failure: number;
  cancelled: number;
}

/**
 * Client-side status counts (SPEC-057 P0: failed excludes cancelled).
 */
export function countClientStatusCounts(
  docs: Array<{ status?: string | null }>,
): StatusCounts {
  return {
    all: docs.length,
    pending: docs.filter((d) => d.status === "pending").length,
    processing: docs.filter((d) => d.status === "processing").length,
    completed: docs.filter(
      (d) => !d.status || d.status === "completed" || d.status === "indexed",
    ).length,
    failed: docs.filter((d) => d.status === "failed").length,
    partial_failure: docs.filter((d) => d.status === "partial_failure").length,
    cancelled: docs.filter((d) => d.status === "cancelled").length,
  };
}

/**
=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
 * Return type for useDocumentFiltering hook.
 */
export interface UseDocumentFilteringReturn {
  /** Filtered and sorted documents */
  documents: Document[];
  /** Total count of filtered documents */
  totalCount: number;
  /** Total number of pages */
  totalPages: number;
  /** All documents (unfiltered) */
  allDocuments: Document[];
  /** Status counts for tabs */
  statusCounts: StatusCounts;
}

/**
 * Filter documents by search query.
 *
 * SPEC-084 / GH-319: status filtering is server-side (before pagination).
 * Client must not re-filter status on a truncated page.
 */
function filterDocuments(docs: Document[], searchQuery: string): Document[] {
  if (!searchQuery.trim()) {
    return docs;
  }
<<<<<<< HEAD

  // Apply status filter
  if (statusFilter !== "all") {
    filtered = filtered.filter((doc) => {
      const docStatus = doc.status || "completed";
      return docStatus === statusFilter;
    });
  }

  return filtered;
=======
  const query = searchQuery.toLowerCase().trim();
  return docs.filter((doc) => {
    const title = doc.title?.toLowerCase() || "";
    const fileName = doc.file_name?.toLowerCase() || "";
    return (
      title.includes(query) ||
      fileName.includes(query) ||
      doc.id.includes(query)
    );
  });
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

/**
 * Hook for client-side document filtering and sorting.
 *
 * @example
 * ```tsx
 * const { documents, totalCount, totalPages, allDocuments } = useDocumentFiltering({
 *   documents: data?.items || [],
 *   searchQuery,
 *   statusFilter,
 *   sortField,
 *   sortDirection,
 *   pageSize,
 * });
 * ```
 */
export function useDocumentFiltering(
  options: UseDocumentFilteringOptions,
): UseDocumentFilteringReturn {
  const {
    documents: rawDocuments,
    searchQuery,
    sortField,
    sortDirection,
    pageSize,
    serverStatusCounts,
  } = options;

  const allDocuments = rawDocuments;

  // Memoize filtering and sorting for performance
  const documents = useMemo(() => {
    const filtered = filterDocuments(rawDocuments, searchQuery);
    return sortDocuments(filtered, sortField, sortDirection);
  }, [rawDocuments, searchQuery, sortField, sortDirection]);

  const totalCount = documents.length;
  const totalPages = Math.ceil(totalCount / pageSize);

  // Calculate status counts (use server-side if available for efficiency)
  const statusCounts = useMemo<StatusCounts>(() => {
    if (serverStatusCounts) {
      const pending = serverStatusCounts.pending;
      const processing = serverStatusCounts.processing;
      const completed = serverStatusCounts.completed;
      const failed = serverStatusCounts.failed;
      const partial_failure = serverStatusCounts.partial_failure || 0;
      const cancelled = serverStatusCounts.cancelled || 0;
      return {
        all:
          pending +
          processing +
          completed +
          failed +
          partial_failure +
          cancelled,
        pending,
        processing,
        completed,
        failed,
        partial_failure,
        cancelled,
      };
    }
    return countClientStatusCounts(allDocuments);
  }, [allDocuments, serverStatusCounts]);

  return {
    documents,
    totalCount,
    totalPages,
    allDocuments,
    statusCounts,
  };
}

export default useDocumentFiltering;
