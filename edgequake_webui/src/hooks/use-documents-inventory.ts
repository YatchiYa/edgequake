/**
 * SPEC-099 — inventory controller: queries + filter VM + overflow honesty.
 */
"use client";

import { useDocumentFiltering } from "@/hooks/use-document-filtering";
import { useDocumentQueries } from "@/hooks/use-document-queries";
import {
  VIRTUAL_PAGE_SIZE,
  buildInventoryViewModel,
  type InventoryViewModel,
} from "@/lib/documents/inventory-view-model";
import type { SortDirection, SortField } from "@/lib/documents/document-sort";
import type { DocStatus } from "@/hooks/use-document-preferences";
import { useMemo } from "react";

export interface UseDocumentsInventoryOptions {
  tenantId: string | null;
  workspaceId: string | null;
  searchQuery: string;
  statusFilter: DocStatus;
  sortField: SortField;
  sortDirection: SortDirection;
  pageSize?: number;
}

export function useDocumentsInventory(options: UseDocumentsInventoryOptions) {
  const pageSize = options.pageSize ?? VIRTUAL_PAGE_SIZE;
  const queries = useDocumentQueries({
    tenantId: options.tenantId,
    workspaceId: options.workspaceId,
    currentPage: 1,
    pageSize,
    statusFilter: options.statusFilter,
  });

  const filtering = useDocumentFiltering({
    documents: queries.data?.items || [],
    searchQuery: options.searchQuery,
    statusFilter: options.statusFilter,
    sortField: options.sortField,
    sortDirection: options.sortDirection,
    pageSize,
    // Prefer client domain counts for filter honesty when search is active;
    // otherwise use server counts when present.
    serverStatusCounts: options.searchQuery.trim()
      ? undefined
      : queries.data?.status_counts,
  });

  const inventory: InventoryViewModel = useMemo(() => {
    const apiTotal =
      typeof (queries.data as { total?: number } | undefined)?.total === "number"
        ? (queries.data as { total: number }).total
        : null;
    return buildInventoryViewModel({
      fetchedItems: queries.data?.items || [],
      filteredRows: filtering.documents,
      pageSize,
      serverStatusCounts: options.searchQuery.trim()
        ? null
        : queries.data?.status_counts,
      apiTotal,
    });
  }, [
    queries.data,
    filtering.documents,
    pageSize,
    options.searchQuery,
  ]);

  return {
    ...queries,
    documents: filtering.documents,
    totalCount: filtering.totalCount,
    statusCounts: inventory.statusCounts,
    allDocuments: filtering.allDocuments,
    inventory,
    pageSize,
  };
}
