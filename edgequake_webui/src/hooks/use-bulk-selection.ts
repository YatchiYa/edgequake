/**
 * @module useBulkSelection
 * @description Manages bulk document selection state and operations.
 * Extracted from DocumentManager for SRP compliance (OODA-16).
 *
 * WHY: Selection logic and bulk operations were inline in DocumentManager.
 * This hook:
 * - Encapsulates selectedIds state
 * - Provides selection handlers (all/one/clear)
 * - Provides bulk operation handlers with progress tracking
 * - Handles toast notifications and cache invalidation
 *
 * @implements FEAT0003 - Batch document processing
 * @implements UC0009 - User deletes documents from knowledge graph
 * @implements UC0008 - User reprocesses failed documents
 */
"use client";

import {
    deleteDocument,
    reprocessDocument,
    type ReprocessMode,
} from "@/lib/api/edgequake";
import { invalidateKnowledgeGraph } from "@/lib/cache-manager";
import type { Document } from "@/types";
import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

/**
 * Options for useBulkSelection hook.
 */
export interface UseBulkSelectionOptions {
  /**
   * Current list of documents (used for select all).
   * WHY: Need document IDs for select all operation.
   */
  documents: Document[];
  /**
   * SPEC-050 GAP-FIX: Callback to request confirmation before bulk delete.
   * WHY: Instead of deleting directly, the toolbar Delete button should open a
   * confirmation dialog with the selected documents listed. This callback lets
   * DocumentManager own the dialog state (SRP) while useBulkSelection owns selection.
   * If not provided, falls back to the old direct-delete behaviour (backward compat).
   */
  onDeleteRequested?: (selectedDocuments: Document[]) => void;
}

/**
 * Return type for useBulkSelection hook.
 */
export interface UseBulkSelectionReturn {
  /**
   * Set of currently selected document IDs.
   */
  selectedIds: Set<string>;

  /**
   * Number of selected documents.
   * WHY: Convenience getter for UI display.
   */
  selectedCount: number;

  /**
   * Whether all documents are selected.
   * WHY: For checkbox "all" state.
   */
  isAllSelected: boolean;

  /**
   * Select or deselect all documents.
   */
  handleSelectAll: (checked: boolean) => void;

  /**
   * Toggle selection for a single document.
   */
  handleSelectOne: (docId: string, checked: boolean) => void;

  /**
   * Clear all selections.
   * WHY: Used after bulk operations or by BatchActionsBar.
   */
  handleClearSelection: () => void;

  /**
   * Delete all selected documents.
   * Shows progress toast and invalidates cache.
   */
  handleBulkDelete: () => Promise<void>;

  /**
   * Reprocess all selected documents.
   * @param mode Reprocess intent: "entities" (reuse markdown, default) or
   *             "full" (re-run PDF -> markdown conversion for selected PDFs).
   * Shows progress toast and invalidates cache.
   */
  handleBulkReprocess: (mode?: ReprocessMode) => Promise<void>;

  /**
   * Whether a bulk delete operation is in progress.
   */
  isBulkDeleting: boolean;

  /**
   * Whether a bulk reprocess operation is in progress.
   */
  isBulkReprocessing: boolean;
}

/**
 * Hook for managing bulk document selection and operations.
 *
 * @example
 * ```tsx
 * const {
 *   selectedIds,
 *   selectedCount,
 *   isAllSelected,
 *   handleSelectAll,
 *   handleSelectOne,
 *   handleClearSelection,
 *   handleBulkDelete,
 *   handleBulkReprocess,
 * } = useBulkSelection({ documents });
 *
 * // In checkbox
 * <Checkbox
 *   checked={isAllSelected}
 *   onCheckedChange={handleSelectAll}
 * />
 *
 * // In BatchActionsBar
 * <BatchActionsBar
 *   selectedCount={selectedCount}
 *   onDelete={handleBulkDelete}
 *   onReprocess={handleBulkReprocess}
 *   onClear={handleClearSelection}
 * />
 * ```
 */
export function useBulkSelection({
  documents,
  onDeleteRequested,
}: UseBulkSelectionOptions): UseBulkSelectionReturn {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  // Selection state
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // Loading states
  const [isBulkDeleting, setIsBulkDeleting] = useState(false);
  const [isBulkReprocessing, setIsBulkReprocessing] = useState(false);

  // Computed values
  const selectedCount = selectedIds.size;
  const isAllSelected =
    selectedCount === documents.length && documents.length > 0;

  /**
   * Select or deselect all documents.
   * WHY: Bulk selection improves efficiency for batch operations.
   */
  const handleSelectAll = useCallback(
    (checked: boolean) => {
      if (checked) {
        setSelectedIds(new Set(documents.map((d) => d.id)));
      } else {
        setSelectedIds(new Set());
      }
    },
    [documents],
  );

  /**
   * Toggle selection for a single document.
   */
  const handleSelectOne = useCallback((docId: string, checked: boolean) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (checked) {
        next.add(docId);
      } else {
        next.delete(docId);
      }
      return next;
    });
  }, []);

  /**
   * Clear all selections.
   * WHY: Reset after bulk operations or user request.
   */
  const handleClearSelection = useCallback(() => {
    setSelectedIds(new Set());
  }, []);

  /**
   * Delete all selected documents.
   *
   * SPEC-050 GAP-FIX: If `onDeleteRequested` is provided, delegate to the
   * caller's confirmation dialog instead of deleting directly.
   * WHY: Maintains SRP — useBulkSelection owns selection, DocumentManager
   * owns the confirmation dialog and the actual delete flow.
   */
  const handleBulkDelete = useCallback(async () => {
    const idsToDelete = Array.from(selectedIds);
    if (idsToDelete.length === 0) return;

    // SPEC-050: Route through confirmation dialog if callback is provided.
    if (onDeleteRequested) {
      const selectedDocuments = documents.filter((d) => idsToDelete.includes(d.id));
      onDeleteRequested(selectedDocuments);
      return;
    }

    // Legacy fallback: direct delete without confirmation (kept for back-compat).
    setIsBulkDeleting(true);
    let successCount = 0;
    let errorCount = 0;
    let lastError: string | undefined;

    const toastId = toast.loading(
      t("documents.bulk.deleting", "Deleting {{count}} document(s)…", {
        count: idsToDelete.length,
      }),
    );

    try {
      for (let i = 0; i < idsToDelete.length; i++) {
        const id = idsToDelete[i];
        toast.loading(
          t("documents.bulk.deleteProgress", "Deleting {{current}} of {{total}}…", {
            current: i + 1,
            total: idsToDelete.length,
          }),
          { id: toastId },
        );
        try {
          await deleteDocument(id);
          successCount++;
        } catch (err) {
          errorCount++;
          lastError =
            err instanceof Error
              ? err.message
              : t("common.unknownError", "Unknown error");
        }
      }

      toast.dismiss(toastId);

      if (successCount > 0) {
        toast.success(
          t("documents.bulk.deleteSuccess", { count: successCount }) ||
            `Deleted ${successCount} document(s)`,
        );
        queryClient.invalidateQueries({ queryKey: ["documents"] });
        invalidateKnowledgeGraph(queryClient);
      }
      if (errorCount > 0) {
        toast.error(
          t("documents.bulk.deleteFailed", { count: errorCount }) ||
            `Failed to delete ${errorCount} document(s)`,
          {
            description: lastError,
          },
        );
      }
    } finally {
      setIsBulkDeleting(false);
      setSelectedIds(new Set());
    }
  }, [selectedIds, queryClient, t]);

  /**
   * Reprocess all selected documents.
   * WHY: Bulk reprocess is more efficient than one-by-one.
   * Uses optimistic update to immediately show "pending" status for all selected docs.
   */
  const handleBulkReprocess = useCallback(
    async (mode: ReprocessMode = "entities") => {
      const idsToReprocess = Array.from(selectedIds);
      if (idsToReprocess.length === 0) return;

      setIsBulkReprocessing(true);
      let successCount = 0;
      let errorCount = 0;

      // Cancel outgoing refetches and snapshot for rollback
      await queryClient.cancelQueries({ queryKey: ["documents"] });
      const previousDocuments = queryClient.getQueriesData({
        queryKey: ["documents"],
      });

      // Optimistically update all selected documents to "pending"
      const idsSet = new Set(idsToReprocess);
      queryClient.setQueriesData(
        { queryKey: ["documents"] },
        (oldData: { items?: Document[] } | undefined) => {
          if (!oldData?.items) return oldData;
          return {
            ...oldData,
            items: oldData.items.map((doc: Document) =>
              idsSet.has(doc.id)
                ? {
                    ...doc,
                    status: "pending",
                    error_message: undefined,
                    current_stage: undefined,
                  }
                : doc,
            ),
          };
        },
      );

      try {
        for (const id of idsToReprocess) {
          try {
            const doc = documents.find((d) => d.id === id);
            if (!doc?.id) {
              errorCount++;
              continue;
            }
            // WHY: reprocessDocument expects the document's `id` (KV metadata key),
            // not its track_id.  Using track_id caused silent no-ops on the backend.
            // mode propagates the bulk re-conversion intent to the backend.
            await reprocessDocument(doc.id, true, mode);
            successCount++;
          } catch {
            errorCount++;
          }
        }

        if (successCount > 0) {
          toast.success(
            t("documents.bulk.reprocessSuccess", { count: successCount }) ||
              `Queued ${successCount} document(s) for reprocessing`,
          );
          queryClient.invalidateQueries({ queryKey: ["documents"] });
          queryClient.invalidateQueries({ queryKey: ["pipeline-status"] });
        }
        if (errorCount > 0) {
          // Partial failure: rollback optimistic update for failed ones
          // and refetch to get accurate state
          toast.error(
            t("documents.bulk.reprocessFailed", { count: errorCount }) ||
              `Failed to queue ${errorCount} document(s)`,
          );
          queryClient.invalidateQueries({ queryKey: ["documents"] });
        }
      } catch {
        // Full failure: rollback all optimistic updates
        for (const [queryKey, data] of previousDocuments) {
          queryClient.setQueryData(queryKey, data);
        }
      } finally {
        setIsBulkReprocessing(false);
        setSelectedIds(new Set());
      }
    },
    [selectedIds, documents, queryClient, t],
  );

  return {
    selectedIds,
    selectedCount,
    isAllSelected,
    handleSelectAll,
    handleSelectOne,
    handleClearSelection,
    handleBulkDelete,
    handleBulkReprocess,
    isBulkDeleting,
    isBulkReprocessing,
  };
}

export default useBulkSelection;
