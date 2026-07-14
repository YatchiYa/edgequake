/**
 * @module useDocumentTitle
 * @description Dynamic page title based on document count and processing status.
 * Extracted from DocumentManager for SRP compliance (OODA-22).
 *
 * WHY: Users can see document count in browser tab without switching.
 *
 * @implements FEAT0604 - Dynamic page titles
 */
"use client";

import { useEffect } from "react";

/**
 * Options for useDocumentTitle hook.
 */
export interface UseDocumentTitleOptions {
  /** Total document count */
  totalCount: number;
  /** Number of actively working documents (not queued) */
  processingCount: number;
  /** Number of queued / waiting documents */
  queuedCount?: number;
  /** Base title to use */
  baseTitle?: string;
}

/**
 * Hook to update page title with document and processing counts.
 *
 * Title patterns:
 * - Processing: "⏳ Processing (2) | Documents (42) - EdgeQuake"
 * - Normal: "Documents (42) - EdgeQuake"
 * - Empty: "Documents - EdgeQuake"
 *
 * @example
 * ```tsx
 * useDocumentTitle({
 *   totalCount,
 *   processingCount: pipelineStatus?.running_tasks || 0,
 * });
 * ```
 */
export function useDocumentTitle(options: UseDocumentTitleOptions): void {
  const {
    totalCount,
    processingCount,
    queuedCount = 0,
    baseTitle = "Documents - EdgeQuake",
  } = options;

  useEffect(() => {
    const count = totalCount || 0;
    const processing = processingCount || 0;
    const queued = queuedCount || 0;

    if (processing > 0) {
      document.title = `⏳ Working (${processing}) | Documents (${count}) - EdgeQuake`;
    } else if (queued > 0) {
      document.title = `⏸ Queued (${queued}) | Documents (${count}) - EdgeQuake`;
    } else if (count > 0) {
      document.title = `Documents (${count}) - EdgeQuake`;
    } else {
      document.title = baseTitle;
    }

    return () => {
      document.title = baseTitle;
    };
  }, [totalCount, processingCount, queuedCount, baseTitle]);
}

export default useDocumentTitle;
