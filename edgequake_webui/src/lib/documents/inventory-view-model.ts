/**
 * SPEC-099 LAW-099-7 / LAW-099-8 — single inventory view-model for counts.
 *
 * Header, filter chips, and table rows must share one filtered view of truth.
 * When the fetch is capped, never imply a complete corpus.
 */

import {
  getDocumentDisplayStatus,
  normalizeStatus,
  type DocumentStatus,
} from "@/lib/documents/status-domain";
import type { Document } from "@/types";

export const VIRTUAL_PAGE_SIZE = 100;

export interface StatusCounts {
  all: number;
  pending: number;
  processing: number;
  completed: number;
  failed: number;
  partial_failure: number;
  cancelled: number;
}

export interface InventoryViewModel {
  /** Rows shown in the table (filtered + sorted page) */
  rows: Document[];
  /** Count of rows in the current view */
  filteredCount: number;
  /** Fetched item count before client search filter */
  fetchedCount: number;
  /** Page size / fetch cap */
  pageSize: number;
  /** True when fetch hit the cap (may be truncated) */
  isTruncated: boolean;
  /**
   * Known corpus total when API provides it (status_counts sum or total field).
   * Null when unknown.
   */
  totalKnown: number | null;
  /** Status chip counts (view-model SSOT) */
  statusCounts: StatusCounts;
  /**
   * Header badge label — e.g. "17", "17 of 240", or "100+".
   */
  countLabel: string;
  /** Affordance text when truncated */
  overflowLabel: string | null;
}

const EMPTY_COUNTS: StatusCounts = {
  all: 0,
  pending: 0,
  processing: 0,
  completed: 0,
  failed: 0,
  partial_failure: 0,
  cancelled: 0,
};

function bucketDisplayStatus(status: DocumentStatus): keyof StatusCounts | null {
  switch (status) {
    case "pending":
    case "queued":
    case "cleaning":
    case "held":
      return "pending";
    case "completed":
    case "indexed":
    case "partial_success":
      return "completed";
    case "failed":
    case "delete_failed":
    case "dead_letter":
      return "failed";
    case "partial_failure":
      return "partial_failure";
    case "cancelled":
      return "cancelled";
    default:
      if (
        status === "processing" ||
        status === "uploading" ||
        status === "converting" ||
        status === "preprocessing" ||
        status === "chunking" ||
        status === "extracting" ||
        status === "gleaning" ||
        status === "merging" ||
        status === "summarizing" ||
        status === "embedding" ||
        status === "re_embedding" ||
        status === "storing" ||
        status === "indexing" ||
        status === "deleting" ||
        status === "stopping" ||
        status === "cancelling"
      ) {
        return "processing";
      }
      return null;
  }
}

/** Count using domain display status (not raw wire status). */
export function countClientStatusCounts(
  docs: Array<{
    status?: string | null;
    current_stage?: string | null;
    display_status?: string | null;
    ui_phase?: string | null;
  }>,
): StatusCounts {
  const counts: StatusCounts = { ...EMPTY_COUNTS, all: docs.length };
  for (const doc of docs) {
    const display = getDocumentDisplayStatus(doc);
    const bucket = bucketDisplayStatus(display);
    if (bucket && bucket !== "all") {
      counts[bucket] += 1;
    }
  }
  return counts;
}

export function buildInventoryViewModel(input: {
  fetchedItems: Document[];
  filteredRows: Document[];
  pageSize?: number;
  serverStatusCounts?: {
    pending: number;
    processing: number;
    completed: number;
    failed: number;
    partial_failure?: number;
    cancelled?: number;
  } | null;
  /** Optional API total when present */
  apiTotal?: number | null;
}): InventoryViewModel {
  const pageSize = input.pageSize ?? VIRTUAL_PAGE_SIZE;
  const fetchedCount = input.fetchedItems.length;
  const filteredCount = input.filteredRows.length;
  const isTruncated = fetchedCount >= pageSize;

  let statusCounts: StatusCounts;
  if (input.serverStatusCounts) {
    const pending = input.serverStatusCounts.pending;
    const processing = input.serverStatusCounts.processing;
    const completed = input.serverStatusCounts.completed;
    const failed = input.serverStatusCounts.failed;
    const partial_failure = input.serverStatusCounts.partial_failure || 0;
    const cancelled = input.serverStatusCounts.cancelled || 0;
    statusCounts = {
      all: pending + processing + completed + failed + partial_failure + cancelled,
      pending,
      processing,
      completed,
      failed,
      partial_failure,
      cancelled,
    };
  } else {
    // Client counts on the filtered view so chips match visible rows (LAW-099-8)
    statusCounts = countClientStatusCounts(input.filteredRows);
  }

  const totalKnown =
    typeof input.apiTotal === "number" && Number.isFinite(input.apiTotal)
      ? input.apiTotal
      : input.serverStatusCounts
        ? statusCounts.all
        : isTruncated
          ? null
          : fetchedCount;

  let countLabel: string;
  let overflowLabel: string | null = null;

  if (totalKnown !== null && totalKnown > filteredCount) {
    countLabel = `${filteredCount} of ${totalKnown}`;
    overflowLabel = `Showing ${filteredCount} of ${totalKnown}`;
  } else if (isTruncated && totalKnown === null) {
    countLabel = `${filteredCount}+`;
    overflowLabel = `Showing ${filteredCount}+ (fetch capped at ${pageSize})`;
  } else {
    countLabel = String(filteredCount);
    overflowLabel = null;
  }

  // When search filters a truncated page, chip "all" should match filtered rows
  // if we used client counts; when server counts exist, keep server "all" but
  // header uses filteredCount for the left badge.
  if (!input.serverStatusCounts) {
    statusCounts = { ...statusCounts, all: filteredCount };
  }

  return {
    rows: input.filteredRows,
    filteredCount,
    fetchedCount,
    pageSize,
    isTruncated,
    totalKnown,
    statusCounts,
    countLabel,
    overflowLabel,
  };
}

/** Normalize a raw status for filter matching (exported for tests). */
export function normalizeForFilter(status: string | null | undefined): DocumentStatus {
  return normalizeStatus(status);
}
